use ::smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, rkyv::Deserialize, rkyv::Serialize, rkyv::Archive)]
pub enum Target {
    Target(SmolStr),
    Configuration(SmolStr),
}
