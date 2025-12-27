use async_trait::async_trait;
use hone::HoneResult;
use hone::context::{Computer, Context};
use hone::engine::{Engine, ResolveOptions};
use hone::node::{NodeKey, NodeValue};
use hone::status::{Hash, HashPair, NodeData};
use rkyv::{Archive, Deserialize, Serialize};
use std::sync::Arc;
use zako_cancel::CancelSource;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Archive, Serialize, Deserialize)]
pub struct TestKey(pub String);

impl NodeKey for TestKey {}

#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct TestValue(pub i32);

impl NodeValue for TestValue {}

#[derive(Debug)]
struct TestComputer;

#[async_trait]
impl Computer<(), TestKey, TestValue> for TestComputer {
    async fn compute<'c>(
        &self,
        ctx: &'c Context<(), TestKey, TestValue>,
    ) -> HoneResult<NodeData<(), TestValue>> {
        let key = ctx.this();
        let value = if key.0 == "a" {
            let b = ctx.request(TestKey("b".to_string())).await?;
            let c = ctx.request(TestKey("c".to_string())).await?;
            b.value().0 + c.value().0
        } else if key.0 == "b" {
            10
        } else if key.0 == "c" {
            20
        } else if key.0 == "cycle" {
            ctx.request(TestKey("cycle".to_string())).await?;
            0
        } else {
            0
        };

        Ok(NodeData::new(
            HashPair {
                output_hash: Hash::from_bytes(&[0; 32]),
                input_hash: Hash::from_bytes(&[0; 32]),
            },
            Arc::new(TestValue(value)),
        ))
    }
}

#[tokio::test]
async fn test_engine_basic_resolve() {
    let db_path = "test_basic_resolve.redb";
    let _ = std::fs::remove_file(db_path);
    let db = redb::Database::create(db_path).unwrap();
    let engine = Engine::new(Arc::new(TestComputer), Arc::new(db)).unwrap();

    let cancel_source = CancelSource::new();
    let result = engine
        .resolve(
            TestKey("a".to_string()),
            cancel_source.token(),
            ResolveOptions::default(),
            &(),
        )
        .await;

    assert!(
        result.is_ok(),
        "Result should be OK, but got {:?}",
        result.err()
    );
    assert_eq!(result.unwrap().value().0, 30);
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_engine_cycle_detection() {
    let db_path = "test_cycle_detection.redb";
    let _ = std::fs::remove_file(db_path);
    let db = redb::Database::create(db_path).unwrap();
    let engine = Engine::new(Arc::new(TestComputer), Arc::new(db)).unwrap();

    let cancel_source = CancelSource::new();
    let result = engine
        .resolve(
            TestKey("cycle".to_string()),
            cancel_source.token(),
            ResolveOptions::default(),
            &(),
        )
        .await;

    assert!(result.is_err(), "Should detect circular dependency");
    let _ = std::fs::remove_file(db_path);
}

#[tokio::test]
async fn test_engine_cancellation() {
    let db_path = "test_cancellation.redb";
    let _ = std::fs::remove_file(db_path);
    let db = redb::Database::create(db_path).unwrap();

    #[derive(Debug)]
    struct SlowComputer;

    #[async_trait]
    impl Computer<(), TestKey, TestValue> for SlowComputer {
        async fn compute<'c>(
            &self,
            ctx: &'c Context<(), TestKey, TestValue>,
        ) -> HoneResult<NodeData<(), TestValue>> {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if ctx.cancel_token().is_cancelled() {
                return Err(hone::error::HoneError::Canceled {
                    reason: ctx.cancel_token().reason().clone(),
                });
            }
            Ok(NodeData::new(
                HashPair {
                    output_hash: Hash::from_bytes(&[0; 32]),
                    input_hash: Hash::from_bytes(&[0; 32]),
                },
                Arc::new(TestValue(1)),
            ))
        }
    }

    let engine = Engine::new(Arc::new(SlowComputer), Arc::new(db)).unwrap();
    let cancel_source = CancelSource::new();
    let token = cancel_source.token();

    let handle = tokio::spawn(async move {
        engine
            .resolve(
                TestKey("slow".to_string()),
                token,
                ResolveOptions::default(),
                &(),
            )
            .await
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    cancel_source.cancel(zako_cancel::CancelReason::Invalidated);

    let result = handle.await.unwrap();
    assert!(result.is_err());
    let _ = std::fs::remove_file(db_path);
}
