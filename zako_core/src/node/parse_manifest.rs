use crate::{blob_handle::BlobHandle, package::Package};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ParseManifest {
    pub blob_handle: BlobHandle,
}

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ParseManifestResult {
    pub project: Package,
}
