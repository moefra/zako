use std::str::FromStr;
use std::sync::LazyLock;
use crate::api::id::{Id, ToolName, ToolType};

pub static COMPILER: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.c.tool_type.compiler").unwrap())
});

pub static PREPROCESSOR: LazyLock<ToolType> = LazyLock::new(|| {
    ToolType(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.c.tool_type.preprocessor").unwrap())
});

pub static GCC: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.c.tool_name.gcc").unwrap())
});

pub static CLANG: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.c.tool_name.clang").unwrap())
});

pub static MSVC: LazyLock<ToolName> = LazyLock::new(|| {
    ToolName(Id::from_str("moe.kawayi:zmake@1.0.0#builtin.c.tool_name.preprocessor").unwrap())
});
