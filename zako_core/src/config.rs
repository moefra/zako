use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export, export_to = "config.d.ts")]
#[ts(optional_fields)]
pub struct Config {
    pub r#type: ConfigType,
    pub default: Option<ConfigDefault>,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export, export_to = "config_default.d.ts")]
#[ts(optional_fields)]
#[serde(untagged)]
pub enum ConfigDefault {
    String(String),
    Boolean(bool),
    Number(i64),
    Object(ConfigOperation),
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export, export_to = "config_operation.d.ts")]
#[ts(optional_fields)]
pub struct ConfigOperation {
    pub inherit: String,
    pub action: Option<String>,
}

#[derive(TS, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[ts(export, export_to = "config_type.d.ts")]
#[ts(optional_fields)]
pub enum ConfigType {
    Boolean,
    Number,
    String,
}
