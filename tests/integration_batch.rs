//! Integration tests for batch operations.
//!
//! These tests require a running gateway server and are gated behind `#[ignore]`.
//! Run with: `cargo test --test integration_batch -- --ignored`

use otari::{BatchRequestItem, BatchStatus, Config, CreateBatchParams, Otari, OtariError};

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_create_and_retrieve_batch() {
    let gw = Otari::from_config(Config::default()).unwrap();

    let params = CreateBatchParams::new(
        "openai:gpt-4o-mini",
        vec![BatchRequestItem {
            custom_id: "req-1".to_string(),
            body: serde_json::json!({
                "messages": [{"role": "user", "content": "Say hello"}],
                "max_tokens": 50
            }),
        }],
    )
    .completion_window("24h");

    let batch = gw.create_batch(params).await.unwrap();
    assert!(!batch.id.is_empty());
    println!("Created batch: {}", batch.id);

    // Retrieve the batch
    let provider = batch.provider.as_deref().unwrap_or("openai");
    let retrieved = gw.retrieve_batch(&batch.id, provider).await.unwrap();
    assert_eq!(retrieved.id, batch.id);
    println!("Retrieved batch status: {:?}", retrieved.status);

    // Cancel the batch
    let cancelled = gw.cancel_batch(&batch.id, provider).await.unwrap();
    assert!(
        cancelled.status == BatchStatus::Cancelling || cancelled.status == BatchStatus::Cancelled
    );
    println!("Cancelled batch status: {:?}", cancelled.status);
}

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_retrieve_batch_results_not_complete() {
    let gw = Otari::from_config(Config::default()).unwrap();

    let params = CreateBatchParams::new(
        "openai:gpt-4o-mini",
        vec![BatchRequestItem {
            custom_id: "req-1".to_string(),
            body: serde_json::json!({
                "messages": [{"role": "user", "content": "Say hello"}],
                "max_tokens": 50
            }),
        }],
    )
    .completion_window("24h");

    let batch = gw.create_batch(params).await.unwrap();
    let provider = batch.provider.as_deref().unwrap_or("openai");

    // Immediately try to get results — should fail with BatchNotComplete
    let err = gw
        .retrieve_batch_results(&batch.id, provider)
        .await
        .unwrap_err();
    assert!(matches!(err, OtariError::BatchNotComplete { .. }));
    println!("Got expected error: {err}");

    // Clean up
    let _ = gw.cancel_batch(&batch.id, provider).await;
}
