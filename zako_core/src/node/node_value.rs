use bitcode::{Decode, Encode};
use hone::node::{NodeValue, Persistent};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use zako_digest::hash::XXHash3;

use crate::node::file::RawFileResult;
use crate::node::glob::{Glob, GlobResult, RawGlobResult};
use crate::node::resolve_project::{RawResolveProject, ResolveProject};
use crate::node::transpile_ts::TranspileTsResult;
use crate::{
    blob_handle::BlobHandle,
    context::BuildContext,
    node::file::FileResult,
    node::transpile_ts::RawTranspileTsResult,
    path::{NeutralPath, interned::InternedNeutralPath},
};

#[derive(Debug, Clone, IntoStaticStr)]
pub enum ZakoValue {
    Glob(GlobResult),
    ResolveProject(ResolveProject),
    FileResult(FileResult),
    TranspileTsResult(TranspileTsResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr, Decode, Encode)]
pub enum RawZakoValue {
    GlobResult(RawGlobResult),
    ResolvedProjectResult(RawResolveProject),
    FileResult(RawFileResult),
    TranspileTsResult(RawTranspileTsResult),
}

impl Persistent<BuildContext> for ZakoValue {
    type Persisted = RawZakoValue;

    fn to_persisted(&self, ctx: &BuildContext) -> Option<Self::Persisted> {
        Some(match self {
            ZakoValue::Glob(glob) => Self::Persisted::GlobResult(glob.to_persisted(ctx)?),
            ZakoValue::ResolveProject(project) => {
                Self::Persisted::ResolvedProjectResult(project.to_persisted(ctx)?)
            }
            ZakoValue::FileResult(file) => Self::Persisted::FileResult(file.to_persisted(ctx)?),
            ZakoValue::TranspileTsResult(result) => {
                Self::Persisted::TranspileTsResult(RawTranspileTsResult {
                    code: result.code.clone(),
                    source_map: result.source_map.clone(),
                })
            }
        })
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Option<Self> {
        Some(match p {
            RawZakoValue::GlobResult(glob) => {
                ZakoValue::Glob(GlobResult::from_persisted(glob, ctx)?)
            }
            RawZakoValue::ResolvedProjectResult(project) => {
                ZakoValue::ResolveProject(ResolveProject::from_persisted(project, ctx)?)
            }
            RawZakoValue::FileResult(file) => {
                ZakoValue::FileResult(FileResult::from_persisted(file, ctx)?)
            }
            RawZakoValue::TranspileTsResult(result) => {
                ZakoValue::TranspileTsResult(TranspileTsResult {
                    code: result.code,
                    source_map: result.source_map,
                })
            }
        })
    }
}

impl NodeValue<BuildContext> for ZakoValue {}
