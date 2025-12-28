use tokio::sync::oneshot;

#[derive(Debug)]
pub struct V8TranspileRequest {
    pub specifier: String,
    pub resp: oneshot::Sender<Result<String, eyre::Report>>,
}
