use std::path::PathBuf;

use bitcode::{Decode, Encode};
use hone::node::Persistent;
use zako_digest::hash::XXHash3;

use crate::{context::BuildContext, intern::InternedAbsolutePath, package::InternedPackageId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResolveProject {
    pub package: InternedPackageId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Decode, Encode)]
pub struct RawResolveProject {
    pub path: String,
}

impl Persistent<BuildContext> for ResolveProject {
    type Persisted = RawResolveProject;

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(ResolveProject {
            package: unsafe { InternedPackageId::try_parse(p.path, ctx.interner())? },
        })
    }

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(RawResolveProject {
            path: self.package.to_string(ctx.interner()),
        })
    }
}

impl XXHash3 for ResolveProject {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(&self.package.as_u64().to_le_bytes());
    }
}
