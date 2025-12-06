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

use oxc::allocator::Allocator;
use oxc::codegen::Codegen;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc::transformer::{TransformOptions, Transformer};

pub struct Transpiled{
    pub code:String,
    pub source_map:Option<String>
}

pub fn transform_typescript(source_code: &str, source_name: &str) -> Result<Transpiled, String> {
    let allocator = Allocator::new();

    let path = Path::new(source_name);

    let options = TransformOptions::from_target("es2023")?;

    let ret = Parser::new(&allocator, &source_code, SourceType::ts()).parse();

    if !ret.errors.is_empty() {
        let mut err_str = String::new();
        for error in ret.errors {
            let error = error.with_source_code(source_code.to_string());
            err_str.push_str(&format!("{}\n", &error));
        }
        return Err(err_str);
    }

    let transformer = Transformer::new(&allocator, &path, &options);

    let mut program = ret.program;

    let ret = SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);

    let ret = transformer.build_with_scoping(ret.semantic.into_scoping(), &mut program);

    if !ret.errors.is_empty() {
        let mut err_str = String::new();
        for error in ret.errors {
            let error = error.with_source_code(source_code.to_string());
            err_str.push_str(&format!("{}\n", &error));
        }
        return Err(err_str);
    }

    let generated = Codegen::new().build(&program);

    return Ok(Transpiled{
        code: generated.code,
        source_map: generated.map.map(|s| s.to_json_string()),
    });
}
