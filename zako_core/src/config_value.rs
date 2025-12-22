use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{id::Label, intern::InternedString};

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_value.d.ts")]
#[ts(optional_fields)]
pub struct ConfigValue {
    pub r#type: ConfigType,
    pub default: Option<ConfigDefault>,
}

impl Blake3Hash for ConfigValue {
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        self.r#type.hash_into_blake3(hasher);
        self.default.hash_into_blake3(hasher);
    }
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_default.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
pub enum ConfigDefault {
    Label(String),
    String { string: String },
    Boolean(bool),
    Number(i64),
    Object(ConfigOperation),
}

impl Blake3Hash for ConfigDefault {
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        match self {
            ConfigDefault::Label(s) => s.hash_into_blake3(hasher),
            ConfigDefault::Boolean(b) => b.hash_into_blake3(hasher),
            ConfigDefault::Number(n) => n.hash_into(hasher),
            ConfigDefault::Object(o) => o.hash_into_blake3(hasher),
            ConfigDefault::String { string } => {
                hasher.update(b"::"); // why `::`? because it invalid in Label, so it can separate the two
                string.hash_into_blake3(hasher);
            }
        }
    }
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_operation.d.ts")]
#[ts(optional_fields)]
pub struct ConfigOperation {
    pub inherit: String,
    pub action: Option<String>,
}

impl Blake3Hash for ConfigOperation {
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        self.inherit.hash_into_blake3(hasher);
        self.action.hash_into_blake3(hasher);
    }
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_type.d.ts")]
#[ts(optional_fields)]
pub enum ConfigType {
    Boolean,
    Number,
    String,
}

impl Blake3Hash for ConfigType {
    fn hash_into_blake3(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(match self {
            ConfigType::Boolean => b"boolean",
            ConfigType::Number => b"number",
            ConfigType::String => b"string",
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedConfigValue {
    Label(Label),
    String(SmolStr),
    Boolean(bool),
    Number(i64),
}
