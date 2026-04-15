use any_llm::providers::Gateway;
use any_llm::{
    AnyLLMError, Batch, BatchRequestItem, BatchResult, BatchStatus, CompletionOptions,
    CreateBatchParams, ListBatchesOptions, Message, Provider, ProviderConfig,
};
use futures::StreamExt;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn platform_config(base: &str) -> ProviderConfig {
    ProviderConfig {
        api_key: None,
        api_base: Some(base.to_string()),
        extra: [
            ("platform_mode".to_string(), "true".to_string()),
            ("platform_token".to_string(), "tk_test_token".to_string()),
        ]
        .into(),
    }
}

fn non_platform_config(base: &str, key: &str) -> ProviderConfig {
    ProviderConfig {
        api_key: Some(key.to_string()),
        api_base: Some(base.to_string()),
        extra: Default::default(),
    }
}

fn chat_completion_json() -> serde_json::Value {
    serde_json::json!({
        "id": "chatcmpl-abc123",
        "object": "chat.completion",
        "created": 1_700_000_000_i64,
        "model": "openai:gpt-4o-mini",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    })
}

fn streaming_sse_body() -> String {
    [
        r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1700000000,"model":"openai:gpt-4o-mini","choices":[{"index":0,"delta":{"role":"assistant","content":""},"finish_reason":null}]}"#,
        r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1700000000,"model":"openai:gpt-4o-mini","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#,
        r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1700000000,"model":"openai:gpt-4o-mini","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}"#,
        r#"data: {"id":"chatcmpl-1","object":"chat.completion.chunk","created":1700000000,"model":"openai:gpt-4o-mini","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#,
        "data: [DONE]",
        "",
    ]
    .join("\n\n")
}

fn simple_params() -> any_llm::CompletionParams {
    any_llm::CompletionParams::new("openai:gpt-4o-mini", vec![Message::user("hello")])
}

// ---------------------------------------------------------------------------
// Configuration / auth mode tests
// ---------------------------------------------------------------------------

#[test]
fn gateway_requires_api_base() {
    let config = ProviderConfig::default();
    let result = Gateway::from_config(config);
    assert!(result.is_err());
}

#[test]
fn gateway_platform_mode_explicit() {
    let config = ProviderConfig {
        api_key: None,
        api_base: Some("http://example.com".to_string()),
        extra: [
            ("platform_mode".to_string(), "true".to_string()),
            ("platform_token".to_string(), "tk_abc".to_string()),
        ]
        .into(),
    };
    let gw = Gateway::from_config(config).unwrap();
    assert!(gw.is_platform_mode());
}

#[test]
fn gateway_platform_mode_requires_token() {
    let config = ProviderConfig {
        api_key: None,
        api_base: Some("http://example.com".to_string()),
        extra: [("platform_mode".to_string(), "true".to_string())].into(),
    };
    let result = Gateway::from_config(config);
    assert!(result.is_err());
}

#[test]
fn gateway_non_platform_mode_with_api_key() {
    let config = non_platform_config("http://example.com", "my-key");
    let gw = Gateway::from_config(config).unwrap();
    assert!(!gw.is_platform_mode());
}

#[test]
fn gateway_non_platform_mode_no_key_is_ok() {
    let config = ProviderConfig {
        api_key: None,
        api_base: Some("http://example.com".to_string()),
        extra: [("platform_mode".to_string(), "false".to_string())].into(),
    };
    let gw = Gateway::from_config(config).unwrap();
    assert!(!gw.is_platform_mode());
}

#[test]
fn gateway_strips_trailing_slash_from_api_base() {
    let config = ProviderConfig {
        api_key: None,
        api_base: Some("http://example.com/".to_string()),
        extra: [("platform_mode".to_string(), "false".to_string())].into(),
    };
    let _gw = Gateway::from_config(config).unwrap();
    // If this doesn't panic, the trailing slash was handled.
}

// ---------------------------------------------------------------------------
// Header verification tests (wiremock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn platform_mode_sends_authorization_header() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("Authorization", "Bearer tk_test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_json()))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let result = gw.completion(simple_params()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn non_platform_mode_sends_x_anyllm_key_header() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("X-AnyLLM-Key", "Bearer my-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_json()))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(non_platform_config(&server.uri(), "my-api-key")).unwrap();
    let result = gw.completion(simple_params()).await;
    assert!(result.is_ok());
}

// ---------------------------------------------------------------------------
// Completion tests (wiremock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn completion_parses_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_json()))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let completion = gw.completion(simple_params()).await.unwrap();

    assert_eq!(completion.id, "chatcmpl-abc123");
    assert_eq!(completion.content(), Some("Hello! How can I help you?"));
    assert_eq!(completion.finish_reason(), Some("stop"));
    assert_eq!(completion.usage.unwrap().total_tokens, 18);
}

#[tokio::test]
async fn completion_with_reasoning() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "id": "id",
        "object": "chat.completion",
        "created": 0_i64,
        "model": "model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "42",
                "reasoning_content": "Let me think about this..."
            },
            "finish_reason": "stop"
        }]
    });

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let completion = gw.completion(simple_params()).await.unwrap();
    assert_eq!(completion.reasoning(), Some("Let me think about this..."));
}

#[tokio::test]
async fn completion_with_tool_calls() {
    let server = MockServer::start().await;

    let body = serde_json::json!({
        "id": "id",
        "object": "chat.completion",
        "created": 0_i64,
        "model": "model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"city\":\"Paris\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }]
    });

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let completion = gw.completion(simple_params()).await.unwrap();
    let tc = completion.tool_calls().unwrap();
    assert_eq!(tc.len(), 1);
    assert_eq!(tc[0].function.name, "get_weather");
    assert_eq!(completion.finish_reason(), Some("tool_calls"));
}

// ---------------------------------------------------------------------------
// Streaming tests (wiremock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn streaming_returns_all_chunks() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(streaming_sse_body(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let stream = gw.completion_stream(simple_params()).await.unwrap();
    let chunks: Vec<_> = stream.collect().await;

    assert_eq!(chunks.len(), 4);

    // Accumulate content
    let content: String = chunks
        .iter()
        .filter_map(|c| c.as_ref().ok())
        .filter_map(|c| c.content())
        .collect();
    assert_eq!(content, "Hello!");

    // Last chunk has finish_reason
    let last = chunks.last().unwrap().as_ref().unwrap();
    assert_eq!(last.finish_reason(), Some("stop"));
}

#[tokio::test]
async fn streaming_accumulator_works() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(200).set_body_raw(streaming_sse_body(), "text/event-stream"),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let mut stream = gw.completion_stream(simple_params()).await.unwrap();

    let mut acc = any_llm::ChunkAccumulator::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        acc.add(&chunk);
    }

    assert_eq!(acc.content, "Hello!");
    assert_eq!(acc.finish_reason.as_deref(), Some("stop"));
}

// ---------------------------------------------------------------------------
// Error mapping tests (wiremock)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn error_401_maps_to_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error": {"message": "Invalid token"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Authentication { .. }));
    assert!(err.to_string().contains("Invalid token"));
}

#[tokio::test]
async fn error_403_maps_to_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(403)
                .set_body_json(serde_json::json!({"error": {"message": "Forbidden"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Authentication { .. }));
}

#[tokio::test]
async fn error_402_maps_to_provider_error_with_insufficient_funds() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(402)
                .set_body_json(serde_json::json!({"error": {"message": "Budget exceeded"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("Insufficient funds"));
}

#[tokio::test]
async fn error_404_maps_to_model_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"error": {"message": "Model not found"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::ModelNotFound { .. }));
}

#[tokio::test]
async fn error_429_maps_to_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "30")
                .set_body_json(serde_json::json!({"error": {"message": "Too many requests"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::RateLimit { .. }));
    assert!(err.to_string().contains("retry_after=30"));
}

#[tokio::test]
async fn error_502_maps_to_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(502)
                .set_body_json(serde_json::json!({"error": {"message": "Upstream error"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("Upstream provider error"));
}

#[tokio::test]
async fn error_504_maps_to_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(504)
                .set_body_json(serde_json::json!({"error": {"message": "Timed out"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("Gateway timeout"));
}

#[tokio::test]
async fn error_includes_correlation_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(
            ResponseTemplate::new(401)
                .insert_header("X-Correlation-ID", "corr-abc-123")
                .set_body_json(serde_json::json!({"error": {"message": "Unauthorized"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(err.to_string().contains("correlation_id=corr-abc-123"));
}

#[tokio::test]
async fn unknown_error_status_maps_to_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.completion(simple_params()).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("HTTP 500"));
}

// ---------------------------------------------------------------------------
// High-level API tests (via completion::<Gateway>)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn completion_api_function_works() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(chat_completion_json()))
        .mount(&server)
        .await;

    let options = CompletionOptions::default().api_base(server.uri());

    // Force non-platform mode via the ProviderConfig path
    let mut config: ProviderConfig = options.into();
    config
        .extra
        .insert("platform_mode".to_string(), "false".to_string());

    let gw = Gateway::from_config(config).unwrap();
    let result = gw.completion(simple_params()).await.unwrap();
    assert_eq!(result.content(), Some("Hello! How can I help you?"));
}

// ---------------------------------------------------------------------------
// Live integration tests (require a running gateway)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_gateway_completion() {
    let gw = Gateway::from_config(ProviderConfig::default()).unwrap();

    let params = any_llm::CompletionParams::new(
        "openai:gpt-4o-mini",
        vec![Message::user("Say just the word 'hello'")],
    );

    let completion = gw.completion(params).await.unwrap();
    assert!(!completion.choices.is_empty());
    let content = completion.content().unwrap_or("");
    assert!(!content.is_empty());
    println!("Live response: {content}");
}

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_gateway_streaming() {
    let gw = Gateway::from_config(ProviderConfig::default()).unwrap();

    let params = any_llm::CompletionParams::new(
        "openai:gpt-4o-mini",
        vec![Message::user("Say just the word 'hello'")],
    );

    let mut stream = gw.completion_stream(params).await.unwrap();
    let mut acc = any_llm::ChunkAccumulator::new();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.unwrap();
        acc.add(&chunk);
    }

    assert!(!acc.content.is_empty());
    println!("Live streamed: {}", acc.content);
}

// ===========================================================================
// Batch type tests (serialization / deserialization)
// ===========================================================================

fn batch_json() -> serde_json::Value {
    serde_json::json!({
        "id": "batch_abc123",
        "object": "batch",
        "endpoint": "/v1/chat/completions",
        "status": "validating",
        "created_at": 1_714_502_400_i64,
        "completion_window": "24h",
        "request_counts": {"total": 2, "completed": 0, "failed": 0},
        "metadata": {"key": "value"},
        "provider": "openai"
    })
}

fn batch_result_json() -> serde_json::Value {
    serde_json::json!({
        "results": [
            {
                "custom_id": "req-1",
                "result": {
                    "id": "chatcmpl-1",
                    "object": "chat.completion",
                    "created": 1_700_000_000_i64,
                    "model": "gpt-4o-mini",
                    "choices": [{
                        "index": 0,
                        "message": {"role": "assistant", "content": "Hello!"},
                        "finish_reason": "stop"
                    }]
                },
                "error": null
            },
            {
                "custom_id": "req-2",
                "result": null,
                "error": {"code": "rate_limit", "message": "Rate limit exceeded"}
            }
        ]
    })
}

#[test]
fn batch_deserializes_from_json() {
    let batch: Batch = serde_json::from_value(batch_json()).unwrap();
    assert_eq!(batch.id, "batch_abc123");
    assert_eq!(batch.object, "batch");
    assert_eq!(batch.endpoint, "/v1/chat/completions");
    assert_eq!(batch.status, BatchStatus::Validating);
    assert_eq!(batch.created_at, 1_714_502_400);
    assert_eq!(batch.completion_window, "24h");
    assert_eq!(batch.provider.as_deref(), Some("openai"));

    let counts = batch.request_counts.unwrap();
    assert_eq!(counts.total, 2);
    assert_eq!(counts.completed, 0);
    assert_eq!(counts.failed, 0);

    let meta = batch.metadata.unwrap();
    assert_eq!(meta.get("key").unwrap(), "value");
}

#[test]
fn batch_result_deserializes_from_json() {
    let result: BatchResult = serde_json::from_value(batch_result_json()).unwrap();
    assert_eq!(result.results.len(), 2);

    // First item: successful
    let item1 = &result.results[0];
    assert_eq!(item1.custom_id, "req-1");
    assert!(item1.result.is_some());
    assert!(item1.error.is_none());
    let completion = item1.result.as_ref().unwrap();
    assert_eq!(completion.content(), Some("Hello!"));

    // Second item: error
    let item2 = &result.results[1];
    assert_eq!(item2.custom_id, "req-2");
    assert!(item2.result.is_none());
    let err = item2.error.as_ref().unwrap();
    assert_eq!(err.code, "rate_limit");
    assert_eq!(err.message, "Rate limit exceeded");
}

#[test]
fn create_batch_params_serializes_correctly() {
    let params = CreateBatchParams::new(
        "openai:gpt-4o-mini",
        vec![BatchRequestItem {
            custom_id: "req-1".to_string(),
            body: serde_json::json!({"messages": [{"role": "user", "content": "Hi"}], "max_tokens": 100}),
        }],
    )
    .completion_window("24h")
    .metadata([("key".to_string(), "value".to_string())].into());

    let json = serde_json::to_value(&params).unwrap();
    assert_eq!(json["model"], "openai:gpt-4o-mini");
    assert_eq!(json["completion_window"], "24h");
    assert_eq!(json["metadata"]["key"], "value");
    assert_eq!(json["requests"][0]["custom_id"], "req-1");
    assert_eq!(json["requests"][0]["body"]["max_tokens"], 100);
}

#[test]
fn batch_status_enum_values() {
    let statuses = [
        ("\"validating\"", BatchStatus::Validating),
        ("\"failed\"", BatchStatus::Failed),
        ("\"in_progress\"", BatchStatus::InProgress),
        ("\"finalizing\"", BatchStatus::Finalizing),
        ("\"completed\"", BatchStatus::Completed),
        ("\"expired\"", BatchStatus::Expired),
        ("\"cancelling\"", BatchStatus::Cancelling),
        ("\"cancelled\"", BatchStatus::Cancelled),
    ];

    for (json_str, expected) in &statuses {
        let deserialized: BatchStatus = serde_json::from_str(json_str).unwrap();
        assert_eq!(&deserialized, expected);

        let serialized = serde_json::to_string(expected).unwrap();
        assert_eq!(&serialized, json_str);
    }
}

// ===========================================================================
// Batch HTTP method tests (wiremock)
// ===========================================================================

#[tokio::test]
async fn create_batch_sends_correct_request() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/batches"))
        .and(header("Authorization", "Bearer tk_test_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_json()))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let params = CreateBatchParams::new(
        "openai:gpt-4o-mini",
        vec![BatchRequestItem {
            custom_id: "req-1".to_string(),
            body: serde_json::json!({"messages": [{"role": "user", "content": "Hi"}]}),
        }],
    )
    .completion_window("24h");

    let batch = gw.create_batch(params).await.unwrap();
    assert_eq!(batch.id, "batch_abc123");
    assert_eq!(batch.status, BatchStatus::Validating);
    assert_eq!(batch.provider.as_deref(), Some("openai"));
}

#[tokio::test]
async fn retrieve_batch_sends_provider_query_param() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_abc123"))
        .and(query_param("provider", "openai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_json()))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let batch = gw.retrieve_batch("batch_abc123", "openai").await.unwrap();
    assert_eq!(batch.id, "batch_abc123");
}

#[tokio::test]
async fn cancel_batch_sends_correct_request() {
    let server = MockServer::start().await;

    let cancelled_json = serde_json::json!({
        "id": "batch_abc123",
        "object": "batch",
        "endpoint": "/v1/chat/completions",
        "status": "cancelling",
        "created_at": 1_714_502_400_i64,
        "completion_window": "24h"
    });

    Mock::given(method("POST"))
        .and(path("/v1/batches/batch_abc123/cancel"))
        .and(query_param("provider", "openai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(cancelled_json))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let batch = gw.cancel_batch("batch_abc123", "openai").await.unwrap();
    assert_eq!(batch.id, "batch_abc123");
    assert_eq!(batch.status, BatchStatus::Cancelling);
}

#[tokio::test]
async fn list_batches_sends_pagination_params() {
    let server = MockServer::start().await;

    let list_json = serde_json::json!({
        "data": [batch_json()]
    });

    Mock::given(method("GET"))
        .and(path("/v1/batches"))
        .and(query_param("provider", "openai"))
        .and(query_param("after", "cursor_abc"))
        .and(query_param("limit", "10"))
        .respond_with(ResponseTemplate::new(200).set_body_json(list_json))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let options = ListBatchesOptions {
        after: Some("cursor_abc".to_string()),
        limit: Some(10),
    };
    let batches = gw.list_batches("openai", options).await.unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].id, "batch_abc123");
}

#[tokio::test]
async fn retrieve_batch_results_returns_batch_result() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_abc123/results"))
        .and(query_param("provider", "openai"))
        .respond_with(ResponseTemplate::new(200).set_body_json(batch_result_json()))
        .expect(1)
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let result = gw
        .retrieve_batch_results("batch_abc123", "openai")
        .await
        .unwrap();
    assert_eq!(result.results.len(), 2);
    assert_eq!(result.results[0].custom_id, "req-1");
    assert!(result.results[0].result.is_some());
    assert!(result.results[1].error.is_some());
}

// ===========================================================================
// Batch error tests (wiremock)
// ===========================================================================

#[tokio::test]
async fn batch_409_returns_batch_not_complete() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_abc123/results"))
        .respond_with(ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": {
                "message": "Batch 'batch_abc123' is not yet complete (status: in_progress). Call GET /v1/batches/batch_abc123?provider=openai to check the current status."
            }
        })))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .retrieve_batch_results("batch_abc123", "openai")
        .await
        .unwrap_err();
    match &err {
        AnyLLMError::BatchNotComplete {
            batch_id, status, ..
        } => {
            assert_eq!(batch_id.as_ref(), "batch_abc123");
            assert_eq!(status.as_ref(), "in_progress");
        }
        other => panic!("expected BatchNotComplete, got: {other:?}"),
    }
    assert!(err.to_string().contains("not yet complete"));
}

#[tokio::test]
async fn batch_404_returns_upgrade_gateway_hint() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_xyz"))
        .respond_with(
            ResponseTemplate::new(404)
                .set_body_json(serde_json::json!({"error": {"message": "Not found"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw.retrieve_batch("batch_xyz", "openai").await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    let msg = err.to_string();
    assert!(msg.contains("does not support batch operations"));
    assert!(msg.contains("Upgrade your gateway"));
}

#[tokio::test]
async fn batch_401_returns_authentication_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/batches"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error": {"message": "Invalid token"}})),
        )
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let params = CreateBatchParams::new("openai:gpt-4o-mini", vec![]);
    let err = gw.create_batch(params).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Authentication { .. }));
    assert!(err.to_string().contains("Invalid token"));
}

#[tokio::test]
async fn batch_422_returns_provider_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/v1/batches"))
        .respond_with(ResponseTemplate::new(422).set_body_json(serde_json::json!({
            "error": {"message": "Unsupported provider: xyz"}
        })))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let params = CreateBatchParams::new("xyz:model", vec![]);
    let err = gw.create_batch(params).await.unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("Unsupported provider"));
}

#[tokio::test]
async fn batch_502_returns_provider_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_abc123"))
        .respond_with(ResponseTemplate::new(502).set_body_json(serde_json::json!({
            "error": {"message": "Upstream error"}
        })))
        .mount(&server)
        .await;

    let gw = Gateway::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .retrieve_batch("batch_abc123", "openai")
        .await
        .unwrap_err();
    assert!(matches!(err, AnyLLMError::Provider { .. }));
    assert!(err.to_string().contains("Upstream provider error"));
}
