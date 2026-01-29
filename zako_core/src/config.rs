use ::std::collections::BTreeMap;
use std::collections::HashMap;

use eyre::Context;
use smol_str::SmolStr;
use zako_digest::blake3::Blake3Hash;

use crate::{
    config_value::{ConfigDefault, ConfigValue, ResolvedConfigValue},
    id::Label,
    intern::Interner,
};

/// Raw, immutable configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub config: BTreeMap<SmolStr, ConfigValue>,
}

#[cfg(not(feature = "v8snapshot"))]
impl Default for Configuration {
    fn default() -> Self {
        Self::new()
    }
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            config: Default::default(),
        }
    }

    pub fn generate_template_code(self) -> String {
        let mut code = String::new();

        for (key, value) in self.config.iter() {
            code.push_str(&format!("{} = {}\n", key, format!("{:?}", value.default)));
        }

        code
    }

    pub fn resolve(self, interner: &Interner) -> Result<ResolvedConfiguration, eyre::Report> {
        let mut configs = self.config.into_iter().collect::<Vec<_>>();

        configs.sort_by_key(|(k, _)| k.clone());

        let mut built_config = Vec::new();

        for config in configs {
            let label = Label::try_parse(config.0.as_str(), interner)
                .wrap_err_with(|| format!("failed to resolve config key: {}", config.0))?;

            match config.1.default {
                ConfigDefault::String(string) => {
                    built_config.push((label, ResolvedConfigValue::String(string.into())));
                }
                ConfigDefault::Boolean(b) => {
                    built_config.push((label, ResolvedConfigValue::Boolean(b)));
                }
                ConfigDefault::Number(n) => {
                    built_config.push((label, ResolvedConfigValue::Number(n)));
                }
                ConfigDefault::Object(_) => {
                    todo!();
                }
            }
        }

        Ok(ResolvedConfiguration {
            config: built_config.into_iter().collect(),
        })
    }
}

impl Blake3Hash for Configuration {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.config.hash_into_blake3(hasher);
    }
}

/// Interned, immutable configuration.
///
/// It is used to store the configuration in the build graph.
#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Serialize, rkyv::Deserialize, rkyv::Archive)]
pub struct ResolvedConfiguration {
    pub config: BTreeMap<Label, ResolvedConfigValue>,
    // TODO: Use index to get the value by key
    // Issue URL: https://github.com/moefra/zako/issues/18
    // pub index: HashMap<InternedString, usize> ?
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Interner error while processing configuration: {0}")]
    InternerError(#[from] ::zako_interner::InternerError),
    #[error("Id parse error: {0}")]
    IdParseError(#[from] crate::id::IdParseError),
    #[error("Other error: {0}")]
    Other(#[from] eyre::Report),
}

impl ResolvedConfiguration {
    pub fn empty() -> Self {
        Self {
            config: Default::default(),
        }
    }

    pub fn resolve(&self, interner: &Interner) -> Result<Configuration, ConfigError> {
        let mut config: BTreeMap<SmolStr, ConfigValue> = Default::default();
        for (key, value) in self.config.iter() {
            config.insert(
                SmolStr::new(key.resolved(interner)?),
                value.resolve(interner)?,
            );
        }
        Ok(Configuration { config })
    }
}
