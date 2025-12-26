use crate::consts;
use crate::global_state::GlobalState;
use crate::worker::WorkerBehavior;
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tracing::instrument;
use zako_cancel::CancelToken;
use zako_digest::blake3_hash::Blake3Hash;

#[derive(Debug, Error)]
pub enum TransformerError {
    #[error("Failed to parse es target `{0}`: {1}")]
    ESTargetError(String, String),
    #[error("Failed to parse source code:\n{0:?}")]
    ParseError(Vec<String>),
}

/// Input for Oxc Transpiler Worker
#[derive(Debug, Clone)]
pub struct OxcTranspilerInput {
    pub source_text: String,
    pub source_name: String,
    pub source_type: SourceType,
}

/// Output from Oxc Transpiler Worker
#[derive(Debug, Clone)]
pub struct OxcTranspilerOutput {
    pub code: String,
    pub map: Option<String>,
}

impl Blake3Hash for OxcTranspilerOutput {
    fn hash_into_blake3(&self, hasher: &mut blake3::Hasher) {
        self.code.hash_into_blake3(hasher);
        self.map.hash_into_blake3(hasher);
    }
}

/// A worker that transpiles code using Oxc
#[derive(Debug, Clone)]
pub struct OxcTranspilerWorker;

/// State for Oxc Worker, including a cache for transpilation results
pub struct OxcState {
    allocator: Allocator,
}

impl WorkerBehavior for OxcTranspilerWorker {
    type Context = GlobalState;
    type Input = OxcTranspilerInput;
    type Output = Result<OxcTranspilerOutput, TransformerError>;
    type State = OxcState;

    fn init(_: &Arc<Self::Context>) -> Self::State {
        OxcState {
            allocator: Allocator::with_capacity(128 * 1024), // 128kb for each buffer
        }
    }

    #[instrument(skip(state, _cancel_token))]
    fn process(
        state: &mut Self::State,
        input: Self::Input,
        _cancel_token: CancelToken,
    ) -> Self::Output {
        let allocator = &mut state.allocator; // 128kb for each buffer
        allocator.reset(); // in case last run return error and do not reset at last

        let output = {
            let OxcTranspilerInput {
                source_name,
                source_text,
                source_type,
            } = input;

            let path = Path::new(&source_name);

            let ret = Parser::new(&allocator, &source_text, source_type).parse();

            if ret.panicked {
                let mut err_str: Vec<String> = Vec::with_capacity(ret.errors.len());
                for error in ret.errors {
                    err_str.push(format!(
                        "{:?}",
                        error.with_source_code(source_text.to_string())
                    ));
                }
                return Err(TransformerError::ParseError(err_str));
            }

            let mut program = ret.program;

            let options =
                TransformOptions::from_target(consts::TRANSPILE_TARGET).map_err(|err| {
                    TransformerError::ESTargetError(
                        consts::TRANSPILE_TARGET.to_string(),
                        err.to_string(),
                    )
                })?;

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
                        error.with_source_code(source_text.to_string())
                    ));
                }
                return Err(TransformerError::ParseError(err_str));
            }

            let generated = Codegen::new().build(&program);

            let output = OxcTranspilerOutput {
                code: generated.code,
                map: generated.map.map(|s| s.to_json_string()),
            };

            output
        };

        allocator.reset();

        return Ok(output);
    }

    fn gc(state: &mut Self::State) {
        state.allocator = Allocator::with_capacity(128 * 1024);
    }
}
