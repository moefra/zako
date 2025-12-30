use tokio::sync::oneshot;

/// The request to import a file into the V8 engine.
///
/// The script must be transpiled.
#[derive(Debug)]
pub struct V8ImportRequest {
    pub specifier: String,
    pub resp: oneshot::Sender<Result<String, eyre::Report>>,
}
