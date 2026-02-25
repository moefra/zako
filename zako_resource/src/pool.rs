use std::sync::Arc;

use parking_lot::Mutex;
use tokio::sync::oneshot;
use zako_shared::FastMap;

use crate::{
    RequestPriority, ResourceDescriptor, ResourcePolicy, ResourcePoolError,
    allocation::{ResourceGrant, ResourceRequest},
    resource_key::ResourceKey,
    shares::ResourceUnitShares,
};

#[derive(Debug, Clone)]
struct GrantedAllocation {
    allocation_id: u64,
    grant: ResourceGrant,
}

#[derive(Debug)]
struct PendingRequest {
    request: ResourceRequest,
    priority: RequestPriority,
    sequence: u64,
    sender: oneshot::Sender<Result<GrantedAllocation, ResourcePoolError>>,
}

#[derive(Debug)]
pub struct ResourceHolder {
    allocation_id: u64,
    span: tracing::Span,
    grant: ResourceGrant,
    pool: Arc<Mutex<ResourcePoolInner>>,
}

impl ResourceHolder {
    fn new(
        span: tracing::Span,
        allocation: GrantedAllocation,
        pool: Arc<Mutex<ResourcePoolInner>>,
    ) -> Box<dyn crate::ResourceHolder> {
        Box::new(Self {
            allocation_id: allocation.allocation_id,
            span,
            grant: allocation.grant,
            pool,
        })
    }
}

impl crate::ResourceHolder for ResourceHolder {
    fn allocation_id(&self) -> u64 {
        self.allocation_id
    }

    fn grant(&self) -> &ResourceGrant {
        &self.grant
    }
    fn span(&self) -> &tracing::Span {
        &self.span
    }
}

impl Drop for ResourceHolder {
    fn drop(&mut self) {
        if let Err(error) = ResourcePool::release_allocation(&self.pool, self.allocation_id) {
            tracing::error!(
                ?error,
                allocation_id = self.allocation_id,
                "failed to release allocation on holder drop"
            );
        }
    }
}

#[derive(Debug)]
struct ResourcePoolInner {
    descriptors: FastMap<ResourceKey, ResourceDescriptor>,
    used: FastMap<ResourceKey, ResourceUnitShares>,
    active_allocations: FastMap<u64, ResourceGrant>,
    waiters: Vec<PendingRequest>,
    next_allocation_id: u64,
    next_waiter_sequence: u64,
}

/// A resource pool implementation with deterministic wait-queue scheduling.
#[derive(Debug)]
pub struct ResourcePool {
    inner: Arc<Mutex<ResourcePoolInner>>,
}

impl ResourcePool {
    /// Creates a pool from an iterator of descriptors.
    pub fn new(
        descriptors: impl IntoIterator<Item = ResourceDescriptor>,
    ) -> Result<Self, ResourcePoolError> {
        let mut descriptor_map = FastMap::default();
        let mut used = FastMap::default();

        for descriptor in descriptors {
            let key = descriptor.key.clone();
            if descriptor.granularity.as_shares() == 0 {
                return Err(ResourcePoolError::InvalidGranularity { key });
            }
            if descriptor_map.insert(key.clone(), descriptor).is_some() {
                return Err(ResourcePoolError::DuplicateDescriptor(key));
            }
            used.insert(key, Self::zero_shares());
        }

        Ok(Self {
            inner: Arc::new(Mutex::new(ResourcePoolInner {
                descriptors: descriptor_map,
                used,
                active_allocations: FastMap::default(),
                waiters: Vec::new(),
                next_allocation_id: 1,
                next_waiter_sequence: 0,
            })),
        })
    }

    fn zero_shares() -> ResourceUnitShares {
        ResourceUnitShares::from_shares(0)
    }

    fn effective_limit(
        descriptor: &ResourceDescriptor,
    ) -> Result<Option<ResourceUnitShares>, ResourcePoolError> {
        match descriptor.total {
            None => Ok(None),
            Some(total) => match descriptor.policy {
                ResourcePolicy::Hard => Ok(Some(total)),
                ResourcePolicy::Soft { max_overcommit } => total
                    .checked_add(max_overcommit)
                    .ok_or(ResourcePoolError::ArithmeticOverflow)
                    .map(Some),
            },
        }
    }

    fn validate_requested_amount(
        key: &ResourceKey,
        requested: ResourceUnitShares,
        granularity: ResourceUnitShares,
    ) -> Result<(), ResourcePoolError> {
        if requested < granularity {
            return Err(ResourcePoolError::BelowGranularity {
                key: key.clone(),
                requested,
                granularity,
            });
        }
        if requested.as_shares() % granularity.as_shares() != 0 {
            return Err(ResourcePoolError::NotDivisibleByGranularity {
                key: key.clone(),
                requested,
                granularity,
            });
        }
        Ok(())
    }

    fn validate_request(
        inner: &ResourcePoolInner,
        request: &ResourceRequest,
    ) -> Result<(), ResourcePoolError> {
        for (key, range) in &request.items {
            let descriptor = inner
                .descriptors
                .get(key)
                .ok_or_else(|| ResourcePoolError::UnknownResourceKey(key.clone()))?;

            if range.min > range.max {
                return Err(ResourcePoolError::InvalidRange {
                    key: key.clone(),
                    min: range.min,
                    max: range.max,
                });
            }

            Self::validate_requested_amount(key, range.min, descriptor.granularity)?;
            Self::validate_requested_amount(key, range.max, descriptor.granularity)?;

            if let Some(limit) = Self::effective_limit(descriptor)? {
                if range.min > limit {
                    return match descriptor.policy {
                        ResourcePolicy::Hard => Err(ResourcePoolError::ExceedsHardLimit {
                            key: key.clone(),
                            requested: range.min,
                            limit,
                        }),
                        ResourcePolicy::Soft { .. } => {
                            Err(ResourcePoolError::ExceedsSoftLimit {
                                key: key.clone(),
                                requested: range.min,
                                limit,
                            })
                        }
                    };
                }
            }
        }
        Ok(())
    }

    fn compute_grant(
        inner: &ResourcePoolInner,
        request: &ResourceRequest,
    ) -> Result<Option<ResourceGrant>, ResourcePoolError> {
        let mut items = FastMap::default();
        for (key, range) in &request.items {
            let descriptor = inner
                .descriptors
                .get(key)
                .ok_or_else(|| ResourcePoolError::UnknownResourceKey(key.clone()))?;
            let used = inner
                .used
                .get(key)
                .copied()
                .unwrap_or_else(Self::zero_shares);

            let granted = match Self::effective_limit(descriptor)? {
                None => range.max,
                Some(limit) => {
                    let available = limit.checked_sub(used).ok_or(ResourcePoolError::InconsistentState(
                        "used resource exceeds descriptor effective limit",
                    ))?;
                    if available < range.min {
                        return Ok(None);
                    }
                    if available < range.max {
                        available
                    } else {
                        range.max
                    }
                }
            };

            items.insert(key.clone(), granted);
        }
        Ok(Some(ResourceGrant {
            items,
            priority: request.priority,
        }))
    }

    fn reserve_grant(
        inner: &mut ResourcePoolInner,
        grant: &ResourceGrant,
    ) -> Result<u64, ResourcePoolError> {
        let allocation_id = inner.next_allocation_id;
        let next_allocation_id = allocation_id
            .checked_add(1)
            .ok_or(ResourcePoolError::ArithmeticOverflow)?;

        let mut updates = Vec::with_capacity(grant.items.len());
        for (key, amount) in &grant.items {
            let descriptor = inner
                .descriptors
                .get(key)
                .ok_or_else(|| ResourcePoolError::UnknownResourceKey(key.clone()))?;
            let used = inner
                .used
                .get(key)
                .copied()
                .unwrap_or_else(Self::zero_shares);
            let new_used = used
                .checked_add(*amount)
                .ok_or(ResourcePoolError::ArithmeticOverflow)?;

            if let Some(limit) = Self::effective_limit(descriptor)? {
                if new_used > limit {
                    return Err(ResourcePoolError::InconsistentState(
                        "granted allocation exceeds descriptor limit",
                    ));
                }
            }

            updates.push((key.clone(), new_used));
        }

        for (key, new_used) in updates {
            inner.used.insert(key, new_used);
        }
        inner.next_allocation_id = next_allocation_id;
        inner.active_allocations.insert(allocation_id, grant.clone());
        Ok(allocation_id)
    }

    fn allocate_immediately(
        inner: &mut ResourcePoolInner,
        request: &ResourceRequest,
    ) -> Result<Option<GrantedAllocation>, ResourcePoolError> {
        Self::validate_request(inner, request)?;
        let Some(grant) = Self::compute_grant(inner, request)? else {
            return Ok(None);
        };
        let allocation_id = Self::reserve_grant(inner, &grant)?;
        Ok(Some(GrantedAllocation {
            allocation_id,
            grant,
        }))
    }

    fn enqueue_waiter(
        inner: &mut ResourcePoolInner,
        request: &ResourceRequest,
    ) -> Result<oneshot::Receiver<Result<GrantedAllocation, ResourcePoolError>>, ResourcePoolError>
    {
        let sequence = inner.next_waiter_sequence;
        inner.next_waiter_sequence = sequence
            .checked_add(1)
            .ok_or(ResourcePoolError::ArithmeticOverflow)?;

        let (sender, receiver) = oneshot::channel();
        inner.waiters.push(PendingRequest {
            request: request.clone(),
            priority: request.priority,
            sequence,
            sender,
        });
        Ok(receiver)
    }

    fn release_allocation_locked(
        inner: &mut ResourcePoolInner,
        allocation_id: u64,
        run_scheduler: bool,
    ) -> Result<bool, ResourcePoolError> {
        let Some(grant) = inner.active_allocations.remove(&allocation_id) else {
            return Ok(false);
        };

        for (key, amount) in grant.items {
            let used = inner
                .used
                .get(&key)
                .copied()
                .unwrap_or_else(Self::zero_shares);
            let new_used = used.checked_sub(amount).ok_or(ResourcePoolError::InconsistentState(
                "released amount exceeds currently used amount",
            ))?;
            inner.used.insert(key, new_used);
        }

        if run_scheduler {
            Self::schedule_waiters(inner)?;
        }

        Ok(true)
    }

    fn schedule_waiters(inner: &mut ResourcePoolInner) -> Result<(), ResourcePoolError> {
        loop {
            inner.waiters.retain(|waiter| !waiter.sender.is_closed());

            let mut selected: Option<(usize, RequestPriority, u64, ResourceGrant)> = None;
            for (index, waiter) in inner.waiters.iter().enumerate() {
                let Some(grant) = Self::compute_grant(inner, &waiter.request)? else {
                    continue;
                };

                let replace = match selected {
                    None => true,
                    Some((_, best_priority, best_sequence, _)) => {
                        (waiter.priority, waiter.sequence) < (best_priority, best_sequence)
                    }
                };

                if replace {
                    selected = Some((index, waiter.priority, waiter.sequence, grant));
                }
            }

            let Some((selected_index, _, _, grant)) = selected else {
                break;
            };

            let waiter = inner.waiters.swap_remove(selected_index);
            let allocation_id = Self::reserve_grant(inner, &grant)?;
            let granted = GrantedAllocation {
                allocation_id,
                grant,
            };

            if waiter.sender.send(Ok(granted)).is_err() {
                let _ = Self::release_allocation_locked(inner, allocation_id, false)?;
            }
        }
        Ok(())
    }

    fn release_allocation(
        inner: &Arc<Mutex<ResourcePoolInner>>,
        allocation_id: u64,
    ) -> Result<(), ResourcePoolError> {
        let mut locked = inner.lock();
        let _ = Self::release_allocation_locked(&mut locked, allocation_id, true)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::ResourcePool for ResourcePool {
    async fn allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Box<dyn crate::ResourceHolder>, ResourcePoolError> {
        let span = tracing::Span::current();
        let wait_receiver = {
            let mut inner = self.inner.lock();
            if let Some(allocation) = Self::allocate_immediately(&mut inner, request)? {
                return Ok(ResourceHolder::new(span, allocation, Arc::clone(&self.inner)));
            }
            Self::enqueue_waiter(&mut inner, request)?
        };

        match wait_receiver.await {
            Ok(Ok(allocation)) => Ok(ResourceHolder::new(span, allocation, Arc::clone(&self.inner))),
            Ok(Err(error)) => Err(error),
            Err(_) => Err(ResourcePoolError::AllocationChannelClosed),
        }
    }

    fn try_allocate(
        &self,
        request: &ResourceRequest,
    ) -> Result<Option<Box<dyn crate::ResourceHolder>>, ResourcePoolError> {
        let span = tracing::Span::current();
        let mut inner = self.inner.lock();
        let maybe_allocation = Self::allocate_immediately(&mut inner, request)?;
        Ok(maybe_allocation.map(|allocation| {
            ResourceHolder::new(span.clone(), allocation, Arc::clone(&self.inner))
        }))
    }

    fn deallocate(&self, holder: &dyn crate::ResourceHolder) {
        let allocation_id = holder.allocation_id();
        if let Err(error) = Self::release_allocation(&self.inner, allocation_id) {
            tracing::error!(
                ?error,
                allocation_id,
                "failed to deallocate resource holder"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use tokio::sync::{mpsc, oneshot};
    use tokio::time::timeout;

    use super::*;
    use crate::{ResourceRange, ResourcePool as ResourcePoolTrait};

    fn shares(value: u64) -> ResourceUnitShares {
        ResourceUnitShares::from_shares(value)
    }

    fn hard_descriptor(key: ResourceKey, total: u64, granularity: u64) -> ResourceDescriptor {
        ResourceDescriptor::new(
            key,
            Some(shares(total)),
            shares(granularity),
            ResourcePolicy::Hard,
        )
    }

    fn soft_descriptor(
        key: ResourceKey,
        total: u64,
        granularity: u64,
        overcommit: u64,
    ) -> ResourceDescriptor {
        ResourceDescriptor::new(
            key,
            Some(shares(total)),
            shares(granularity),
            ResourcePolicy::Soft {
                max_overcommit: shares(overcommit),
            },
        )
    }

    fn exact_request(key: ResourceKey, amount: u64, priority: RequestPriority) -> ResourceRequest {
        let mut items = FastMap::default();
        items.insert(
            key,
            ResourceRange {
                min: shares(amount),
                max: shares(amount),
            },
        );
        ResourceRequest { items, priority }
    }

    fn range_request(
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

    async fn wait_for_queue_len(pool: &ResourcePool, expected: usize) {
        let result = timeout(Duration::from_secs(1), async {
            loop {
                if pool.inner.lock().waiters.len() == expected {
                    break;
                }
                tokio::task::yield_now().await;
            }
        })
        .await;
        assert!(result.is_ok());
    }

    #[test]
    fn new_rejects_duplicate_descriptor_keys() {
        let result = ResourcePool::new([
            hard_descriptor(ResourceKey::ThreadCount, 8, 1),
            hard_descriptor(ResourceKey::ThreadCount, 16, 1),
        ]);
        assert!(matches!(
            result,
            Err(ResourcePoolError::DuplicateDescriptor(
                ResourceKey::ThreadCount
            ))
        ));
    }

    #[test]
    fn new_rejects_zero_granularity() {
        let result = ResourcePool::new([ResourceDescriptor::new(
            ResourceKey::ThreadCount,
            Some(shares(8)),
            shares(0),
            ResourcePolicy::Hard,
        )]);
        assert!(matches!(
            result,
            Err(ResourcePoolError::InvalidGranularity {
                key: ResourceKey::ThreadCount
            })
        ));
    }

    #[test]
    fn request_unknown_key_returns_error() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 8, 1)])
            .unwrap();
        let request = exact_request(ResourceKey::MemoryCapacity, 1, RequestPriority::NORMAL);
        let result = ResourcePoolTrait::try_allocate(&pool, &request);
        assert!(matches!(
            result,
            Err(ResourcePoolError::UnknownResourceKey(
                ResourceKey::MemoryCapacity
            ))
        ));
    }

    #[test]
    fn invalid_range_returns_error() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 8, 1)])
            .unwrap();
        let mut items = FastMap::default();
        items.insert(
            ResourceKey::ThreadCount,
            ResourceRange {
                min: shares(4),
                max: shares(2),
            },
        );
        let request = ResourceRequest {
            items,
            priority: RequestPriority::NORMAL,
        };
        let result = ResourcePoolTrait::try_allocate(&pool, &request);
        assert!(matches!(
            result,
            Err(ResourcePoolError::InvalidRange {
                key: ResourceKey::ThreadCount,
                ..
            })
        ));
    }

    #[test]
    fn below_granularity_returns_error() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 16, 4)])
            .unwrap();
        let request = exact_request(ResourceKey::ThreadCount, 2, RequestPriority::NORMAL);
        let result = ResourcePoolTrait::try_allocate(&pool, &request);
        assert!(matches!(
            result,
            Err(ResourcePoolError::BelowGranularity {
                key: ResourceKey::ThreadCount,
                ..
            })
        ));
    }

    #[test]
    fn not_divisible_by_granularity_returns_error() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 16, 4)])
            .unwrap();
        let request = exact_request(ResourceKey::ThreadCount, 6, RequestPriority::NORMAL);
        let result = ResourcePoolTrait::try_allocate(&pool, &request);
        assert!(matches!(
            result,
            Err(ResourcePoolError::NotDivisibleByGranularity {
                key: ResourceKey::ThreadCount,
                ..
            })
        ));
    }

    #[test]
    fn try_allocate_success_and_drop_restores_capacity() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 8, 1)])
            .unwrap();
        let request = exact_request(ResourceKey::ThreadCount, 5, RequestPriority::NORMAL);

        let holder = ResourcePoolTrait::try_allocate(&pool, &request)
            .unwrap()
            .unwrap();
        assert_eq!(
            pool.inner.lock().used[&ResourceKey::ThreadCount],
            shares(5)
        );

        drop(holder);
        assert_eq!(
            pool.inner.lock().used[&ResourceKey::ThreadCount],
            shares(0)
        );
    }

    #[test]
    fn try_allocate_returns_none_when_insufficient() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 4, 1)])
            .unwrap();
        let full = exact_request(ResourceKey::ThreadCount, 4, RequestPriority::NORMAL);
        let one = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::NORMAL);

        let _holder = ResourcePoolTrait::try_allocate(&pool, &full).unwrap().unwrap();
        let result = ResourcePoolTrait::try_allocate(&pool, &one).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn hard_limit_blocks_impossible_request() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 10, 1)])
            .unwrap();
        let request = exact_request(ResourceKey::ThreadCount, 11, RequestPriority::NORMAL);
        let result = ResourcePoolTrait::try_allocate(&pool, &request);
        assert!(matches!(
            result,
            Err(ResourcePoolError::ExceedsHardLimit {
                key: ResourceKey::ThreadCount,
                ..
            })
        ));
    }

    #[test]
    fn soft_limit_allows_overcommit_and_blocks_beyond_limit() {
        let pool = ResourcePool::new([soft_descriptor(ResourceKey::ThreadCount, 10, 1, 5)])
            .unwrap();

        let twelve = exact_request(ResourceKey::ThreadCount, 12, RequestPriority::NORMAL);
        let four = exact_request(ResourceKey::ThreadCount, 4, RequestPriority::NORMAL);
        let sixteen = exact_request(ResourceKey::ThreadCount, 16, RequestPriority::NORMAL);

        let first = ResourcePoolTrait::try_allocate(&pool, &twelve)
            .unwrap()
            .unwrap();
        assert_eq!(
            first.grant().items[&ResourceKey::ThreadCount],
            shares(12)
        );

        assert!(ResourcePoolTrait::try_allocate(&pool, &four).unwrap().is_none());

        let limit_error = ResourcePoolTrait::try_allocate(&pool, &sixteen);
        assert!(matches!(
            limit_error,
            Err(ResourcePoolError::ExceedsSoftLimit {
                key: ResourceKey::ThreadCount,
                ..
            })
        ));
    }

    #[test]
    fn range_request_uses_greedy_max_when_available() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 10, 1)])
            .unwrap();
        let request = range_request(ResourceKey::ThreadCount, 2, 8, RequestPriority::NORMAL);
        let holder = ResourcePoolTrait::try_allocate(&pool, &request)
            .unwrap()
            .unwrap();
        assert_eq!(
            holder.grant().items[&ResourceKey::ThreadCount],
            shares(8)
        );
    }

    #[test]
    fn range_request_clamps_when_max_is_not_available() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 10, 1)])
            .unwrap();
        let preempt = exact_request(ResourceKey::ThreadCount, 6, RequestPriority::NORMAL);
        let _preempt_holder = ResourcePoolTrait::try_allocate(&pool, &preempt)
            .unwrap()
            .unwrap();

        let request = range_request(ResourceKey::ThreadCount, 2, 8, RequestPriority::NORMAL);
        let holder = ResourcePoolTrait::try_allocate(&pool, &request)
            .unwrap()
            .unwrap();
        assert_eq!(
            holder.grant().items[&ResourceKey::ThreadCount],
            shares(4)
        );
    }

    #[tokio::test]
    async fn allocate_waits_until_resources_are_released() {
        let pool = Arc::new(
            ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 1, 1)]).unwrap(),
        );
        let request = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::NORMAL);
        let blocking = ResourcePoolTrait::try_allocate(pool.as_ref(), &request)
            .unwrap()
            .unwrap();

        let pool_for_task = Arc::clone(&pool);
        let request_for_task = request.clone();
        let waiter = tokio::spawn(async move {
            ResourcePoolTrait::allocate(pool_for_task.as_ref(), &request_for_task).await
        });

        wait_for_queue_len(pool.as_ref(), 1).await;
        drop(blocking);

        let acquired = timeout(Duration::from_secs(1), waiter)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        drop(acquired);
    }

    #[tokio::test]
    async fn queue_scheduling_is_priority_then_fifo() {
        let pool = Arc::new(
            ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 1, 1)]).unwrap(),
        );
        let base_request = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::NORMAL);
        let blocker = ResourcePoolTrait::try_allocate(pool.as_ref(), &base_request)
            .unwrap()
            .unwrap();

        let (order_tx, mut order_rx) = mpsc::unbounded_channel::<&'static str>();

        let spawn_waiter = |pool: Arc<ResourcePool>,
                            request: ResourceRequest,
                            label: &'static str,
                            order_tx: mpsc::UnboundedSender<&'static str>| {
            let (release_tx, release_rx) = oneshot::channel::<()>();
            let handle = tokio::spawn(async move {
                let holder = ResourcePoolTrait::allocate(pool.as_ref(), &request)
                    .await
                    .unwrap();
                let _ = order_tx.send(label);
                let _ = release_rx.await;
                drop(holder);
            });
            (release_tx, handle)
        };

        let low_request = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::LOW);
        let high_first = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::HIGH);
        let high_second = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::HIGH);

        let (low_release, low_handle) = spawn_waiter(
            Arc::clone(&pool),
            low_request,
            "low",
            order_tx.clone(),
        );
        wait_for_queue_len(pool.as_ref(), 1).await;

        let (high_first_release, high_first_handle) = spawn_waiter(
            Arc::clone(&pool),
            high_first,
            "high_first",
            order_tx.clone(),
        );
        wait_for_queue_len(pool.as_ref(), 2).await;

        let (high_second_release, high_second_handle) = spawn_waiter(
            Arc::clone(&pool),
            high_second,
            "high_second",
            order_tx,
        );
        wait_for_queue_len(pool.as_ref(), 3).await;

        drop(blocker);

        let first = timeout(Duration::from_secs(1), order_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(first, "high_first");
        let _ = high_first_release.send(());

        let second = timeout(Duration::from_secs(1), order_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(second, "high_second");
        let _ = high_second_release.send(());

        let third = timeout(Duration::from_secs(1), order_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(third, "low");
        let _ = low_release.send(());

        low_handle.await.unwrap();
        high_first_handle.await.unwrap();
        high_second_handle.await.unwrap();
    }

    #[tokio::test]
    async fn canceled_waiter_does_not_leak_reservation() {
        let pool = Arc::new(
            ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 1, 1)]).unwrap(),
        );
        let request = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::NORMAL);
        let blocker = ResourcePoolTrait::try_allocate(pool.as_ref(), &request)
            .unwrap()
            .unwrap();

        let pool_for_waiter = Arc::clone(&pool);
        let request_for_waiter = request.clone();
        let waiter = tokio::spawn(async move {
            let _ = ResourcePoolTrait::allocate(pool_for_waiter.as_ref(), &request_for_waiter).await;
        });

        wait_for_queue_len(pool.as_ref(), 1).await;
        waiter.abort();
        let _ = waiter.await;

        drop(blocker);
        wait_for_queue_len(pool.as_ref(), 0).await;

        let acquired = ResourcePoolTrait::try_allocate(pool.as_ref(), &request)
            .unwrap()
            .unwrap();
        drop(acquired);
    }

    #[test]
    fn explicit_deallocate_and_drop_are_idempotent() {
        let pool = ResourcePool::new([hard_descriptor(ResourceKey::ThreadCount, 1, 1)])
            .unwrap();
        let request = exact_request(ResourceKey::ThreadCount, 1, RequestPriority::NORMAL);

        let holder = ResourcePoolTrait::try_allocate(&pool, &request)
            .unwrap()
            .unwrap();
        ResourcePoolTrait::deallocate(&pool, holder.as_ref());

        let second = ResourcePoolTrait::try_allocate(&pool, &request)
            .unwrap()
            .unwrap();
        drop(holder);

        let third = ResourcePoolTrait::try_allocate(&pool, &request).unwrap();
        assert!(third.is_none());

        drop(second);
    }
}
