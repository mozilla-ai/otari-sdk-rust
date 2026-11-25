//! The SDK never follows a redirect: the gateway answers 307 to another host,
//! and that host must see nothing.

use otari::{CompletionParams, Config, Message, Otari, OtariError};
use wiremock::matchers::any;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A gateway that redirects every request to a second server, which fails the
/// test on drop if any request reaches it.
async fn redirecting_gateway() -> (MockServer, MockServer, Otari) {
    let other = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [],
        })))
        .expect(0)
        .mount(&other)
        .await;

    let gateway = MockServer::start().await;
    Mock::given(any())
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/api/v1/models", other.uri())),
        )
        .mount(&gateway)
        .await;

    let client = Otari::from_config(Config::new("vk").with_api_base(gateway.uri())).unwrap();
    (gateway, other, client)
}

fn params() -> CompletionParams {
    CompletionParams::new("openai:gpt-4o-mini", vec![Message::user("hello")])
}

fn assert_refused<T: std::fmt::Debug>(result: Result<T, OtariError>) {
    match result {
        Err(OtariError::Http(e)) if e.is_redirect() => {}
        other => panic!("expected a refused redirect, got: {other:?}"),
    }
}

#[tokio::test]
async fn generated_core_get_does_not_follow_a_redirect() {
    let (_gateway, _other, client) = redirecting_gateway().await;
    assert_refused(client.list_models(None).await);
}

#[tokio::test]
async fn completion_post_does_not_follow_a_redirect() {
    let (_gateway, _other, client) = redirecting_gateway().await;
    assert_refused(client.completion(params()).await);
}

#[tokio::test]
async fn streaming_post_does_not_follow_a_redirect() {
    let (_gateway, _other, client) = redirecting_gateway().await;
    assert_refused(client.completion_stream(params()).await.map(|_| ()));
}

#[tokio::test]
async fn control_plane_does_not_follow_a_redirect() {
    let (_gateway, _other, client) = redirecting_gateway().await;
    let keys = client.control_plane("master").keys().list(None, None).await;
    assert_refused(keys);
}
