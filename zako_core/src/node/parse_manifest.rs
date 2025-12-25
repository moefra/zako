use camino::Utf8PathBuf;
use hone::node::Persistent;
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    blob_handle::BlobHandle,
    context::BuildContext,
    intern::InternedAbsolutePath,
    path::interned::InternedNeutralPath,
    pattern::{InternedPattern, Pattern},
    project::Project,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ParseManifest {
    pub path: Utf8PathBuf,
    pub manifest_file_name: Option<SmolStr>,
}

#[derive(Debug, Clone, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ParseManifestResult {
    pub manifest: Project,
}
