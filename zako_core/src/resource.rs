use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use ahash::AHashMap;
use dashmap::DashMap;
use parking_lot::Mutex;
use smallvec::SmallVec;
use smol_str::SmolStr;
use tokio::sync::oneshot::{Sender, error::RecvError};

use crate::intern::InternedString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ResourceType {
    DiskIO,
    Memory,
    Processor,
    Network,
    GPU,
    Other(SmolStr),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResourceRequest {
    pub requested: im::HashMap<ResourceType, u64>,
}

impl ResourceRequest {
    pub fn cpu(count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Processor, count);
        Self { requested }
    }
    pub fn fs_io(count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::DiskIO, count);
        Self { requested }
    }
    pub fn gpu(count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::GPU, count);
        Self { requested }
    }
    pub fn memory(count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Memory, count);
        Self { requested }
    }
    pub fn network(count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Network, count);
        Self { requested }
    }
    pub fn other(name: SmolStr, count: u64) -> Self {
        let mut requested = im::HashMap::new();
        requested.insert(ResourceType::Other(name), count);
        Self { requested }
    }
    pub fn request_cpu(self, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Processor, count);
        Self { requested }
    }
    pub fn request_fs_io(self, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::DiskIO, count);
        Self { requested }
    }
    pub fn request_gpu(self, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::GPU, count);
        Self { requested }
    }
    pub fn request_memory(self, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Memory, count);
        Self { requested }
    }
    pub fn request_network(self, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Network, count);
        Self { requested }
    }
    pub fn request_other(self, name: SmolStr, count: u64) -> Self {
        let mut requested = self.requested;
        requested.insert(ResourceType::Other(name), count);
        Self { requested }
    }
}

#[derive(Debug)]
pub struct RawResourcePool {
    all_resources: AHashMap<ResourceType, u64>,
    rest_resources: AHashMap<ResourceType, u64>,
    requested: VecDeque<(ResourceRequest, Sender<()>)>,
}

/// TODO: Implement priority based resource allocation
///
/// TODO: Implement better task scheduling
#[derive(Debug)]
pub struct ResourcePool(Mutex<RawResourcePool>);

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

    pub fn notify(locked: &mut RawResourcePool) {
        // Implement notification logic here
        loop {
            if let Some(request) = locked.requested.pop_front() {
                if Self::try_occupy_inner(locked, &request.0) {
                    request.1.send(());
                } else {
                    locked.requested.push_front(request);
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Make sure that we can process a request at least once.
    fn make_sure_capacity(locked: &mut RawResourcePool, request: &ResourceRequest) {
        for (resource_type, count) in request.requested.iter() {
            let rest = locked
                .all_resources
                .entry(resource_type.clone())
                .or_insert(1);

            if *rest < *count {
                *rest = *count;
            }
        }
    }

    fn can_occupy(locked: &mut RawResourcePool, request: &ResourceRequest) -> bool {
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
}

pub struct ResourceGuard<'a> {
    occupied: ResourceRequest,
    from_pool: &'a ResourcePool,
}

impl<'a> Drop for ResourceGuard<'a> {
    fn drop(&mut self) {
        self.from_pool.release_inner(&self.occupied);
    }
}
