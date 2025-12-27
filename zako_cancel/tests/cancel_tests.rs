use std::time::Duration;
use zako_cancel::{CancelReason, CancelSource};

#[tokio::test]
async fn test_basic_cancel() {
    let source = CancelSource::new();
    let token = source.token();

    assert!(!token.is_cancelled());

    source.cancel(CancelReason::Invalidated);

    assert!(token.is_cancelled());
    match token.reason() {
        Some(CancelReason::Invalidated) => (),
        _ => panic!("Expected Invalidated reason"),
    }
}

#[tokio::test]
async fn test_wait_cancel() {
    let source = CancelSource::new();
    let token = source.token();

    let token_clone = token.clone();
    let handle = tokio::spawn(async move {
        token_clone.cancelled().await;
        true
    });

    tokio::time::sleep(Duration::from_millis(10)).await;
    source.cancel(CancelReason::SiblingFailed);

    assert!(handle.await.unwrap());
}

#[tokio::test]
async fn test_child_token_cancel() {
    let source = CancelSource::new();
    let parent_token = source.token();
    let child_token = parent_token.child_token();

    assert!(!parent_token.is_cancelled());
    assert!(!child_token.is_cancelled());

    source.cancel(CancelReason::Timeout(Duration::from_secs(1)));

    assert!(parent_token.is_cancelled());
    assert!(child_token.is_cancelled());
}

#[tokio::test]
async fn test_child_source_cancel() {
    let source = CancelSource::new();
    let child_source = source.create_child();
    let parent_token = source.token();
    let child_token = child_source.token();

    child_source.cancel(CancelReason::Invalidated);

    assert!(child_token.is_cancelled());
    // Child cancellation should NOT propagate to parent in this implementation
    // because SharedState::child() creates a child token linked to parent,
    // but the source creation is a bit different.
    // Looking at `SharedState::child(&self)`: it returns a state with a child token of parent.
    // In `CancelSource::create_child(&self)`, it creates a new CancelSource with this child state.
    // So if child cancels, it cancels its own token, but not the parent's.
    assert!(!parent_token.is_cancelled());
}

#[tokio::test]
async fn test_parent_cancel_propagates_to_child() {
    let source = CancelSource::new();
    let child_source = source.create_child();
    let parent_token = source.token();
    let child_token = child_source.token();

    source.cancel(CancelReason::SiblingFailed);

    assert!(parent_token.is_cancelled());
    assert!(child_token.is_cancelled());
}

#[test]
fn test_cancel_reason_once_lock() {
    let source = CancelSource::new();
    let token = source.token();

    source.cancel(CancelReason::Invalidated);
    source.cancel(CancelReason::SiblingFailed);

    match token.reason() {
        Some(CancelReason::Invalidated) => (),
        _ => panic!("Reason should be the first one set"),
    }
}
