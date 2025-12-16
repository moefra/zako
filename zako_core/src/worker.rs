use zako_cancel::CancelToken;

/// A worker and its behavior definition.
pub trait WorkerBehavior: Send + Sync + 'static {
    /// Input type for processing.
    type Input: Send + 'static;
    /// Output type for processing.
    type Output: Send + 'static;
    /// State type for maintaining worker state.
    type State;

    /// Initiate the worker
    fn init() -> Self::State;

    /// Process the worker
    fn process(
        state: &mut Self::State,
        input: Self::Input,
        cancel_token: CancelToken,
    ) -> Self::Output;

    /// Clean the resources
    fn clean(_state: Self::State) {}
}
