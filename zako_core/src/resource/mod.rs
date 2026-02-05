pub mod heuristics;

use std::collections::{HashMap, VecDeque};

use ahash::AHashMap;
use parking_lot::Mutex;
use smallvec::SmallVec;
use smol_str::SmolStr;
use sysinfo::System;
use tokio::sync::oneshot::{Sender, error::RecvError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    /// unit: IO thread
    DiskIO,
    /// unit: byte
    Memory,
    /// unit: thread
    Processor,
    /// unit: byte
    Network,
    /// unit: thread
    GPU,
    /// unit: determined by user
    Other(SmolStr),
}

pub type ResourceUnit = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceRange {
    pub minimum: ResourceUnit,
    pub maximum: ResourceUnit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRequest<U = ResourceUnit>
where
    U: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash,
{
    pub requested: im::HashMap<ResourceType, U>,
}

pub type ResourceRangeRequest = ResourceRequest<ResourceRange>;

impl<U> ResourceRequest<U>
where
    U: std::fmt::Debug + Clone + PartialEq + Eq + std::hash::Hash,
{
    pub fn cpu(count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Processor, count);
        Self { requested }
    }
    pub fn fs_io(count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::DiskIO, count);
        Self { requested }
    }
    pub fn gpu(count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::GPU, count);
        Self { requested }
    }
    pub fn memory(count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Memory, count);
        Self { requested }
    }
    pub fn network(count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Network, count);
        Self { requested }
    }
    pub fn other(name: SmolStr, count: U) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Other(name), count);
        Self { requested }
    }
    pub fn request_cpu(self, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Processor, count);
        Self { requested }
    }
    pub fn request_fs_io(self, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::DiskIO, count);
        Self { requested }
    }
    pub fn request_gpu(self, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::GPU, count);
        Self { requested }
    }
    pub fn request_memory(self, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Memory, count);
        Self { requested }
    }
    pub fn request_network(self, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Network, count);
        Self { requested }
    }
    pub fn request_other(self, name: SmolStr, count: U) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Other(name), count);
        Self { requested }
    }
}

#[derive(Debug)]
struct RawResourcePool {
    all_resources: AHashMap<ResourceType, ResourceUnit>,
    rest_resources: AHashMap<ResourceType, ResourceUnit>,
    requested: VecDeque<(ResourceRequest, Sender<()>)>,
}

// TODO: Implement priority based resource allocation
// Issue URL: https://github.com/moefra/zako/issues/6
//
// TODO: Implement better task scheduling
// Issue URL: https://github.com/moefra/zako/issues/5
#[derive(Debug)]
pub struct ResourcePool(Mutex<RawResourcePool>);

pub enum DetectableResourceValue {
    Value(ResourceUnit),
    Heuristics,
}

impl ResourcePool {
    pub fn new(
        cpu_capacity: u64,
        fs_io_capacity: u64,
        gpu_capacity: u64,
        memory_capacity: u64,
        network_capacity: u64,
        other_capacity: HashMap<SmolStr, u64>,
    ) -> Self {
        let mut all_resources = AHashMap::with_capacity(6);
        all_resources.extend(
            other_capacity
                .into_iter()
                .map(|(k, v)| (ResourceType::Other(k), v)),
        );
        all_resources.insert(ResourceType::Processor, cpu_capacity);
        all_resources.insert(ResourceType::DiskIO, fs_io_capacity);
        all_resources.insert(ResourceType::GPU, gpu_capacity);
        all_resources.insert(ResourceType::Memory, memory_capacity);
        all_resources.insert(ResourceType::Network, network_capacity);
        let requested = VecDeque::new();
        let pool = RawResourcePool {
            rest_resources: all_resources.clone(),
            all_resources,
            requested,
        };
        Self(Mutex::new(pool))
    }

    pub fn new_with_heuristics(
        system: &System,
        cpu_capacity: DetectableResourceValue,
        fs_io_capacity: DetectableResourceValue,
        gpu_capacity: DetectableResourceValue,
        memory_capacity: DetectableResourceValue,
        network_capacity: DetectableResourceValue,
        other_capacity: HashMap<SmolStr, DetectableResourceValue>,
    ) -> Self {
        let cpu = match cpu_capacity {
            DetectableResourceValue::Value(v) => v,
            DetectableResourceValue::Heuristics => heuristics::determine_cpu_capacity(&system),
        };

        let fs_io = match fs_io_capacity {
            DetectableResourceValue::Value(v) => v,
            DetectableResourceValue::Heuristics => heuristics::determine_disk_io_capacity(&system),
        };

        let gpu = match gpu_capacity {
            DetectableResourceValue::Value(v) => v,
            DetectableResourceValue::Heuristics => heuristics::determine_gpu_capacity(&system),
        };

        let memory = match memory_capacity {
            DetectableResourceValue::Value(v) => v,
            DetectableResourceValue::Heuristics => heuristics::determine_memory_capacity(&system),
        };

        let network = match network_capacity {
            DetectableResourceValue::Value(v) => v,
            DetectableResourceValue::Heuristics => heuristics::determine_network_capacity(&system),
        };

        let other: HashMap<SmolStr, u64> = other_capacity
            .into_iter()
            .map(|(k, v)| {
                let value = match v {
                    DetectableResourceValue::Value(val) => val,
                    DetectableResourceValue::Heuristics => 1, // Default to 1 for unknown resources
                };
                (k, value)
            })
            .collect();

        Self::new(cpu, fs_io, gpu, memory, network, other)
    }

    pub fn notify(locked: &mut RawResourcePool) {
        // Implement notification logic here
        loop {
            if let Some(request) = locked.requested.pop_front() {
                if Self::try_occupy_inner(locked, &request.0) {
                    let _ = request.1.send(());
                } else {
                    locked.requested.push_front(request);
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn get_cpu_count(&self) -> u64 {
        self.0.lock().all_resources[&ResourceType::Processor]
    }

    /// Make sure that we can process a request at least once.
    fn make_sure_capacity(locked: &mut RawResourcePool, request: &ResourceRequest) {
        for (resource_type, count) in request.requested.iter() {
            let all = locked
                .all_resources
                .entry(resource_type.clone())
                .or_insert(0);

            let increase = u64::checked_sub(*count, *all).unwrap_or(0);

            if *all < *count {
                *all = *count;

                let rest = locked
                    .rest_resources
                    .entry(resource_type.clone())
                    .or_insert(0);
                *rest += increase;
            }
        }
    }

    fn _can_occupy(locked: &mut RawResourcePool, request: &ResourceRequest) -> bool {
        for (resource_type, count) in request.requested.iter() {
            let rest = locked.rest_resources.get(resource_type);
            if let Some(rest) = rest {
                if rest < count {
                    return false;
                }
            } else {
                unreachable!(
                    "ResourcePool::can_occupy can only be called after make_sure_capacity"
                );
            }
        }
        true
    }

    pub(crate) fn release_inner(&self, request: &ResourceRequest) {
        let mut locked = self.0.lock();

        for (resource_type, count) in request.requested.iter() {
            let rest = locked.rest_resources.get_mut(resource_type);

            if let Some(rest) = rest {
                *rest += count;
            } else {
                unreachable!("The resource should exists when releasing");
            }
        }

        Self::notify(&mut locked);
    }

    pub fn try_occupy_inner(locked: &mut RawResourcePool, request: &ResourceRequest) -> bool {
        let mut taken_type: SmallVec<[ResourceType; 4]> = SmallVec::new();
        let mut taken_count: SmallVec<[u64; 4]> = SmallVec::new();

        let mut taken = true;
        for (resource_type, count) in request.requested.iter() {
            let rest = locked.rest_resources.get_mut(resource_type);

            if let Some(rest) = rest {
                if *rest < *count {
                    taken = false;
                    break;
                }
                taken_type.push(resource_type.clone());
                taken_count.push(*count);
                *rest -= count;
            } else {
                unreachable!(
                    "ResourcePool::try_occupy_inner can only be called after make_sure_capacity"
                );
            }
        }

        if !taken {
            while !taken_count.is_empty() {
                let resource_type = taken_type.pop().unwrap();
                let count = taken_count.pop().unwrap();
                let rest = locked
                    .rest_resources
                    .get_mut(&resource_type)
                    .expect("Resource type not found,but it should actually be");
                *rest += count;
            }
        }

        return taken;
    }

    pub async fn occupy<'a>(
        &'a self,
        request: ResourceRequest,
    ) -> Result<ResourceGuard<'a>, RecvError> {
        let mut locked = self.0.lock();
        Self::make_sure_capacity(&mut locked, &request);

        if Self::try_occupy_inner(&mut locked, &request) {
            Self::notify(&mut locked);

            return Ok(ResourceGuard {
                occupied: request,
                from_pool: self,
            });
        }

        // enqueue
        let (sder, recv) = tokio::sync::oneshot::channel();
        locked.requested.push_back((request.clone(), sder));

        let _ = recv.await?;

        return Ok(ResourceGuard {
            occupied: request,
            from_pool: self,
        });
    }

    pub async fn occupy_range<'a>(
        &'a self,
        request: ResourceRangeRequest,
    ) -> Result<ResourceGuard<'a>, RecvError> {
        // Convert range request to a concrete request by trying to get the maximum possible
        // within the range, falling back to minimum if needed
        let mut locked = self.0.lock();

        // First, ensure we can handle at least the minimum for each resource
        let min_request = ResourceRequest {
            requested: request
                .requested
                .iter()
                .map(|(k, v)| (k.clone(), v.minimum))
                .collect(),
        };
        Self::make_sure_capacity(&mut locked, &min_request);

        // Try to allocate the best possible amount within the range
        let mut actual_request = im::HashMap::new();
        for (resource_type, range) in request.requested.iter() {
            let available = locked
                .rest_resources
                .get(resource_type)
                .copied()
                .unwrap_or(0);

            // Allocate as much as possible within the range
            let to_allocate = available.min(range.maximum).max(range.minimum);
            actual_request.insert(resource_type.clone(), to_allocate);
        }

        let concrete_request = ResourceRequest {
            requested: actual_request,
        };

        if Self::try_occupy_inner(&mut locked, &concrete_request) {
            Self::notify(&mut locked);
            return Ok(ResourceGuard {
                occupied: concrete_request,
                from_pool: self,
            });
        }

        // If we couldn't allocate even the minimum, wait for resources
        let (sder, recv) = tokio::sync::oneshot::channel();
        locked.requested.push_back((min_request.clone(), sder));
        drop(locked);

        recv.await?;

        // After being notified, try to get the best allocation again
        let mut locked = self.0.lock();
        let mut actual_request = im::HashMap::new();
        for (resource_type, range) in request.requested.iter() {
            let available = locked
                .rest_resources
                .get(resource_type)
                .copied()
                .unwrap_or(0);

            let to_allocate = available.min(range.maximum).max(range.minimum);
            actual_request.insert(resource_type.clone(), to_allocate);
        }

        let concrete_request = ResourceRequest {
            requested: actual_request,
        };

        // The minimum was already allocated by the notification, so we need to
        // release it and reallocate with the actual request
        for (resource_type, count) in min_request.requested.iter() {
            if let Some(rest) = locked.rest_resources.get_mut(resource_type) {
                *rest += count;
            }
        }

        Self::try_occupy_inner(&mut locked, &concrete_request);

        Ok(ResourceGuard {
            occupied: concrete_request,
            from_pool: self,
        })
    }
}

/// TODO: Make this to ResourceGuard<U = ResourceUnit, 'a> to support request resource range.
//Issue URL: https://github.com/moefra/zako/issues/31
/// Issue URL: https://github.com/moefra/zako/issues/30
pub struct ResourceGuard<'a> {
    occupied: ResourceRequest,
    from_pool: &'a ResourcePool,
}

impl<'a> Drop for ResourceGuard<'a> {
    fn drop(&mut self) {
        self.from_pool.release_inner(&self.occupied);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resource_pool_new() {
        let pool = ResourcePool::new(4, 8, 1, 1024, 16, HashMap::new());
        assert_eq!(pool.get_cpu_count(), 4);
    }

    #[test]
    fn test_resource_pool_new_with_other() {
        let mut other = HashMap::new();
        other.insert(SmolStr::new("custom"), 10);
        let pool = ResourcePool::new(4, 8, 1, 1024, 16, other);
        assert_eq!(pool.get_cpu_count(), 4);
    }

    #[test]
    fn test_resource_pool_new_with_heuristics() {
        let pool = ResourcePool::new_with_heuristics(
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Heuristics,
            HashMap::new(),
        );
        // CPU count should be at least 1
        assert!(pool.get_cpu_count() >= 1);
    }

    #[test]
    fn test_resource_pool_new_with_mixed_values() {
        let pool = ResourcePool::new_with_heuristics(
            DetectableResourceValue::Value(8),
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Value(2),
            DetectableResourceValue::Heuristics,
            DetectableResourceValue::Heuristics,
            HashMap::new(),
        );
        assert_eq!(pool.get_cpu_count(), 8);
    }

    #[test]
    fn test_resource_request_cpu() {
        let request = ResourceRequest::cpu(4);
        assert_eq!(request.requested.get(&ResourceType::Processor), Some(&4));
    }

    #[test]
    fn test_resource_request_chain() {
        let request = ResourceRequest::cpu(4)
            .request_memory(1024)
            .request_fs_io(2);
        assert_eq!(request.requested.get(&ResourceType::Processor), Some(&4));
        assert_eq!(request.requested.get(&ResourceType::Memory), Some(&1024));
        assert_eq!(request.requested.get(&ResourceType::DiskIO), Some(&2));
    }

    #[test]
    fn test_resource_range() {
        let range = ResourceRange {
            minimum: 1,
            maximum: 10,
        };
        assert_eq!(range.minimum, 1);
        assert_eq!(range.maximum, 10);
    }

    #[tokio::test]
    async fn test_occupy_and_release() {
        let pool = ResourcePool::new(4, 8, 1, 1024, 16, HashMap::new());

        let request = ResourceRequest::cpu(2);
        let guard = pool.occupy(request).await.unwrap();

        // Check that resources are occupied
        {
            let locked = pool.0.lock();
            assert_eq!(locked.rest_resources[&ResourceType::Processor], 2);
        }

        // Drop guard to release resources
        drop(guard);

        // Check that resources are released
        {
            let locked = pool.0.lock();
            assert_eq!(locked.rest_resources[&ResourceType::Processor], 4);
        }
    }

    #[tokio::test]
    async fn test_occupy_exceeds_capacity_expands() {
        let pool = ResourcePool::new(4, 8, 1, 1024, 16, HashMap::new());

        // Request more than initial capacity
        let request = ResourceRequest::cpu(8);
        let guard = pool.occupy(request).await.unwrap();

        // Capacity should have expanded
        {
            let locked = pool.0.lock();
            assert_eq!(locked.all_resources[&ResourceType::Processor], 8);
        }

        drop(guard);
    }

    #[tokio::test]
    async fn test_occupy_range_gets_maximum_when_available() {
        let pool = ResourcePool::new(10, 8, 1, 1024, 16, HashMap::new());

        let mut requested = im::HashMap::new();
        requested.insert(
            ResourceType::Processor,
            ResourceRange {
                minimum: 2,
                maximum: 8,
            },
        );
        let range_request = ResourceRangeRequest { requested };

        let guard = pool.occupy_range(range_request).await.unwrap();

        // Should have gotten maximum (8) since 10 are available
        assert_eq!(
            guard.occupied.requested.get(&ResourceType::Processor),
            Some(&8)
        );
    }

    #[tokio::test]
    async fn test_occupy_range_gets_available_when_less_than_max() {
        let pool = ResourcePool::new(5, 8, 1, 1024, 16, HashMap::new());

        let mut requested = im::HashMap::new();
        requested.insert(
            ResourceType::Processor,
            ResourceRange {
                minimum: 2,
                maximum: 10,
            },
        );
        let range_request = ResourceRangeRequest { requested };

        let guard = pool.occupy_range(range_request).await.unwrap();

        // Should have gotten available (5) since max (10) exceeds capacity
        assert_eq!(
            guard.occupied.requested.get(&ResourceType::Processor),
            Some(&5)
        );
    }

    #[tokio::test]
    async fn test_multiple_occupations() {
        let pool = ResourcePool::new(10, 8, 1, 1024, 16, HashMap::new());

        let request1 = ResourceRequest::cpu(3);
        let request2 = ResourceRequest::cpu(4);

        let guard1 = pool.occupy(request1).await.unwrap();
        let guard2 = pool.occupy(request2).await.unwrap();

        {
            let locked = pool.0.lock();
            assert_eq!(locked.rest_resources[&ResourceType::Processor], 3); // 10 - 3 - 4
        }

        drop(guard1);

        {
            let locked = pool.0.lock();
            assert_eq!(locked.rest_resources[&ResourceType::Processor], 6); // 10 - 4
        }

        drop(guard2);

        {
            let locked = pool.0.lock();
            assert_eq!(locked.rest_resources[&ResourceType::Processor], 10);
        }
    }

    #[test]
    fn test_resource_type_other() {
        let resource_type = ResourceType::Other(SmolStr::new("custom_resource"));
        assert_eq!(
            resource_type,
            ResourceType::Other(SmolStr::new("custom_resource"))
        );
    }

    #[test]
    fn test_resource_request_other() {
        let request = ResourceRequest::other(SmolStr::new("custom"), 5);
        assert_eq!(
            request
                .requested
                .get(&ResourceType::Other(SmolStr::new("custom"))),
            Some(&5)
        );
    }
}
