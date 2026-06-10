//! Tests for the generated-core ergonomic surface on `Otari`.
//!
//! Mirrors the Python reference's rewrite (`tests/unit/test_client.py`): a mock
//! HTTP server (`wiremock`) asserts method / URL / headers per auth mode, body
//! shaping, and typed deserialization per method; error statuses map to typed
//! `OtariError` variants; mocked `text/event-stream` bytes exercise the chat
//! and responses/messages SSE shims and the `[DONE]` stop.
//!
//! No LLM provider key is available here, so real streamed chat cannot be
//! exercised end to end — these feed mocked SSE bytes only.

use futures::StreamExt;
use otari::{Config, Otari, OtariError};
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn platform_client(base: &str) -> Otari {
    Otari::from_config(Config {
        api_key: None,
        api_base: Some(base.to_string()),
        extra: [
            ("platform_mode".to_string(), "true".to_string()),
            ("platform_token".to_string(), "tk".to_string()),
        ]
        .into(),
    })
    .unwrap()
}

fn key_client(base: &str) -> Otari {
    Otari::from_config(Config {
        api_key: Some("vk".to_string()),
        api_base: Some(base.to_string()),
        extra: Default::default(),
    })
    .unwrap()
}

fn chat_response() -> serde_json::Value {
    json!({
        "id": "chatcmpl-1",
        "object": "chat.completion",
        "created": 1,
        "model": "openai:gpt-4o-mini",
        "choices": [
            {"index": 0, "finish_reason": "stop",
             "message": {"role": "assistant", "content": "Hi"}}
        ]
    })
}

fn sse_body(events: &[&str]) -> String {
    let mut body = String::new();
    for e in events {
        body.push_str("data: ");
        body.push_str(e);
        body.push_str("\n\n");
    }
    body.push_str("data: [DONE]\n\n");
    body
}

// ---------------------------------------------------------------------------
// Request shaping + typed response parsing (non-platform / Otari-Key mode)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn chat_returns_typed_completion_with_otari_key() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Otari-Key", "Bearer vk"))
        .and(body_partial_json(
            json!({"model": "openai:gpt-4o-mini", "temperature": 0.5}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_response()))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .chat(json!({
            "model": "openai:gpt-4o-mini",
            "messages": [{"role": "user", "content": "Hi"}],
            "temperature": 0.5
        }))
        .await
        .unwrap();
    assert_eq!(
        result.choices[0]
            .message
            .content
            .as_ref()
            .and_then(|c| c.as_deref()),
        Some("Hi")
    );
}

#[tokio::test]
async fn chat_sends_bearer_in_platform_mode() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer tk"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_response()))
        .mount(&server)
        .await;

    let client = platform_client(&server.uri());
    client
        .chat(json!({"model": "m", "messages": [{"role": "user", "content": "Hi"}]}))
        .await
        .unwrap();
}

#[tokio::test]
async fn embedding_returns_typed_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/embeddings"))
        .and(body_partial_json(json!({"input": "hello"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "model": "openai:text-embedding-3-small",
            "data": [{"object": "embedding", "index": 0, "embedding": [0.1, 0.2]}],
            "usage": {"prompt_tokens": 1, "total_tokens": 1}
        })))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .embedding(json!({"model": "openai:text-embedding-3-small", "input": "hello"}))
        .await
        .unwrap();
    assert_eq!(result.data[0].embedding, vec![0.1, 0.2]);
}

#[tokio::test]
async fn rerank_typed_returns_typed_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/rerank"))
        .and(body_partial_json(json!({"documents": ["a", "b"]})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "rerank-1",
            "results": [{"index": 0, "relevance_score": 0.9}]
        })))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .rerank_typed(json!({"model": "m", "query": "q", "documents": ["a", "b"]}))
        .await
        .unwrap();
    assert!((result.results[0].relevance_score - 0.9).abs() < f64::EPSILON);
}

#[tokio::test]
async fn moderate_returns_typed_response() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "modr-1",
            "model": "m",
            "results": [{"flagged": false, "categories": {}, "category_scores": {}}]
        })))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .moderate(json!({"model": "m", "input": "text"}), false)
        .await
        .unwrap();
    assert!(!result.results[0].flagged);
}

#[tokio::test]
async fn moderate_include_raw_sets_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(query_param("include_raw", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "modr-1", "model": "m",
            "results": [{"flagged": false, "categories": {}, "category_scores": {}}]
        })))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    client
        .moderate(json!({"model": "m", "input": "text"}), true)
        .await
        .unwrap();
}

#[tokio::test]
async fn message_returns_raw_value() {
    let server = MockServer::start().await;
    let msg = json!({
        "id": "msg-1",
        "type": "message",
        "role": "assistant",
        "model": "anthropic:claude-3-5-sonnet",
        "content": [{"type": "text", "text": "Hi"}],
        "usage": {"input_tokens": 1, "output_tokens": 1}
    });
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("Otari-Key", "Bearer vk"))
        .and(body_partial_json(
            json!({"max_tokens": 64, "model": "anthropic:claude-3-5-sonnet"}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(msg))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .message(json!({
            "model": "anthropic:claude-3-5-sonnet",
            "messages": [{"role": "user", "content": "Hi"}],
            "max_tokens": 64
        }))
        .await
        .unwrap();
    assert_eq!(result["id"], "msg-1");
}

#[tokio::test]
async fn list_models_returns_typed_models() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list",
            "data": [{"id": "openai:gpt-4o", "object": "model", "created": 1, "owned_by": "openai"}]
        })))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let models = client.list_models(None).await.unwrap();
    assert_eq!(models[0].id, "openai:gpt-4o");
}

#[tokio::test]
async fn response_returns_raw_value() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"id": "resp-1", "output": []})),
        )
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let result = client
        .response(json!({"model": "m", "input": "Hi"}))
        .await
        .unwrap();
    assert_eq!(result["id"], "resp-1");
}

// ---------------------------------------------------------------------------
// Error mapping (generated Error<T> -> typed OtariError), both auth modes
// ---------------------------------------------------------------------------

async fn assert_status_maps(status: u16, check: impl Fn(&OtariError) -> bool) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(status).set_body_json(json!({"detail": "boom"})))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let err = client
        .chat(json!({"model": "m", "messages": [{"role": "user", "content": "Hi"}]}))
        .await
        .unwrap_err();
    assert!(check(&err), "status {status} mapped to {err:?}");
    assert!(err.to_string().contains("boom"), "status {status}: {err}");
}

#[tokio::test]
async fn error_statuses_map_to_typed_errors() {
    assert_status_maps(401, |e| matches!(e, OtariError::Authentication { .. })).await;
    assert_status_maps(403, |e| matches!(e, OtariError::Authentication { .. })).await;
    assert_status_maps(404, |e| matches!(e, OtariError::ModelNotFound { .. })).await;
    assert_status_maps(429, |e| matches!(e, OtariError::RateLimit { .. })).await;
    assert_status_maps(502, |e| matches!(e, OtariError::Provider { .. })).await;
    assert_status_maps(503, |e| matches!(e, OtariError::Provider { .. })).await;
    assert_status_maps(504, |e| matches!(e, OtariError::Provider { .. })).await;
    assert_status_maps(418, |e| matches!(e, OtariError::Provider { .. })).await;
}

#[tokio::test]
async fn insufficient_funds_maps_with_correlation_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(402)
                .insert_header("x-correlation-id", "abc-123")
                .set_body_json(json!({"detail": "no funds"})),
        )
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let err = client
        .chat(json!({"model": "m", "messages": [{"role": "user", "content": "Hi"}]}))
        .await
        .unwrap_err();
    assert!(matches!(err, OtariError::Provider { .. }));
    assert!(err.to_string().contains("abc-123"));
    assert!(err.to_string().contains("Insufficient funds"));
}

#[tokio::test]
async fn unsupported_moderation_maps_in_any_mode() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(
            ResponseTemplate::new(400)
                .set_body_json(json!({"detail": "Provider anthropic does not support moderation"})),
        )
        .mount(&server)
        .await;

    // Platform mode: still surfaces the capability error.
    let client = platform_client(&server.uri());
    let err = client
        .moderate(json!({"model": "anthropic:claude", "input": "text"}), false)
        .await
        .unwrap_err();
    match err {
        OtariError::Unsupported {
            provider,
            operation,
        } => {
            assert_eq!(provider, "anthropic");
            assert_eq!(operation, "moderation");
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// SSE streaming shim (responses / messages -> raw Value)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn response_stream_yields_raw_events_and_stops_on_done() {
    let server = MockServer::start().await;
    let body = sse_body(&[r#"{"type":"a","seq":1}"#, r#"{"type":"b","seq":2}"#]);
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .and(header("Accept", "text/event-stream"))
        .and(header("Otari-Key", "Bearer vk"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let stream = client
        .response_stream(json!({"model": "m", "input": "Hi"}))
        .await
        .unwrap();
    let events: Vec<_> = stream.collect().await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].as_ref().unwrap()["seq"], 1);
    assert_eq!(events[1].as_ref().unwrap()["seq"], 2);
}

#[tokio::test]
async fn message_stream_yields_raw_events_with_bearer() {
    let server = MockServer::start().await;
    let body = sse_body(&[
        r#"{"type":"message_start"}"#,
        r#"{"type":"content_block_delta"}"#,
    ]);
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .and(header("Authorization", "Bearer tk"))
        .and(header("Accept", "text/event-stream"))
        .respond_with(ResponseTemplate::new(200).set_body_raw(body, "text/event-stream"))
        .mount(&server)
        .await;

    let client = platform_client(&server.uri());
    let stream = client
        .message_stream(json!({"model": "m", "messages": [], "max_tokens": 8}))
        .await
        .unwrap();
    let events: Vec<_> = stream.collect().await;
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].as_ref().unwrap()["type"], "message_start");
}

// ---------------------------------------------------------------------------
// Control-plane accessor wiring
// ---------------------------------------------------------------------------

#[tokio::test]
async fn control_plane_sends_admin_bearer() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/keys"))
        .and(header("Authorization", "Bearer master"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([])))
        .mount(&server)
        .await;

    let client = key_client(&server.uri());
    let cp = client.control_plane("master");

    // Ergonomic alias delegates to the generated operation with the bearer header.
    let keys = cp.keys().list(None, None).await.unwrap();
    assert!(keys.is_empty());

    // Escape hatch: the generated functions stay reachable via `config()`.
    let raw = otari::control_plane::apis::keys_api::list_keys_v1_keys_get(cp.config(), None, None)
        .await
        .unwrap();
    assert!(raw.is_empty());
}
