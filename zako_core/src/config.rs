use std::collections::HashMap;

use bitcode::{Decode, Encode};
use zako_digest::hash::XXHash3;

use crate::{
    config_value::ConfigValue,
    intern::{InternedString, Interner},
};

/// Raw, mutable configuration.
#[derive(Debug, Clone, PartialEq, Eq, Decode, Encode)]
pub struct Configuration {
    pub config: HashMap<String, ConfigValue>,
}

impl Configuration {
    pub fn new() -> Self {
        Self {
            config: HashMap::new(),
        }
    }

    pub fn build(self, interner: &Interner) -> InternedConfiguration {
        let mut config = self.config.into_iter().collect::<Vec<_>>();

        config.sort_by_key(|(k, _)| k.clone());

        let config = config
            .into_iter()
            .map(|(k, v)| (interner.get_or_intern(k), v))
            .collect();

        InternedConfiguration { config }
    }
}

impl XXHash3 for Configuration {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        for (key, value) in self.config.iter() {
            key.hash_into(hasher);
            value.hash_into(hasher);
        }
    }
}

/// Interned, immutable configuration.
///
/// It is used to store the configuration in the build graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InternedConfiguration {
    pub config: Vec<(InternedString, ConfigValue)>,
    // TODO: Use index to get the value by key
    // pub index: HashMap<InternedString, usize> ?
}

impl InternedConfiguration {
    pub fn resolve(&self, interner: &Interner) -> Configuration {
        let mut config = HashMap::new();
        for (key, value) in self.config.iter() {
            config.insert(interner.resolve(key).to_string(), value.clone());
        }
        Configuration { config }
    }
}
