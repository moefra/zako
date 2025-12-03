use crate::configuration::{Configuration, ConfiguredId};
use crate::id::Id;

pub struct Tool {
    id: Id,
    default_configuration: Configuration,
    /// The script that configure the tool
    configure_script: Id,
}

pub struct RequiredTool {}

pub struct ConfiguredTool {
    id: ConfiguredId,
    /// The script that execute tool
    execute_script: Id,
}
