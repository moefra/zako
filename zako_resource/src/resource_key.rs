#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKey {
    ThreadCount,
    MemoryCapacity,
    Other(zako_id::UniqueId),
}
