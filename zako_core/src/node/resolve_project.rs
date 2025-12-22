use std::path::PathBuf;

use bitcode::{Decode, Encode};
use hone::node::Persistent;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{context::BuildContext, intern::InternedAbsolutePath, package::InternedPackageId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolveProject {
    pub package: InternedPackageId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode)]
pub struct RawResolveProject {
    pub package: String,
}

impl Persistent<BuildContext> for ResolveProject {
    type Persisted = RawResolveProject;

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(Self {
            package: InternedPackageId::try_parse(p.package.as_str(), ctx.interner()).ok()?,
        })
    }

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(Self::Persisted {
            package: self.package.resolved(ctx.interner()),
        })
    }
}
