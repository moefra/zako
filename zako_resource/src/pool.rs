use std::collections::VecDeque;

use parking_lot::Mutex;
use sysinfo::System;
use thiserror::Error;
use tokio::sync::oneshot;
use zako_shared::{FastMap, StableMap};

use crate::{
    RequestPriority, ResourceDescriptor, ResourceKey, ResourcePolicy, ResourceRange,
    allocation::{ResourceGrant, ResourceRequest},
    heuristics,
    shares::ResourceUnitShares,
};

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ResourcePoolError {
    #[error("unknown resource key: {0:?}")]
    UnknownResourceKey(ResourceKey),
    #[error("invalid range for {key:?}: min={min:?}, max={max:?}")]
    InvalidRange {
        key: ResourceKey,
        min: ResourceUnitShares,
        max: ResourceUnitShares,
    },
    #[error(
        "request for {key:?} is below granularity: requested={requested:?}, granularity={granularity:?}"
    )]
    BelowGranularity {
        key: ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    },
    #[error(
        "request for {key:?} is not divisible by granularity: requested={requested:?}, granularity={granularity:?}"
    )]
    NotDivisibleByGranularity {
        key: ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    },
    #[error("request for {key:?} exceeds hard limit: requested={requested:?}, limit={limit:?}")]
    ExceedsHardLimit {
        key: ResourceKey,
        requested: ResourceUnitShares,
        limit: ResourceUnitShares,
    },
    #[error("arithmetic overflow in resource pool")]
    ArithmeticOverflow,
    #[error("resource key mismatch with descriptor key")]
    KeyMismatch,
}

#[derive(Debug, Clone)]
pub struct ResourcePoolSnapshot {
    pub totals: FastMap<ResourceKey, Option<ResourceUnitShares>>,
    pub used: FastMap<ResourceKey, ResourceUnitShares>,
    pub rests: FastMap<ResourceKey, ResourceUnitShares>,
    pub queued: usize,
}

#[derive(Debug)]
struct WaiterEntry {
    id: u64,
    request: ResourceRequest,
    tx: oneshot::Sender<ResourceGrant>,
}

#[derive(Debug)]
struct ResourcePoolInner {
    archtypes: FastMap<ResourceKey, ResourceDescriptor>,
    rests: FastMap<ResourceKey, ResourceUnitShares>,
    used: FastMap<ResourceKey, ResourceUnitShares>,
    waiters: StableMap<RequestPriority, VecDeque<WaiterEntry>>,
    next_waiter_id: u64,
}

#[derive(Debug)]
pub struct ResourcePool {
    inner: Mutex<ResourcePoolInner>,
}

impl ResourcePool {
    pub fn new(
        resources: FastMap<ResourceKey, ResourceDescriptor>,
    ) -> Result<Self, ResourcePoolError> {
        let mut rests = FastMap::default();

        for (key, descriptor) in resources.iter() {
            if !key.eq(&descriptor.key) {
                return Err(ResourcePoolError::KeyMismatch);
            }

            if let Some(total) = descriptor.total {
                let effective = match descriptor.policy {
                    ResourcePolicy::Hard => total,
                    ResourcePolicy::Soft { max_overcommit } => ResourceUnitShares::from_shares(
                        total.as_shares().saturating_add(max_overcommit.as_shares()),
                    ),
                };
                rests.insert(key.clone(), effective);
            }
        }

        Ok(Self {
            inner: Mutex::new(ResourcePoolInner {
                archtypes: resources,
                rests,
                used: FastMap::default(),
                waiters: StableMap::default(),
                next_waiter_id: 0,
            }),
        })
    }

    pub fn heuristic(system: &System) -> Result<Self, ResourcePoolError> {
        let mut resources: FastMap<_, _> = Default::default();

        let mut init =
            |key: ResourceKey, method: &'static dyn Fn(&System) -> ResourceUnitShares| {
                resources.insert(
                    key.clone(),
                    ResourceDescriptor::new(
                        key,
                        Some(method(system)),
                        ResourceUnitShares::from_shares(1),
                        ResourcePolicy::Hard,
                    ),
                );
            };

        init(ResourceKey::Processor, &heuristics::cpu);
        init(ResourceKey::Memory, &heuristics::memory);
        init(ResourceKey::Disk, &heuristics::disk);
        init(ResourceKey::Network, &heuristics::network);

        Self::new(resources)
    }

    pub fn get_archtype(&self, key: &ResourceKey) -> Option<ResourceDescriptor> {
        self.inner.lock().archtypes.get(key).cloned()
    }

    pub fn try_acquire(
        &self,
        request: &ResourceRequest,
    ) -> Result<Option<ResourceGrant>, ResourcePoolError> {
        let mut inner = self.inner.lock();
        Self::try_allocate_locked(&mut inner, request)
    }

    pub async fn acquire(
        &self,
        request: ResourceRequest,
    ) -> Result<ResourceHolder<'_>, ResourcePoolError> {
        let (receiver, waiter_id) = {
            let mut inner = self.inner.lock();

            if let Some(grant) = Self::try_allocate_locked(&mut inner, &request)? {
                return Ok(ResourceHolder { pool: self, grant });
            }

            let waiter_id = inner.next_waiter_id;
            inner.next_waiter_id = inner.next_waiter_id.wrapping_add(1);

            let (tx, receiver) = oneshot::channel();
            inner
                .waiters
                .entry(request.priority)
                .or_default()
                .push_back(WaiterEntry {
                    id: waiter_id,
                    request,
                    tx,
                });

            (receiver, waiter_id)
        };

        let mut cleanup = WaiterCleanup {
            pool: self,
            waiter_id,
            active: true,
        };

        let grant = match receiver.await {
            Ok(grant) => grant,
            Err(_) => return Err(ResourcePoolError::ArithmeticOverflow),
        };

        cleanup.disarm();
        Ok(ResourceHolder { pool: self, grant })
    }

    pub fn snapshot(&self) -> ResourcePoolSnapshot {
        let inner = self.inner.lock();

        let mut totals = FastMap::default();
        let mut used = FastMap::default();
        let mut rests = FastMap::default();

        for (key, descriptor) in inner.archtypes.iter() {
            let effective_total = Self::effective_total_snapshot(descriptor);
            totals.insert(key.clone(), effective_total);

            let used_amount = match (effective_total, inner.rests.get(key).copied()) {
                (Some(total_effective), Some(rest)) => total_effective
                    .as_shares()
                    .checked_sub(rest.as_shares())
                    .map(ResourceUnitShares::from_shares)
                    .unwrap_or_else(|| {
                        inner
                            .used
                            .get(key)
                            .copied()
                            .unwrap_or(ResourceUnitShares::from_shares(0))
                    }),
                _ => inner
                    .used
                    .get(key)
                    .copied()
                    .unwrap_or(ResourceUnitShares::from_shares(0)),
            };

            used.insert(key.clone(), used_amount);

            if let Some(rest) = inner.rests.get(key).copied() {
                rests.insert(key.clone(), rest);
            }
        }

        let queued = inner.waiters.values().map(VecDeque::len).sum();

        ResourcePoolSnapshot {
            totals,
            used,
            rests,
            queued,
        }
    }

    /// get maximum available resource unit we can use,ignoring used resource.
    ///
    /// None for unlimited
    fn get_maximum_available(
        descriptor: &ResourceDescriptor,
    ) -> Result<Option<ResourceUnitShares>, ResourcePoolError> {
        let Some(total) = descriptor.total else {
            return Ok(None);
        };

        let effective = match descriptor.policy {
            ResourcePolicy::Hard => total,
            ResourcePolicy::Soft { max_overcommit } => total
                .checked_add(max_overcommit)
                .ok_or(ResourcePoolError::ArithmeticOverflow)?,
        };

        Ok(Some(effective))
    }

    fn effective_total_snapshot(descriptor: &ResourceDescriptor) -> Option<ResourceUnitShares> {
        let total = descriptor.total?;

        match descriptor.policy {
            ResourcePolicy::Hard => Some(total),
            ResourcePolicy::Soft { max_overcommit } => Some(ResourceUnitShares::from_shares(
                total.as_shares().saturating_add(max_overcommit.as_shares()),
            )),
        }
    }

    /// Make sure that:
    /// - The granularity is non-zero
    /// - The requested amount is within the granularity of the resource
    fn validate_requested_amount(
        key: &ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    ) -> Result<(), ResourcePoolError> {
        if granularity.as_shares() == 0 {
            return Err(ResourcePoolError::ArithmeticOverflow);
        }

        if requested < granularity {
            return Err(ResourcePoolError::BelowGranularity {
                key: key.clone(),
                requested,
                granularity,
            });
        }

        if !requested
            .as_shares()
            .is_multiple_of(granularity.as_shares())
        {
            return Err(ResourcePoolError::NotDivisibleByGranularity {
                key: key.clone(),
                requested,
                granularity,
            });
        }

        Ok(())
    }

    /// Make sure that:
    /// - The range is valid (min <= max)
    /// - The range is within the granularity of the resource
    /// - The range is within the hard limit of the resource if it has one
    fn validate_range_for_descriptor(
        key: &ResourceKey,
        descriptor: &ResourceDescriptor,
        range: &ResourceRange,
    ) -> Result<(), ResourcePoolError> {
        if range.min > range.max {
            return Err(ResourcePoolError::InvalidRange {
                key: key.clone(),
                min: range.min,
                max: range.max,
            });
        }

        Self::validate_requested_amount(key, range.min, descriptor.granularity)?;
        Self::validate_requested_amount(key, range.max, descriptor.granularity)?;

        if let (ResourcePolicy::Hard, Some(limit)) = (&descriptor.policy, descriptor.total)
            && range.min > limit
        {
            return Err(ResourcePoolError::ExceedsHardLimit {
                key: key.clone(),
                requested: range.min,
                limit,
            });
        }

        Ok(())
    }

    fn try_allocate_locked(
        inner: &mut ResourcePoolInner,
        request: &ResourceRequest,
    ) -> Result<Option<ResourceGrant>, ResourcePoolError> {
        let mut grant_items = FastMap::default();
        let mut used_updates: Vec<(ResourceKey, ResourceUnitShares)> = Vec::new();
        let mut rest_updates: Vec<(ResourceKey, ResourceUnitShares)> = Vec::new();

        // handle every request
        for (key, range) in request.items.iter() {
            let descriptor = inner
                .archtypes
                .get(key)
                .ok_or_else(|| ResourcePoolError::UnknownResourceKey(key.clone()))?;

            Self::validate_range_for_descriptor(key, descriptor, range)?;

            let grant = match Self::get_maximum_available(descriptor)? {
                None => range.max,
                Some(_) => {
                    let available = inner
                        .rests
                        .get(key)
                        .copied()
                        .unwrap_or(ResourceUnitShares::from_shares(0));

                    if available < range.min {
                        return Ok(None);
                    }

                    available.min(range.max)
                }
            };

            let current_used = inner
                .used
                .get(key)
                .copied()
                .unwrap_or(ResourceUnitShares::from_shares(0));
            let next_used = current_used
                .checked_add(grant)
                .ok_or(ResourcePoolError::ArithmeticOverflow)?;
            used_updates.push((key.clone(), next_used));

            if descriptor.total.is_some() {
                let current_rest = inner
                    .rests
                    .get(key)
                    .copied()
                    .ok_or(ResourcePoolError::ArithmeticOverflow)?;
                let next_rest = current_rest
                    .checked_sub(grant)
                    .ok_or(ResourcePoolError::ArithmeticOverflow)?;
                rest_updates.push((key.clone(), next_rest));
            }

            grant_items.insert(key.clone(), grant);
        }

        // request can be granted, update used and rest
        for (key, next_used) in used_updates {
            inner.used.insert(key, next_used);
        }

        for (key, next_rest) in rest_updates {
            inner.rests.insert(key, next_rest);
        }

        Ok(Some(ResourceGrant {
            items: grant_items,
            priority: request.priority,
        }))
    }

    fn release_grant_locked(
        inner: &mut ResourcePoolInner,
        grant: &ResourceGrant,
    ) -> Result<(), ResourcePoolError> {
        let mut used_updates: Vec<(ResourceKey, ResourceUnitShares)> = Vec::new();
        let mut rest_updates: Vec<(ResourceKey, ResourceUnitShares)> = Vec::new();

        for (key, granted) in grant.items.iter() {
            let current_used = inner
                .used
                .get(key)
                .copied()
                .unwrap_or(ResourceUnitShares::from_shares(0));
            let next_used = current_used
                .checked_sub(*granted)
                .ok_or(ResourcePoolError::ArithmeticOverflow)?;
            used_updates.push((key.clone(), next_used));

            if let Some(current_rest) = inner.rests.get(key).copied() {
                let next_rest = current_rest
                    .checked_add(*granted)
                    .ok_or(ResourcePoolError::ArithmeticOverflow)?;
                rest_updates.push((key.clone(), next_rest));
            }
        }

        for (key, next_used) in used_updates {
            inner.used.insert(key, next_used);
        }

        for (key, next_rest) in rest_updates {
            inner.rests.insert(key, next_rest);
        }

        Ok(())
    }

    fn pop_front_waiter_locked(
        inner: &mut ResourcePoolInner,
        priority: RequestPriority,
    ) -> Option<WaiterEntry> {
        let mut remove_key = false;
        let entry = if let Some(queue) = inner.waiters.get_mut(&priority) {
            let popped = queue.pop_front();
            remove_key = queue.is_empty();
            popped
        } else {
            None
        };

        if remove_key {
            inner.waiters.remove(&priority);
        }

        entry
    }

    fn remove_waiter_locked(inner: &mut ResourcePoolInner, waiter_id: u64) -> bool {
        let priorities: Vec<RequestPriority> = inner.waiters.keys().copied().collect();

        for priority in priorities {
            let mut remove_key = false;
            let mut removed = false;

            if let Some(queue) = inner.waiters.get_mut(&priority) {
                if let Some(index) = queue.iter().position(|entry| entry.id == waiter_id) {
                    queue.remove(index);
                    removed = true;
                }
                remove_key = queue.is_empty();
            }

            if remove_key {
                inner.waiters.remove(&priority);
            }

            if removed {
                return true;
            }
        }

        false
    }

    fn dispatch_waiters_locked(inner: &mut ResourcePoolInner) {
        loop {
            let priorities: Vec<RequestPriority> = inner.waiters.keys().copied().collect();
            let mut progressed = false;

            for priority in priorities {
                loop {
                    // check front waiter is available
                    let Some(front_closed) = inner
                        .waiters
                        .get(&priority)
                        .and_then(|queue| queue.front().map(|entry| entry.tx.is_closed()))
                    else {
                        break;
                    };

                    if front_closed {
                        let _ = Self::pop_front_waiter_locked(inner, priority);
                        progressed = true;
                        continue;
                    }

                    // get front waiter request
                    let Some(request) = inner
                        .waiters
                        .get(&priority)
                        .and_then(|queue| queue.front().map(|entry| entry.request.clone()))
                    else {
                        break;
                    };

                    // try allocate resource
                    let grant = match Self::try_allocate_locked(inner, &request) {
                        Ok(Some(grant)) => grant,
                        Ok(None) => break,
                        Err(_) => {
                            let _ = Self::pop_front_waiter_locked(inner, priority);
                            progressed = true;
                            continue;
                        }
                    };

                    // pop front waiter
                    let Some(entry) = Self::pop_front_waiter_locked(inner, priority) else {
                        let _ = Self::release_grant_locked(inner, &grant);
                        break;
                    };

                    // send grant to waiter
                    if entry.tx.send(grant.clone()).is_err() {
                        let _ = Self::release_grant_locked(inner, &grant);
                    }

                    progressed = true;
                }
            }

            if !progressed {
                break;
            }
        }
    }
}

pub struct ResourceHolder<'a> {
    pool: &'a ResourcePool,
    grant: ResourceGrant,
}

impl ResourceHolder<'_> {
    pub fn grant(&self) -> &ResourceGrant {
        &self.grant
    }
}

impl Drop for ResourceHolder<'_> {
    fn drop(&mut self) {
        let mut inner = self.pool.inner.lock();
        if ResourcePool::release_grant_locked(&mut inner, &self.grant).is_ok() {
            ResourcePool::dispatch_waiters_locked(&mut inner);
        }
    }
}

struct WaiterCleanup<'a> {
    pool: &'a ResourcePool,
    waiter_id: u64,
    active: bool,
}

impl WaiterCleanup<'_> {
    fn disarm(&mut self) {
        self.active = false;
    }
}

impl Drop for WaiterCleanup<'_> {
    fn drop(&mut self) {
        if !self.active {
            return;
        }

        let mut inner = self.pool.inner.lock();
        let _ = ResourcePool::remove_waiter_locked(&mut inner, self.waiter_id);
    }
}

#[cfg(test)]
mod tests {
    use std::{
        future::Future,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll, Wake, Waker},
    };

    use super::*;

    fn shares(value: u64) -> ResourceUnitShares {
        ResourceUnitShares::from_shares(value)
    }

    fn hard_descriptor(
        key: ResourceKey,
        total: Option<u64>,
        granularity: u64,
    ) -> ResourceDescriptor {
        ResourceDescriptor::new(
            key,
            total.map(shares),
            shares(granularity),
            ResourcePolicy::Hard,
        )
    }

    fn soft_descriptor(
        key: ResourceKey,
        total: Option<u64>,
        granularity: u64,
        overcommit: u64,
    ) -> ResourceDescriptor {
        ResourceDescriptor::new(
            key,
            total.map(shares),
            shares(granularity),
            ResourcePolicy::Soft {
                max_overcommit: shares(overcommit),
            },
        )
    }

    fn request_one(
        key: ResourceKey,
        min: u64,
        max: u64,
        priority: RequestPriority,
    ) -> ResourceRequest {
        let mut items = FastMap::default();
        items.insert(
            key,
            ResourceRange {
                min: shares(min),
                max: shares(max),
            },
        );
        ResourceRequest { items, priority }
    }

    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    fn poll_once<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
        let waker = Waker::from(Arc::new(NoopWake));
        let mut context = Context::from_waker(&waker);
        future.poll(&mut context)
    }

    #[test]
    fn new_initializes_rests_for_finite_resources() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(8), 1),
        );
        resources.insert(ResourceKey::Gpu, hard_descriptor(ResourceKey::Gpu, None, 1));

        let pool = ResourcePool::new(resources).unwrap();
        let snapshot = pool.snapshot();

        assert_eq!(
            snapshot.rests.get(&ResourceKey::Processor),
            Some(&shares(8))
        );
        assert!(!snapshot.rests.contains_key(&ResourceKey::Gpu));
    }

    #[test]
    fn try_acquire_success_and_drop_release() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(8), 1),
        );

        let pool = ResourcePool::new(resources).unwrap();
        let request = request_one(ResourceKey::Processor, 3, 3, RequestPriority::NORMAL);
        let grant = pool.try_acquire(&request);
        assert!(grant.is_ok());
        let grant = grant.unwrap_or_else(|_| unreachable!());
        assert!(grant.is_some());
        let holder = ResourceHolder {
            pool: &pool,
            grant: grant.unwrap_or_else(|| unreachable!()),
        };

        let snapshot = pool.snapshot();
        assert_eq!(
            snapshot.rests.get(&ResourceKey::Processor),
            Some(&shares(5))
        );

        drop(holder);

        let snapshot = pool.snapshot();
        assert_eq!(
            snapshot.rests.get(&ResourceKey::Processor),
            Some(&shares(8))
        );
    }

    #[tokio::test]
    async fn acquire_waits_then_wakes_on_release() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(4), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let first = pool
            .acquire(request_one(
                ResourceKey::Processor,
                4,
                4,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());

        let mut second = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            2,
            2,
            RequestPriority::NORMAL,
        )));
        assert!(matches!(poll_once(second.as_mut()), Poll::Pending));

        drop(first);

        let second = second.await;
        assert!(second.is_ok());
        let second = second.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            second.grant().items.get(&ResourceKey::Processor),
            Some(&shares(2))
        );
    }

    #[tokio::test]
    async fn priority_strict_ordering() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(1), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let first = pool
            .acquire(request_one(
                ResourceKey::Processor,
                1,
                1,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());

        let mut low = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            1,
            1,
            RequestPriority::LOW,
        )));
        assert!(matches!(poll_once(low.as_mut()), Poll::Pending));

        let mut high_one = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            1,
            1,
            RequestPriority::HIGH,
        )));
        assert!(matches!(poll_once(high_one.as_mut()), Poll::Pending));

        let mut high_two = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            1,
            1,
            RequestPriority::HIGH,
        )));
        assert!(matches!(poll_once(high_two.as_mut()), Poll::Pending));

        drop(first);

        let high_one = high_one.await;
        assert!(high_one.is_ok());
        let high_one = high_one.unwrap_or_else(|_| unreachable!());
        assert!(matches!(poll_once(high_two.as_mut()), Poll::Pending));
        assert!(matches!(poll_once(low.as_mut()), Poll::Pending));

        drop(high_one);

        let high_two = high_two.await;
        assert!(high_two.is_ok());
        let high_two = high_two.unwrap_or_else(|_| unreachable!());
        assert!(matches!(poll_once(low.as_mut()), Poll::Pending));

        drop(high_two);

        let low = low.await;
        assert!(low.is_ok());
        let low = low.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            low.grant().items.get(&ResourceKey::Processor),
            Some(&shares(1))
        );
    }

    #[tokio::test]
    async fn range_prefers_max_within_available() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(10), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let first = pool
            .acquire(request_one(
                ResourceKey::Processor,
                2,
                8,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            first.grant().items.get(&ResourceKey::Processor),
            Some(&shares(8))
        );
        drop(first);

        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(5), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();
        let second = pool
            .acquire(request_one(
                ResourceKey::Processor,
                2,
                8,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(second.is_ok());
        let second = second.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            second.grant().items.get(&ResourceKey::Processor),
            Some(&shares(5))
        );
    }

    #[test]
    fn hard_limit_rejects_unsatisfiable_min() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(4), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let request = request_one(ResourceKey::Processor, 5, 5, RequestPriority::NORMAL);
        let result = pool.try_acquire(&request);

        assert!(matches!(
            result,
            Err(ResourcePoolError::ExceedsHardLimit {
                key: ResourceKey::Processor,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn soft_limit_allows_overcommit_within_cap() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            soft_descriptor(ResourceKey::Processor, Some(4), 1, 2),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let first = pool
            .acquire(request_one(
                ResourceKey::Processor,
                6,
                6,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());
        assert_eq!(
            first.grant().items.get(&ResourceKey::Processor),
            Some(&shares(6))
        );

        let waitable = pool.try_acquire(&request_one(
            ResourceKey::Processor,
            7,
            7,
            RequestPriority::NORMAL,
        ));
        assert!(matches!(waitable, Ok(None)));

        drop(first);

        let second = pool
            .acquire(request_one(
                ResourceKey::Processor,
                6,
                6,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(second.is_ok());
    }

    #[test]
    fn unknown_resource_returns_error() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(2), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let request = request_one(ResourceKey::Memory, 1, 1, RequestPriority::NORMAL);
        let result = pool.try_acquire(&request);

        assert!(matches!(
            result,
            Err(ResourcePoolError::UnknownResourceKey(ResourceKey::Memory))
        ));
    }

    #[test]
    fn granularity_validation_errors() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(8), 2),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let below = pool.try_acquire(&request_one(
            ResourceKey::Processor,
            1,
            1,
            RequestPriority::NORMAL,
        ));
        assert!(matches!(
            below,
            Err(ResourcePoolError::BelowGranularity {
                key: ResourceKey::Processor,
                ..
            })
        ));

        let not_divisible = pool.try_acquire(&request_one(
            ResourceKey::Processor,
            3,
            3,
            RequestPriority::NORMAL,
        ));
        assert!(matches!(
            not_divisible,
            Err(ResourcePoolError::NotDivisibleByGranularity {
                key: ResourceKey::Processor,
                ..
            })
        ));
    }

    #[tokio::test]
    async fn cancel_waiting_request_is_cleaned() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(1), 1),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let first = pool
            .acquire(request_one(
                ResourceKey::Processor,
                1,
                1,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(first.is_ok());
        let first = first.unwrap_or_else(|_| unreachable!());

        let mut waiting = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            1,
            1,
            RequestPriority::NORMAL,
        )));
        assert!(matches!(poll_once(waiting.as_mut()), Poll::Pending));
        assert_eq!(pool.snapshot().queued, 1);

        drop(waiting);
        assert_eq!(pool.snapshot().queued, 0);

        drop(first);

        let next = pool
            .acquire(request_one(
                ResourceKey::Processor,
                1,
                1,
                RequestPriority::NORMAL,
            ))
            .await;
        assert!(next.is_ok());
    }

    #[tokio::test]
    async fn snapshot_reports_totals_used_rests_queued() {
        let mut resources = FastMap::default();
        resources.insert(
            ResourceKey::Processor,
            hard_descriptor(ResourceKey::Processor, Some(10), 1),
        );
        resources.insert(
            ResourceKey::Memory,
            soft_descriptor(ResourceKey::Memory, Some(4), 1, 2),
        );
        let pool = ResourcePool::new(resources).unwrap();

        let holder = pool
            .acquire({
                let mut items = FastMap::default();
                items.insert(
                    ResourceKey::Processor,
                    ResourceRange {
                        min: shares(3),
                        max: shares(3),
                    },
                );
                items.insert(
                    ResourceKey::Memory,
                    ResourceRange {
                        min: shares(5),
                        max: shares(5),
                    },
                );
                ResourceRequest {
                    items,
                    priority: RequestPriority::NORMAL,
                }
            })
            .await;
        assert!(holder.is_ok());
        let holder = holder.unwrap_or_else(|_| unreachable!());

        let mut waiting = Box::pin(pool.acquire(request_one(
            ResourceKey::Processor,
            8,
            8,
            RequestPriority::NORMAL,
        )));
        assert!(matches!(poll_once(waiting.as_mut()), Poll::Pending));

        let snapshot = pool.snapshot();

        assert_eq!(
            snapshot.totals.get(&ResourceKey::Processor),
            Some(&Some(shares(10)))
        );
        assert_eq!(
            snapshot.totals.get(&ResourceKey::Memory),
            Some(&Some(shares(6)))
        );
        assert_eq!(snapshot.used.get(&ResourceKey::Processor), Some(&shares(3)));
        assert_eq!(snapshot.used.get(&ResourceKey::Memory), Some(&shares(5)));
        assert_eq!(
            snapshot.rests.get(&ResourceKey::Processor),
            Some(&shares(7))
        );
        assert_eq!(snapshot.rests.get(&ResourceKey::Memory), Some(&shares(1)));
        assert_eq!(snapshot.queued, 1);

        drop(waiting);
        drop(holder);
    }
}
