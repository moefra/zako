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

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::std::collections::HashSet<K, ahash::RandomState>;

/// `Stable` means that the sequence of iteration is stable.
pub type StableMap<K, V> = ::std::collections::BTreeMap<K, V>;

/// `Stable` means that the sequence of iteration is stable.
pub type StableSet<K> = ::std::collections::BTreeSet<K>;

/// A fast cache implementation.
pub type AsyncCache<K, V> = ::moka::future::Cache<K, V, ::ahash::RandomState>;
