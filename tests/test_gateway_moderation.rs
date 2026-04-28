use otari::{
    Config, ModerationContentPart, ModerationImageUrl, ModerationInput, ModerationParams, Otari,
    OtariError,
};
use wiremock::matchers::{body_json, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn platform_config(base: &str) -> Config {
    Config {
        api_key: None,
        api_base: Some(base.to_string()),
        extra: [
            ("platform_mode".to_string(), "true".to_string()),
            ("platform_token".to_string(), "tk_test_token".to_string()),
        ]
        .into(),
    }
}

fn moderation_response_json() -> serde_json::Value {
    serde_json::json!({
        "id": "modr-abc123",
        "model": "omni-moderation-latest",
        "results": [{
            "flagged": true,
            "categories": {"violence": true, "hate": false},
            "category_scores": {"violence": 0.93, "hate": 0.01},
            "category_applied_input_types": {"violence": ["text"]}
        }]
    })
}

// ---------------------------------------------------------------------------
// Happy-path tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn moderation_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let resp = gw
        .moderation(ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("hurt someone".into()),
        ))
        .await
        .unwrap();

    assert_eq!(resp.id, "modr-abc123");
    assert_eq!(resp.results.len(), 1);
    assert!(resp.results[0].flagged);
    assert_eq!(resp.results[0].categories.get("violence"), Some(&true));
    assert_eq!(resp.results[0].categories.get("hate"), Some(&false));
    assert!(resp.results[0].provider_raw.is_none());
    assert!(resp.results[0].category_applied_input_types.is_some());
}

#[tokio::test]
async fn moderation_include_raw_uses_query_param() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(query_param("include_raw", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let params = ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Text("x".into()),
    )
    .with_include_raw(true);

    let resp = gw.moderation(params).await.unwrap();
    assert_eq!(resp.results.len(), 1);
}

#[tokio::test]
async fn moderation_omits_include_raw_query_when_false() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let params = ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Text("x".into()),
    );
    assert!(!params.include_raw);
    gw.moderation(params).await.unwrap();
}

#[tokio::test]
async fn moderation_text_input_serializes_as_string() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(body_json(serde_json::json!({
            "model": "openai:omni-moderation-latest",
            "input": "hello"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    gw.moderation(ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Text("hello".into()),
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn moderation_batch_input_serializes_as_array() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(body_json(serde_json::json!({
            "model": "openai:omni-moderation-latest",
            "input": ["a", "b"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    gw.moderation(ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Batch(vec!["a".into(), "b".into()]),
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn moderation_multimodal_input_serializes_parts() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(body_json(serde_json::json!({
            "model": "openai:omni-moderation-latest",
            "input": [
                {"type": "text", "text": "caption"},
                {"type": "image_url", "image_url": {"url": "https://example.com/x.png"}}
            ]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let parts = vec![
        ModerationContentPart::Text {
            text: "caption".into(),
        },
        ModerationContentPart::ImageUrl {
            image_url: ModerationImageUrl {
                url: "https://example.com/x.png".into(),
            },
        },
    ];
    gw.moderation(ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Parts(parts),
    ))
    .await
    .unwrap();
}

#[tokio::test]
async fn moderation_with_user_includes_user_field() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .and(body_json(serde_json::json!({
            "model": "openai:omni-moderation-latest",
            "input": "x",
            "user": "user_123"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(moderation_response_json()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let params = ModerationParams::new(
        "openai:omni-moderation-latest",
        ModerationInput::Text("x".into()),
    )
    .with_user("user_123");
    gw.moderation(params).await.unwrap();
}

// ---------------------------------------------------------------------------
// Unsupported provider / multimodal mapping
// ---------------------------------------------------------------------------

#[tokio::test]
async fn moderation_unsupported_provider_maps_to_typed_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "detail": "Provider anthropic does not support moderation"
        })))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "anthropic:claude-3-haiku",
            ModerationInput::Text("x".into()),
        ))
        .await
        .unwrap_err();

    match err {
        OtariError::Unsupported {
            provider,
            operation,
        } => {
            assert_eq!(&*provider, "anthropic");
            assert_eq!(&*operation, "moderation");
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[tokio::test]
async fn moderation_unsupported_multimodal_maps_to_typed_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "detail": "Provider mistral does not support multimodal moderation input"
        })))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "mistral:mistral-moderation-latest",
            ModerationInput::Parts(vec![ModerationContentPart::Text { text: "x".into() }]),
        ))
        .await
        .unwrap_err();

    match err {
        OtariError::Unsupported {
            provider,
            operation,
        } => {
            assert_eq!(&*provider, "mistral");
            assert_eq!(&*operation, "multimodal_moderation");
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

#[tokio::test]
async fn moderation_unsupported_without_provider_prefix_falls_back_to_unknown() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
            "detail": "This backend does not support moderation."
        })))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "weird:thing",
            ModerationInput::Text("x".into()),
        ))
        .await
        .unwrap_err();

    match err {
        OtariError::Unsupported {
            provider,
            operation,
        } => {
            assert_eq!(&*provider, "unknown");
            assert_eq!(&*operation, "moderation");
        }
        other => panic!("expected Unsupported, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Generic HTTP error mapping still works for moderation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn moderation_error_401_maps_to_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(
            ResponseTemplate::new(401)
                .set_body_json(serde_json::json!({"error": {"message": "Invalid token"}})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("x".into()),
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::Authentication { .. }));
    assert!(err.to_string().contains("Invalid token"));
}

#[tokio::test]
async fn moderation_error_429_maps_to_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "5")
                .set_body_json(serde_json::json!({"error": {"message": "Too many requests"}})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("x".into()),
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::RateLimit { .. }));
    assert!(err.to_string().contains("retry_after=5"));
}

#[tokio::test]
async fn moderation_error_500_maps_to_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/moderations"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .moderation(ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("x".into()),
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::Provider { .. }));
    assert!(err.to_string().contains("HTTP 500"));
}

// ---------------------------------------------------------------------------
// Live integration test (requires a running gateway)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_gateway_moderation() {
    let gw = Otari::from_config(Config::default()).unwrap();
    let resp = gw
        .moderation(ModerationParams::new(
            "openai:omni-moderation-latest",
            ModerationInput::Text("hurt someone".into()),
        ))
        .await
        .unwrap();
    assert!(!resp.results.is_empty());
}
