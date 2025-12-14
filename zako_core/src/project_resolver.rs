use crate::context::{BuildContext, ContextHandler};
use crate::engine::{Engine, EngineError};
use crate::package_source::PackageSource;
use crate::path::NeutralPath;
use crate::project::Project;
use crate::project_resolver::ProjectResolveError::{CircularDependency, FileNotExists, NotAFile};
use crate::sandbox::SandboxError;
use crate::v8error::V8Error;
use crate::zako_module_loader::{ModuleLoadError, ModuleSpecifier, ModuleType};
use ahash::AHashMap;
use hone::node::{NodeKey, Persistent};
use serde::de::{DeserializeOwned, DeserializeSeed};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::hash::Hash;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use thiserror::Error;
use tracing::{event, instrument};
use zako_digest::hash::XXHash3;

#[derive(Error, Debug)]
pub enum ProjectResolveError {
    #[error("the script file `{0}` does not exists")]
    FileNotExists(PathBuf),
    #[error("the path to project file `{0}` is not a file")]
    NotAFile(PathBuf),
    #[error("detect circular dependency when resolving project {0}")]
    CircularDependency(PathBuf),
    #[error("get an io error")]
    IOError(#[from] io::Error),
    #[error("get an sandbox error")]
    SandboxError(#[from] SandboxError),
    #[error("get an engine error")]
    EngineError(#[from] EngineError),
    #[error("get an module load error")]
    ModuleLoadError(#[from] ModuleLoadError),
    #[error("get an v8 engine error: {0:?}")]
    V8Error(V8Error),
    #[error("get an serde_v8 error")]
    V8SerdeError(#[from] deno_core::serde_v8::Error),
    #[error("get an error when try to prase the id of the package:{0}")]
    ParseError(String),
    #[error("try to get parsed project `{0}` but not found in ProjectResolver.parsed")]
    NoExpectedProjectFound(PathBuf),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageSourceKey {
    source: InternedPackageSource,
    context: ContextHandler,
}

impl Hash for PackageSourceKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.source {
            InternedPackageSource::Registry { package } => {
                package.hash(state);
            }
            InternedPackageSource::Git { repo, checkout } => {
                repo.hash(state);
                checkout.hash(state);
            }
            InternedPackageSource::Http { url } => {
                url.hash(state);
            }
            InternedPackageSource::Path { path } => {
                path.hash(state);
            }
        }
    }
}

impl XXHash3 for PackageSourceKey {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        let _none: () = match &self.source {
            InternedPackageSource::Registry { package } => {
                self.context.interner().resolve(package).hash_into(hasher)
            }
            InternedPackageSource::Git { repo, checkout } => {
                hasher.update(self.context.interner().resolve(repo).as_bytes());
                if let Some(checkout) = checkout {
                    hasher.update(self.context.interner().resolve(checkout).as_bytes());
                }
            }
            InternedPackageSource::Http { url } => {
                self.context.interner().resolve(url).hash_into(hasher)
            }
            InternedPackageSource::Path { path } => {
                self.context.interner().resolve(path).hash_into(hasher)
            }
        };
    }
}

impl Serialize for PackageSourceKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw = self.source.to_raw(self.context.interner());
        raw.serialize(serializer)
    }
}
/// 用于反序列化的 Seed
pub struct DeserializeWithContext {
    pub ctx: ContextHandler,
}

impl<'de> DeserializeSeed<'de> for PackageSourceKey {
    type Value = PackageSourceKey;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = PackageSource::deserialize(deserializer)?;
        Ok(PackageSourceKey {
            source: InternedPackageSource::from_raw(&raw, self.context.clone().interner_mut()),
            context: self.context.clone(),
        })
    }
}

impl Persistent<SharedBuildContext> for PackageSourceKey {
    type Persisted = PackageSource;

    fn to_persisted(&self, ctx: &SharedBuildContext) -> Self::Persisted {
        self.source.to_raw(ctx.interner())
    }

    fn from_persisted(p: Self::Persisted, ctx: &SharedBuildContext) -> Self {
        PackageSourceKey {
            source: InternedPackageSource::from_raw(&p, ctx.interner_mut()),
            context: ctx.get_handle(),
        }
    }
}

impl NodeKey<BuildContext> for PackageSourceKey {}
