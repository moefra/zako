use rkyv::bytecheck::CheckBytes;
use rkyv::util::AlignedVec;
use std::fmt::Debug;
use std::hash::Hash as StdHash;

use rkyv::{Archive, Archived, Serialize};

pub trait Persistent:
    Archive
    + for<'a> Serialize<
        rkyv::rancor::Strategy<
            rkyv::ser::Serializer<
                rkyv::ser::writer::IoWriter<AlignedVec>,
                rkyv::ser::allocator::ArenaHandle<'a>,
                rkyv::ser::sharing::Share,
            >,
            rkyv::rancor::Error,
        >,
    >
{
}

impl<T> Persistent for T where
    T: Archive
        + for<'a> Serialize<
            rkyv::rancor::Strategy<
                rkyv::ser::Serializer<
                    rkyv::ser::writer::IoWriter<AlignedVec>,
                    rkyv::ser::allocator::ArenaHandle<'a>,
                    rkyv::ser::sharing::Share,
                >,
                rkyv::rancor::Error,
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
