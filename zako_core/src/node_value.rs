use bitcode::{Decode, Encode};
use hone::node::{NodeValue, Persistent};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use zako_digest::hash::XXHash3;

use crate::{
    blob_handle::BlobHandle,
    context::BuildContext,
    file_artifact::{FileArtifact, RawFileArtifact},
    path::{NeutralPath, interned::InternedNeutralPath},
};

#[derive(Debug, Clone, IntoStaticStr)]
pub enum ZakoValue {
    Glob(Vec<InternedNeutralPath>),
    ResolvedProject(BuildContext),
    FileResult(FileArtifact),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr, Decode, Encode)]
pub enum RawZakoValue {
    GlobResult(Vec<String>),
    ResolvedProjectResult,
    FileResult(RawFileArtifact),
}

impl Persistent<BuildContext> for ZakoValue {
    type Persisted = RawZakoValue;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(match self {
            ZakoValue::Glob(paths) => Self::Persisted::GlobResult(
                paths
                    .iter()
                    .map(|p| ctx.interner().resolve(&p.interned()).to_string())
                    .collect(),
            ),
            ZakoValue::ResolvedProject(_) => return None,
            ZakoValue::FileResult(file) => Self::Persisted::FileResult(file.to_persisted(ctx)?),
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(match p {
            RawZakoValue::GlobResult(paths) => ZakoValue::Glob(
                paths
                    .into_iter()
                    .map(|p| unsafe {
                        InternedNeutralPath::from_raw(ctx.interner().get_or_intern(&p))
                    })
                    .collect(),
            ),
            RawZakoValue::ResolvedProjectResult => return None,
            RawZakoValue::FileResult(file) => {
                ZakoValue::FileResult(FileArtifact::from_persisted(file, ctx)?)
            }
        })
    }
}

impl XXHash3 for ZakoValue {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let name: &'static str = self.into();
        hasher.update(name.as_bytes());

        match self {
            ZakoValue::Glob(paths) => {
                for path in paths {
                    hasher.update(&path.interned().as_u64().to_le_bytes());
                }
            }
            ZakoValue::ResolvedProject(context) => {
                hasher.update(&context.project_root().as_u64().to_le_bytes());
            }
            ZakoValue::FileResult(file) => {
                file.hash_into(hasher);
            }
        }
    }
}

impl NodeValue<BuildContext> for ZakoValue {}
