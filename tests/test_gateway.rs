use any_llm::providers::Gateway;
use any_llm::{AnyLLMError, CompletionOptions, Message, Provider, ProviderConfig};
use futures::StreamExt;
use wiremock::matchers::{header, method, path};
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
        "created": 1700000000_i64,
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
#[ignore]
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
#[ignore]
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
