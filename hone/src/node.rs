use std::fmt::Debug;
use std::hash::Hash;
use zako_digest::blake3_hash::Blake3Hash;

use rkyv::ser::Serializer;
use rkyv::{Archive, Archived, Deserialize, Serialize};

pub trait Persistent<C>: Sized {
    /// 这里的 Persisted 是实现了 rkyv Archive 的中间结构体
    /// 它通常会派生 #[derive(Archive, Serialize, Deserialize)]
    type Persisted: Archive;

    /// 将运行时对象转换为可序列化的中间结构
    fn to_persisted(&self, ctx: &C) -> Option<Self::Persisted>;

    /// 从【归档引用】还原回运行时对象
    ///
    /// 注意：这里接收的是 &Archived<Self::Persisted>，这是零拷贝的核心！
    fn from_archived(p: &Archived<Self::Persisted>, ctx: &C) -> Option<Self>;
}

pub trait NodeKey<C>: Clone + Debug + Eq + Hash + Send + Sync + 'static + Persistent<C> {}

pub trait NodeValue<C>: Debug + Send + Sync + 'static + Persistent<C> {}
