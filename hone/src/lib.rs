use std::sync::Arc;

pub mod context;
pub mod dependency;
pub mod engine;
pub mod error;
pub mod node;
pub mod status;

use zako_digest::hash::XXHash3;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct KeyId(u64);

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastMap<K, V> = ::dashmap::DashMap<K, V, ::ahash::RandomState>;

/// The result of iteration of this map is not ordered.
///
/// Please do not rely on any specific order.
pub type FastSet<K> = ::dashmap::DashSet<K, ::ahash::RandomState>;

pub type HoneResult<T> = Result<T, error::HoneError>;

pub type SharedHoneResult<T> = Result<T, Arc<error::HoneError>>;
