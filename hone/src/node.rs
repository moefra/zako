use blake3::Hash;
use rkyv::api::high::HighSerializer;
use rkyv::bytecheck::CheckBytes;
use rkyv::util::AlignedVec;
use std::fmt::Debug;
use std::hash::Hash as StdHash;
use zako_digest::blake3_hash::Blake3Hash;

use rkyv::ser::Serializer;
use rkyv::{Archive, Archived, Deserialize, Serialize};

pub trait Persistent:
    Archive
    + Serialize<
        rkyv::api::high::HighSerializer<
            rkyv::ser::writer::IoWriter<AlignedVec>,
            rkyv::ser::allocator::Arena,
            rkyv::ser::sharing::Share,
        >,
    >
{
}

pub trait NodeKey: Clone + Debug + Eq + StdHash + Send + Sync + 'static + Persistent {}

pub trait SafeNodeKey: NodeKey {}

impl<T> SafeNodeKey for T
where
    T: NodeKey,
    // safety comes first
    for<'a> Archived<Self>: CheckBytes<rkyv::validation::archive::ArchiveValidator<'a>>,
{
}

pub trait NodeValue: Debug + Send + Sync + 'static + Persistent {}

pub trait SafeNodeValue: NodeValue {}

impl<T> SafeNodeValue for T
where
    T: NodeValue,
    // safety comes first
    for<'a> Archived<Self>: CheckBytes<rkyv::validation::archive::ArchiveValidator<'a>>,
{
}
