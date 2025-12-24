use tokio::sync::oneshot;

#[derive(Debug)]
pub struct V8Request {
    pub specifier: String,
    // Engine 返回: Ok(源码) 或 Err(错误信息)
    pub resp: oneshot::Sender<Result<String, String>>,
}
