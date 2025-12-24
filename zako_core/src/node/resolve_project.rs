use hone::node::Persistent;
use std::path::PathBuf;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{context::BuildContext, intern::InternedAbsolutePath, package::InternedPackageId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub struct ResolveProject {
    pub package: InternedPackageId,
}
