use std::{
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use thiserror::Error;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Error)]
/// The reasons from users.
pub enum UserCancelReason {
    #[error("SIGINT detected")]
    SIGINT,
    #[error("Other cancel reason from user: {0:?}")]
    Other(Arc<eyre::Report>),
}

/// The reasons why the operation is cancelled.
#[derive(Debug, Clone, Error)]
pub enum CancelReason {
    #[error("The interrupt from user: {0:?}")]
    UserInterrupt(UserCancelReason),
    #[error("Sibling Failed. Fail fast to prevent resource leak.")]
    SiblingFailed,
    #[error("Timeout. Waited for `{0:?}`")]
    Timeout(Duration),
    #[error("Invalidated. Some make the operation invalidated.")]
    Invalidated,
    #[error("Can not get or can not retain got resource")]
    Starvation,
    #[error(transparent)]
    Other(Arc<eyre::Report>),
}

/// 包装 Token 和 Reason
#[derive(Clone, Debug)]
struct SharedState {
    token: CancellationToken,
    /// 取消的原因，只记录第一个原因，后续的原因被忽略
    reason: Arc<OnceLock<CancelReason>>,
}

impl SharedState {
    #[inline]
    pub(crate) fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            reason: Arc::new(OnceLock::new()),
        }
    }

    /// 触发取消
    #[inline]
    pub(crate) fn cancel(&self, reason: CancelReason) {
        let _ = self.reason.get_or_init(|| reason);
        self.token.cancel();
    }

    /// 检查是否取消
    #[inline]
    pub(crate) fn is_cancelled(&self) -> bool {
        self.token.is_cancelled()
    }

    /// 获取原因（如果有的话）
    #[inline]
    pub(crate) fn reason(&self) -> Option<CancelReason> {
        self.reason.get().cloned()
    }

    /// 暴露原始 Token
    #[inline]
    pub(crate) fn token(&self) -> &CancellationToken {
        &self.token
    }

    /// 创建子 Scope (级联取消)
    #[inline]
    pub(crate) fn child(&self) -> Self {
        // 这里逻辑稍微复杂一点：
        // 子 Scope 应该拥有自己的 Token (linked to parent)
        // 但 Reason 应该指向同一个源吗？通常是的。
        Self {
            token: self.token.child_token(),
            reason: self.reason.clone(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CancelSource {
    state: Arc<SharedState>,
}

impl CancelSource {
    pub fn new() -> Self {
        Self {
            state: Arc::new(SharedState::new()),
        }
    }

    /// 触发取消 (Write)
    pub fn cancel(&self, reason: CancelReason) {
        self.state.cancel(reason);
    }

    /// 获取只读 Token (用于分发给下级)
    pub fn token(&self) -> CancelToken {
        CancelToken {
            state: self.state.clone(),
        }
    }

    /// 创建子 Source (级联控制)
    pub fn create_child(&self) -> CancelSource {
        CancelSource {
            state: Arc::new(self.state.child()),
        }
    }
}

// --- 3. 接收端 (C# 的 CancellationToken) ---
// 只有它能被传递给底层函数
#[derive(Clone, Debug)]
pub struct CancelToken {
    state: Arc<SharedState>,
}

impl CancelToken {
    /// 检查是否取消 (Read)
    pub fn is_cancelled(&self) -> bool {
        self.state.is_cancelled()
    }

    /// 等待取消信号 (Async Wait)
    pub async fn cancelled(&self) {
        self.state.token().cancelled().await;
    }

    /// 获取原因
    pub fn reason(&self) -> Option<CancelReason> {
        self.state.reason().clone()
    }

    /// 允许从 Token 创建更下一级的 Token (链式传播)
    /// 注意：这里创建的是 Token，不是 Source。
    /// 这意味着持有 Token 的人无法创建出一个能“反向取消父级”的 Source。
    pub fn child_token(&self) -> CancelToken {
        CancelToken {
            state: Arc::new(self.state.child()),
        }
    }
}
