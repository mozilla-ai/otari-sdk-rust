//! Tests for the image-generation and audio (speech / transcription) methods.

use otari::{Config, ImageGenerationParams, Otari, OtariError, SpeechParams, TranscriptionParams};
use wiremock::matchers::{body_json, header, header_exists, method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

mod common;

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

fn self_hosted_config(base: &str) -> Config {
    Config {
        api_key: Some("sk_self_hosted".to_string()),
        api_base: Some(base.to_string()),
        extra: Default::default(),
    }
}

// ---------------------------------------------------------------------------
// Image generation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn image_generation_returns_typed_response() {
    let server = MockServer::start().await;
    let response_json = serde_json::json!({
        "created": 1_700_000_000,
        "data": [{"url": "https://example.com/image.png"}]
    });
    Mock::given(method("POST"))
        .and(path("/v1/images/generations"))
        .and(body_json(serde_json::json!({
            "model": "openai:dall-e-3",
            "prompt": "a red bicycle",
            "size": "1024x1024"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(response_json.clone()))
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let result = gw
        .image_generation(
            ImageGenerationParams::new("openai:dall-e-3", "a red bicycle").with_size("1024x1024"),
        )
        .await
        .unwrap();

    assert_eq!(result.created, 1_700_000_000);
    let data = result.data.flatten().expect("data array present");
    assert_eq!(
        data[0].url.clone().flatten().as_deref(),
        Some("https://example.com/image.png")
    );
}

#[tokio::test]
async fn image_generation_sends_platform_bearer_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/images/generations"))
        .and(header("authorization", "Bearer tk_test_token"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(serde_json::json!({"created": 1_700_000_000, "data": []})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    gw.image_generation(ImageGenerationParams::new("openai:dall-e-3", "x"))
        .await
        .unwrap();
}

#[tokio::test]
async fn image_generation_error_429_maps_to_rate_limit() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/images/generations"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "7")
                .set_body_json(serde_json::json!({"error": {"message": "slow down"}})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .image_generation(ImageGenerationParams::new("openai:dall-e-3", "x"))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::RateLimit { .. }));
}

// ---------------------------------------------------------------------------
// Speech (text-to-speech)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn speech_returns_raw_bytes() {
    let server = MockServer::start().await;
    let audio = b"ID3\x00\x00\x00fake-mp3-bytes".to_vec();
    Mock::given(method("POST"))
        .and(path("/v1/audio/speech"))
        .and(body_json(serde_json::json!({
            "model": "openai:tts-1",
            "input": "hello world",
            "voice": "alloy"
        })))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/mpeg")
                .set_body_bytes(audio.clone()),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let bytes = gw
        .speech(SpeechParams::new("openai:tts-1", "hello world", "alloy"))
        .await
        .unwrap();

    assert_eq!(bytes.as_ref(), audio.as_slice());
}

#[tokio::test]
async fn speech_sends_self_hosted_otari_key_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/speech"))
        .and(header("otari-key", "Bearer sk_self_hosted"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "audio/mpeg")
                .set_body_bytes(b"audio".to_vec()),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(self_hosted_config(&server.uri())).unwrap();
    let bytes = gw
        .speech(
            SpeechParams::new("openai:tts-1", "hi", "alloy")
                .with_response_format("wav")
                .with_speed(1.25),
        )
        .await
        .unwrap();
    assert_eq!(bytes.as_ref(), b"audio");
}

#[tokio::test]
async fn speech_error_402_maps_to_provider_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/speech"))
        .respond_with(
            ResponseTemplate::new(402)
                .set_body_json(serde_json::json!({"detail": "budget exceeded"})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .speech(SpeechParams::new("openai:tts-1", "hi", "alloy"))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::Provider { .. }));
    assert!(err.to_string().contains("budget exceeded"));
}

// ---------------------------------------------------------------------------
// Transcription (multipart upload)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn transcription_sends_multipart_and_returns_json() {
    let server = MockServer::start().await;
    let response_json = serde_json::json!({"text": "hello there"});
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .and(header_exists("content-type"))
        .and(|req: &Request| {
            let ct = req
                .headers
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or_default();
            if !ct.starts_with("multipart/form-data") {
                return false;
            }
            let body = String::from_utf8_lossy(&req.body);
            body.contains("name=\"model\"")
                && body.contains("openai:whisper-1")
                && body.contains("name=\"file\"")
                && body.contains("filename=\"clip.mp3\"")
                && body.contains("fake-audio-bytes")
        })
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "application/json")
                .set_body_json(response_json.clone()),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let result = gw
        .transcription(
            TranscriptionParams::new("openai:whisper-1", b"fake-audio-bytes".to_vec())
                .with_filename("clip.mp3")
                .with_language("en"),
        )
        .await
        .unwrap();

    assert_eq!(result.json, Some(response_json));
    assert_eq!(result.text, None);
}

#[tokio::test]
async fn transcription_text_format_returns_string_value() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/plain; charset=utf-8")
                .set_body_string("just the words"),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let result = gw
        .transcription(
            TranscriptionParams::new("openai:whisper-1", b"bytes".to_vec())
                .with_response_format("text"),
        )
        .await
        .unwrap();

    assert_eq!(result.text, Some("just the words".to_string()));
    assert_eq!(result.json, None);
}

#[tokio::test]
async fn transcription_error_401_maps_to_authentication() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/audio/transcriptions"))
        .respond_with(
            ResponseTemplate::new(401).set_body_json(serde_json::json!({"detail": "bad token"})),
        )
        .mount(&server)
        .await;

    let gw = Otari::from_config(platform_config(&server.uri())).unwrap();
    let err = gw
        .transcription(TranscriptionParams::new(
            "openai:whisper-1",
            b"bytes".to_vec(),
        ))
        .await
        .unwrap_err();

    assert!(matches!(err, OtariError::Authentication { .. }));
}

// ---------------------------------------------------------------------------
// Live integration tests (require a running gateway)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore = "requires a running gateway server"]
async fn live_gateway_image_generation() {
    let Some(gw) = common::live_client() else {
        return;
    };
    let result = gw
        .image_generation(ImageGenerationParams::new(
            "openai:dall-e-3",
            "a red bicycle",
        ))
        .await
        .unwrap();
    assert!(result.data.is_some());
}
