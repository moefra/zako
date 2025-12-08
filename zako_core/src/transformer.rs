//! # Transformer Example
//!
//! This example demonstrates code transformation using the Oxc transformer.
//! It supports various transformation options including Babel compatibility
//! and environment-specific transforms.
//!
//! ## Usage
//!
//! Create a `test.js` file and run:
//! ```bash
//! cargo run -p oxc_transformer --example transformer [filename] [options]
//! ```
//!
//! ## Options
//!
//! - `--babel-options <path>`: Path to Babel options file
//! - `--targets <targets>`: Browser/environment targets
//! - `--target <target>`: Single target environment

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransformerError {
    #[error("Failed to parse es target `{0}`: {1}")]
    ESTargetError(String, String),
    #[error("Failed to parse source code:\n{0:?}")]
    ParseError(Vec<String>),
}

pub struct Transpiled {
    pub code: String,
    pub source_map: Option<String>,
}

pub fn transform_typescript(
    source_code: &str,
    source_name: &str,
) -> Result<Transpiled, TransformerError> {
    let allocator = Allocator::with_capacity(128 * 1024); // 128kb for each buffer

    let path = Path::new(source_name);

    let ret = Parser::new(&allocator, &source_code, SourceType::ts()).parse();

    if ret.panicked {
        let mut err_str: Vec<String> = Vec::with_capacity(ret.errors.len());
        for error in ret.errors {
            err_str.push(format!(
                "{:?}",
                error.with_source_code(source_code.to_string())
            ));
        }
        return Err(TransformerError::ParseError(err_str));
    }

    let mut program = ret.program;

    // TODO: it seems es2026 means esnext. switch to es2025 once they release it.
    let options = TransformOptions::from_target("es2026")
        .map_err(|err| TransformerError::ESTargetError("es2026".to_string(), err.to_string()))?;

    let transformer = Transformer::new(&allocator, &path, &options);

    let ret = SemanticBuilder::new()
        .with_check_syntax_error(false) // un-standard syntax sucks but zako sucks too
        .build(&program);

    let ret = transformer.build_with_scoping(ret.semantic.into_scoping(), &mut program);

    if !ret.errors.is_empty() {
        let mut err_str: Vec<String> = Vec::with_capacity(ret.errors.len());
        for error in ret.errors {
            err_str.push(format!(
                "{:?}",
                error.with_source_code(source_code.to_string())
            ));
        }
        return Err(TransformerError::ParseError(err_str));
    }

    let generated = Codegen::new().build(&program);

    return Ok(Transpiled {
        code: generated.code,
        source_map: generated.map.map(|s| s.to_json_string()),
    });
}
