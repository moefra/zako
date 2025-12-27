pub mod oxc_worker;
pub mod protocol;
pub mod v8worker;
pub mod worker_pool;

use std::sync::Arc;

use zako_cancel::CancelToken;

/// A worker and its behavior definition.
pub trait WorkerBehavior: Send + Sync + 'static {
    /// Context provided when init
    type Context: Send + Sync + 'static;
    /// Input type for processing.
    type Input: Send + 'static;
    /// Output type for processing.
    type Output: Send + 'static;
    /// State type for maintaining worker state.
    type State;

    /// Initiate the worker
    fn init(ctx: &Arc<Self::Context>) -> Self::State;

    /// Process the worker
    fn process(
        state: &mut Self::State,
        input: Self::Input,
        cancel_token: CancelToken,
    ) -> Self::Output;

    /// Garbage collect the resources to free memory,
    ///
    /// but not drop the state itself, so that it can be reused.
    fn gc(_state: &mut Self::State) {
        // By default, do nothing
    }
}
