use ::smol_str::SmolStr;

use crate::config::ResolvedConfiguration;

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum Target {
    Target(SmolStr),
    Configuration(ResolvedConfiguration),
}
