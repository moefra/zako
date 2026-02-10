/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type ConcurrentMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type ConcurrentSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastMap<K, V> = ::std::collections::HashMap<K, V, ahash::RandomState>;

/// Anti-hash DoS attack resistant map.
pub type SafeMap<K, V> = ::std::collections::HashMap<K, V, std::collections::hash_map::RandomState>;

/// Anti-hash DoS attack resistant map.
pub type SafeSet<V> = ::std::collections::HashSet<V, std::collections::hash_map::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::std::collections::HashSet<K, ahash::RandomState>;

/// Iteration order is deterministic and sorted by key.
pub type StableMap<K, V> = ::std::collections::BTreeMap<K, V>;

/// Iteration order is deterministic and sorted by key.
pub type StableSet<K> = ::std::collections::BTreeSet<K>;

/// A fast cache implementation.
pub type FastAsyncCache<K, V> = ::moka::future::Cache<K, V, ::ahash::RandomState>;
