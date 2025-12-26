use std::collections::HashMap;

use eyre::{Context, Error};
use smol_str::SmolStr;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{
    config_value::{ConfigDefault, ConfigValue, ResolvedConfigValue},
    id::{IdParseError, Label},
    intern::{InternedString, Interner},
};

/// Raw, immutable configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Configuration {
    pub config: HashMap<SmolStr, ConfigValue, ahash::RandomState>,
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            config: HashMap::with_hasher(ahash::RandomState::new()),
        }
    }

    pub fn from(config: HashMap<SmolStr, ConfigValue, ahash::RandomState>) -> Self {
        Self { config }
    }

    pub fn resolve(self, interner: &Interner) -> Result<ResolvedConfiguration, eyre::Report> {
        let mut configs = self.config.into_iter().collect::<Vec<_>>();

        configs.sort_by_key(|(k, _)| k.clone());

        let mut built_config = Vec::new();

        for config in configs {
            let label = Label::try_parse(config.0.as_str(), interner)
                .wrap_err_with(|| format!("failed to resolve config key: {}", config.0))?;

            match config.1.default {
                Some(ConfigDefault::Label(s)) => {
                    let value_label =
                        Label::try_parse(s.as_str(), interner).wrap_err_with(|| {
                            format!("failed to resolve config value: {}", config.0)
                        })?;
                    built_config.push((label, ResolvedConfigValue::Label(value_label)));
                }
                Some(ConfigDefault::String { string }) => {
                    built_config.push((label, ResolvedConfigValue::String(string.into())));
                }
                Some(ConfigDefault::Boolean(b)) => {
                    built_config.push((label, ResolvedConfigValue::Boolean(b)));
                }
                Some(ConfigDefault::Number(n)) => {
                    built_config.push((label, ResolvedConfigValue::Number(n)));
                }
                Some(ConfigDefault::Object(_)) => {
                    todo!();
                }
                None => {
                    todo!("This branch means user must provide a default value");
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
#[derive(Debug, Clone, PartialEq, Eq, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct ResolvedConfiguration {
    pub config: Vec<(Label, ResolvedConfigValue)>,
    // TODO: Use index to get the value by key
    // Issue URL: https://github.com/moefra/zako/issues/18
    // pub index: HashMap<InternedString, usize> ?
}

impl ResolvedConfiguration {
    pub fn empty() -> Self {
        Self { config: Vec::new() }
    }

    pub fn resolve(&self, interner: &Interner) -> Configuration {
        let mut config = HashMap::with_hasher(ahash::RandomState::new());
        for (key, value) in self.config.iter() {
            config.insert(
                SmolStr::new(key.resolved(interner)),
                value.resolve(interner),
            );
        }
        Configuration { config }
    }
}
