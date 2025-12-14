use bitcode::{Decode, Encode};
use hone::node::{NodeValue, Persistent};
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;
use zako_digest::hash::XXHash3;

use crate::{
    context::BuildContext,
    path::{NeutralPath, interned::InternedNeutralPath},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum ZakoValue {
    GlobResult(Vec<InternedNeutralPath>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, IntoStaticStr, Decode, Encode)]
pub enum RawZakoValue {
    GlobResult(Vec<String>),
}

impl Persistent<BuildContext> for ZakoValue {
    type Persisted = RawZakoValue;

    fn to_persisted(&self, ctx: &BuildContext) -> Self::Persisted {
        match self {
            ZakoValue::GlobResult(paths) => Self::Persisted::GlobResult(
                paths
                    .iter()
                    .map(|p| ctx.interner().resolve(&p.interned()).to_string())
                    .collect(),
            ),
        }
    }

    fn from_persisted(p: Self::Persisted, ctx: &BuildContext) -> Self {
        match p {
            RawZakoValue::GlobResult(paths) => ZakoValue::GlobResult(
                paths
                    .into_iter()
                    .map(|p| unsafe {
                        InternedNeutralPath::from_raw(ctx.interner().get_or_intern(&p))
                    })
                    .collect(),
            ),
        }
    }
}

impl XXHash3 for ZakoValue {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let name: &'static str = self.into();
        hasher.update(name.as_bytes());

        match self {
            ZakoValue::GlobResult(paths) => {
                for path in paths {
                    hasher.update(&path.interned().as_u64().to_le_bytes());
                }
            }
        }
    }
}

impl NodeValue<BuildContext> for ZakoValue {}
