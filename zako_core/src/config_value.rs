use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use ts_rs::TS;
use zako_digest::blake3_hash::Blake3Hash;

use crate::{id::Label, intern::Interner};

#[derive(
    TS,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "config_value.d.ts")]
#[ts(optional_fields)]
pub struct ConfigValue {
    pub r#type: ConfigType,
    pub default: ConfigDefault,
}

impl Blake3Hash for ConfigValue {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.r#type.hash_into_blake3(hasher);
        self.default.hash_into_blake3(hasher);
    }
}

#[derive(
    TS,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "config_default.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
pub enum ConfigDefault {
    String(String),
    Boolean(bool),
    Number(i64),
    Object(ConfigOperation),
}

impl Blake3Hash for ConfigDefault {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        match self {
            ConfigDefault::String(s) => s.hash_into_blake3(hasher),
            ConfigDefault::Boolean(b) => b.hash_into_blake3(hasher),
            ConfigDefault::Number(n) => n.hash_into_blake3(hasher),
            ConfigDefault::Object(o) => o.hash_into_blake3(hasher),
        }
    }
}

#[derive(
    TS,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "config_operation.d.ts")]
#[ts(optional_fields)]
pub struct ConfigOperation {
    #[ts(type = "`${string}:${string}`")]
    pub inherit: String,
    pub action: Option<String>,
}

impl Blake3Hash for ConfigOperation {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.inherit.hash_into_blake3(hasher);
        self.action.hash_into_blake3(hasher);
    }
}

#[derive(
    TS,
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    rkyv::Deserialize,
    rkyv::Serialize,
    rkyv::Archive,
)]
#[ts(export, export_to = "config_type.d.ts")]
#[ts(rename_all = "lowercase")]
pub enum ConfigType {
    Label,
    Boolean,
    Number,
    String,
}

impl Blake3Hash for ConfigType {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        hasher.update(match self {
            ConfigType::Label => b"label",
            ConfigType::Boolean => b"boolean",
            ConfigType::Number => b"number",
            ConfigType::String => b"string",
        });
    }
}

#[derive(Debug, Clone, PartialEq, Eq, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum ResolvedConfigValue {
    Label(Label),
    String(SmolStr),
    Boolean(bool),
    Number(i64),
}

impl ResolvedConfigValue {
    pub fn resolve(&self, interner: &Interner) -> Result<ConfigValue, crate::config::ConfigError> {
        match self {
            ResolvedConfigValue::String(string) => Ok(ConfigValue {
                r#type: ConfigType::String,
                default: ConfigDefault::String(string.to_string()),
            }),
            ResolvedConfigValue::Boolean(boolean) => Ok(ConfigValue {
                r#type: ConfigType::Boolean,
                default: ConfigDefault::Boolean(*boolean),
            }),
            ResolvedConfigValue::Number(number) => Ok(ConfigValue {
                r#type: ConfigType::Number,
                default: ConfigDefault::Number(*number),
            }),
            ResolvedConfigValue::Label(label) => Ok(ConfigValue {
                r#type: ConfigType::Label,
                default: ConfigDefault::Object(ConfigOperation {
                    inherit: label.resolved(interner)?,
                    action: None,
                }),
            }),
        }
    }
}
