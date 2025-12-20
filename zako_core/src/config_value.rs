use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use zako_digest::hash::XXHash3;

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_value.d.ts")]
#[ts(optional_fields)]
pub struct ConfigValue {
    pub r#type: ConfigType,
    pub default: Option<ConfigDefault>,
}

impl XXHash3 for ConfigValue {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        self.r#type.hash_into(hasher);
        self.default.hash_into(hasher);
    }
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Decode, Encode)]
#[ts(export, export_to = "config_default.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
pub enum ConfigDefault {
    String(String),
    Boolean(bool),
    Number(i64),
    Object(ConfigOperation),
}

impl XXHash3 for ConfigDefault {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        match self {
            ConfigDefault::String(s) => s.hash_into(hasher),
            ConfigDefault::Boolean(b) => b.hash_into(hasher),
            ConfigDefault::Number(n) => n.hash_into(hasher),
            ConfigDefault::Object(o) => o.hash_into(hasher),
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

impl XXHash3 for ConfigOperation {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        self.inherit.hash_into(hasher);
        self.action.hash_into(hasher);
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

impl XXHash3 for ConfigType {
    fn hash_into(&self, hasher: &mut xxhash_rust::xxh3::Xxh3) {
        hasher.update(match self {
            ConfigType::Boolean => b"boolean",
            ConfigType::Number => b"number",
            ConfigType::String => b"string",
        });
    }
}
