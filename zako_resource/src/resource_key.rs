//! Canonical keys used by the resource pool.

/// Logical resource kind identifiers.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResourceKey {
    /// CPU thread count capacity.
    ThreadCount,
    /// Memory capacity (bytes).
    MemoryCapacity,
    /// User-defined resource key.
    Other(zako_id::UniqueId),
}
