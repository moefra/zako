use crate::configuration::{Configuration, ConfiguredId};
use crate::id::{Id, ResolvedId};

pub struct Tool {
    id: ResolvedId,
    default_configuration: Configuration,
    /// The script that configure the tool
    configure_script: ResolvedId,
}

pub struct RequiredTool {}

pub struct ConfiguredTool {
    id: ConfiguredId,
    /// The script that execute tool
    execute_script: ResolvedId,
}
