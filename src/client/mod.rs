//! Otari client implementation.
//!
//! Connects to an [Otari gateway](https://github.com/mozilla-ai/otari-sdk-rust)
//! server, which exposes an OpenAI-compatible API that proxies to multiple
//! upstream LLM providers.
//!
//! # Auth modes
//!
//! - **Platform mode**: uses `Authorization: Bearer <token>`. Activated by
//!   setting `OTARI_AI_TOKEN` (canonical) or `OTARI_PLATFORM_TOKEN` (legacy
//!   alias) env var, or by passing `platform_token` in `Config::extra`. In
//!   platform mode, if no base URL is configured the hosted gateway
//!   (`https://api.otari.ai`) is used by default.
//! - **Non-platform mode**: uses the `Otari-Key: Bearer <key>` header.
//!   The key is optional (the gateway may allow unauthenticated access).
//!
//! # Environment variables
//!
//! Canonical names are read first, then the legacy alias as a fallback:
//!
//! - Platform token: `OTARI_AI_TOKEN` (canonical), `OTARI_PLATFORM_TOKEN` (legacy).
//! - API base URL: `GATEWAY_API_BASE` (canonical), `OTARI_API_BASE` (legacy).
//! - API key: `GATEWAY_API_KEY` (canonical), `OTARI_API_KEY` (legacy).

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use reqwest_eventsource::EventSource;

use serde::Deserialize;

use crate::config::Config;
use crate::core::{make_configuration, map_error, map_response};
use crate::error::{OtariError, Result};
use crate::types::{
    Batch, BatchResult, ChatCompletion, CompletionParams, CompletionStream, CreateBatchParams,
    ImageGenerationParams, ListBatchesOptions, ModerationParams, ModerationResponse, RerankParams,
    RerankResponse, SpeechParams, TranscriptionParams,
};

use crate::_client::apis::{images_api, models_api};
use crate::_client::models as gen_models;

pub mod models;

use models::request::GatewayRequest;
use models::response::GatewayResponse;
use models::stream::GatewayStream;

const OTARI_HEADER_NAME: &str = "Otari-Key";

/// Canonical platform token env var (matches the TS/Python SDKs).
const OTARI_AI_TOKEN_ENV: &str = "OTARI_AI_TOKEN";
/// Legacy platform token env var, kept as a back-compatible fallback.
const OTARI_PLATFORM_TOKEN_ENV: &str = "OTARI_PLATFORM_TOKEN";

/// Canonical API base URL env var.
const GATEWAY_API_BASE_ENV: &str = "GATEWAY_API_BASE";
/// Legacy API base URL env var, kept as a back-compatible fallback.
const OTARI_API_BASE_ENV: &str = "OTARI_API_BASE";

/// Canonical API key env var.
const GATEWAY_API_KEY_ENV: &str = "GATEWAY_API_KEY";
/// Legacy API key env var, kept as a back-compatible fallback.
const OTARI_API_KEY_ENV: &str = "OTARI_API_KEY";

/// Default hosted gateway base URL, used in platform mode when no explicit
/// base URL is configured via `Config::api_base` or env.
const HOSTED_API_BASE: &str = "https://api.otari.ai";

/// User-Agent sent on every request. The hosted gateway's edge rejects
/// requests with no User-Agent (HTTP 403), so always identify the client.
const USER_AGENT: &str = concat!("otari-rust/", env!("CARGO_PKG_VERSION"));

/// The User-Agent string this SDK sends, for the generated-core configuration.
pub(crate) fn user_agent() -> &'static str {
    USER_AGENT
}

/// Read the platform token from env: canonical first, then legacy alias.
fn platform_token_from_env() -> Option<String> {
    std::env::var(OTARI_AI_TOKEN_ENV)
        .ok()
        .or_else(|| std::env::var(OTARI_PLATFORM_TOKEN_ENV).ok())
}

/// Read the API base URL from env: canonical first, then legacy alias.
fn api_base_from_env() -> Option<String> {
    std::env::var(GATEWAY_API_BASE_ENV)
        .ok()
        .or_else(|| std::env::var(OTARI_API_BASE_ENV).ok())
}

/// Read the API key from env: canonical first, then legacy alias.
fn api_key_from_env() -> Option<String> {
    std::env::var(GATEWAY_API_KEY_ENV)
        .ok()
        .or_else(|| std::env::var(OTARI_API_KEY_ENV).ok())
}

/// Otari gateway client.
///
/// # Examples
///
/// ```rust,no_run
/// use otari::{completion, Message, CompletionOptions};
///
/// # async fn example() -> otari::Result<()> {
/// // Self-hosted gateway: API key + explicit base URL (Otari-Key header).
/// // For the hosted gateway, set OTARI_AI_TOKEN in the environment instead
/// // and omit the api_key/api_base (defaults to https://api.otari.ai).
/// let options = CompletionOptions::with_api_key("my-gateway-key")
///     .api_base("http://localhost:8000");
///
/// let response = completion(
///     "openai:gpt-4o-mini",
///     vec![Message::user("Hello!")],
///     options,
/// ).await?;
///
/// println!("{}", response.content().unwrap_or_default());
/// # Ok(())
/// # }
/// ```
pub struct Otari {
    client: Client,
    api_base: String,
    platform_mode: bool,
}

impl Otari {
    /// Create a new Otari client from a configuration.
    ///
    /// Auth mode and the base URL are resolved together. In platform mode
    /// (a platform token is available and no explicit API key is set), a
    /// missing base URL defaults to the hosted gateway
    /// (`https://api.otari.ai`). In non-platform mode a base URL is required.
    pub fn from_config(config: Config) -> Result<Self> {
        let platform_token_env = platform_token_from_env();
        let explicit_platform_token = config.extra.get("platform_token").cloned();
        let explicit_platform_mode = config.extra.get("platform_mode").map(|v| v == "true");

        // Resolve auth first so the base-URL decision can agree on whether
        // this is platform mode.
        let (platform_mode, headers) = resolve_auth(
            config.api_key,
            explicit_platform_token,
            explicit_platform_mode,
            platform_token_env,
        )?;

        // Base URL: explicit config, then env (canonical then legacy). In
        // platform mode, fall back to the hosted gateway default. In
        // non-platform mode, a base URL is required.
        let api_base = config
            .api_base
            .or_else(api_base_from_env)
            .or_else(|| platform_mode.then(|| HOSTED_API_BASE.to_string()))
            .ok_or_else(|| {
                OtariError::provider_error(format!(
                    "api_base is required (set via config, {GATEWAY_API_BASE_ENV} env var, \
                     or {OTARI_API_BASE_ENV} legacy env var)"
                ))
            })?
            .trim_end_matches('/')
            .to_string();

        let client = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .map_err(|e| OtariError::provider_error(format!("Failed to build HTTP client: {e}")))?;

        Ok(Self {
            client,
            api_base,
            platform_mode,
        })
    }

    /// Returns `true` if the client is using platform mode authentication.
    pub fn is_platform_mode(&self) -> bool {
        self.platform_mode
    }

    /// Returns the resolved base URL (with any trailing slash trimmed).
    ///
    /// In platform mode with no explicit base URL configured, this is the
    /// hosted gateway default (`https://api.otari.ai`).
    pub fn api_base(&self) -> &str {
        &self.api_base
    }

    /// Build a client for the control-plane (management) endpoints
    /// (keys, users, budgets, pricing, usage).
    ///
    /// Those endpoints authenticate with `Authorization: Bearer <admin/master
    /// key>`, distinct from the inference auth. Pass the gateway master key (or
    /// an admin token); call the ergonomic aliases on the returned
    /// [`crate::control_plane::ControlPlane`] (for example
    /// `cp.keys().create(...)`), or reach the generated functions via
    /// [`crate::control_plane::ControlPlane::config`].
    pub fn control_plane(
        &self,
        admin_key: impl Into<String>,
    ) -> crate::control_plane::ControlPlane {
        // The control-plane endpoints expect `Authorization: Bearer <admin/
        // master key>`. The generated functions read auth from the
        // configuration's `reqwest::Client` default headers (the spec declares
        // no security scheme), so bake the bearer header into a dedicated
        // client rather than relying on `bearer_access_token`.
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if let Ok(val) = HeaderValue::from_str(&format!("Bearer {}", admin_key.into())) {
            headers.insert(AUTHORIZATION, val);
        }
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .build()
            .unwrap_or_default();

        let mut config = crate::control_plane::Configuration::new();
        config.base_path = self.api_base.clone();
        config.user_agent = Some(USER_AGENT.to_string());
        config.client = client;
        crate::control_plane::ControlPlane::new(config)
    }

    /// Build a generated-core [`Configuration`] for the typed inference and
    /// management endpoints, reusing this client's already-authenticated
    /// `reqwest::Client` (per-mode auth header baked into its default headers).
    fn gen_config(&self) -> crate::_client::apis::configuration::Configuration {
        make_configuration(&self.api_base, self.client.clone())
    }

    // ----- Completion operations -----

    /// Create a chat completion.
    pub async fn completion(&self, params: CompletionParams) -> Result<ChatCompletion> {
        let body: GatewayRequest = params.try_into()?;

        let response = self
            .client
            .post(format!("{}/v1/chat/completions", self.api_base))
            .json(&body)
            .send()
            .await?;

        let status = response.status().as_u16();
        if status != 200 {
            return Err(convert_error(response).await);
        }

        Ok(response.json::<GatewayResponse>().await?.into())
    }

    /// Create a streaming chat completion.
    #[allow(clippy::unused_async)]
    pub async fn completion_stream(&self, params: CompletionParams) -> Result<CompletionStream> {
        let model = params.model_id.clone();
        let body = TryInto::<GatewayRequest>::try_into(params)?.stream();

        let request = self
            .client
            .post(format!("{}/v1/chat/completions", self.api_base))
            .json(&body);

        let es = EventSource::new(request).map_err(|e| OtariError::Streaming {
            provider: "otari".into(),
            message: e.to_string().into(),
        })?;

        GatewayStream::new(es, model).try_into()
    }

    // ----- Rerank operations -----

    /// Rerank documents by relevance to a query.
    pub async fn rerank(&self, params: RerankParams) -> Result<RerankResponse> {
        let body = models::rerank::GatewayRerankRequest::from(params);

        let response = self
            .client
            .post(format!("{}/v1/rerank", self.api_base))
            .json(&body)
            .send()
            .await
            .map_err(OtariError::from)?;

        let status = response.status().as_u16();
        if status != 200 {
            return Err(convert_error(response).await);
        }

        response
            .json::<RerankResponse>()
            .await
            .map_err(OtariError::from)
    }

    // ----- Batch operations -----

    /// Create a batch job.
    pub async fn create_batch(&self, params: CreateBatchParams) -> Result<Batch> {
        let url = format!("{}/v1/batches", self.api_base);
        let response = self.client.post(&url).json(&params).send().await?;
        if response.status().as_u16() != 200 {
            return Err(convert_batch_error(response, "/v1/batches").await);
        }
        Ok(response.json::<Batch>().await?)
    }

    /// Retrieve the status of a batch job.
    pub async fn retrieve_batch(&self, batch_id: &str, provider: &str) -> Result<Batch> {
        let url = format!("{}/v1/batches/{}", self.api_base, batch_id);
        let response = self
            .client
            .get(&url)
            .query(&[("provider", provider)])
            .send()
            .await?;
        let path = format!("/v1/batches/{batch_id}");
        if response.status().as_u16() != 200 {
            return Err(convert_batch_error(response, &path).await);
        }
        Ok(response.json::<Batch>().await?)
    }

    /// Cancel a batch job.
    pub async fn cancel_batch(&self, batch_id: &str, provider: &str) -> Result<Batch> {
        let url = format!("{}/v1/batches/{}/cancel", self.api_base, batch_id);
        let response = self
            .client
            .post(&url)
            .query(&[("provider", provider)])
            .send()
            .await?;
        let path = format!("/v1/batches/{batch_id}/cancel");
        if response.status().as_u16() != 200 {
            return Err(convert_batch_error(response, &path).await);
        }
        Ok(response.json::<Batch>().await?)
    }

    /// List batch jobs for a provider.
    pub async fn list_batches(
        &self,
        provider: &str,
        options: ListBatchesOptions,
    ) -> Result<Vec<Batch>> {
        let url = format!("{}/v1/batches", self.api_base);
        let mut query: Vec<(&str, String)> = vec![("provider", provider.to_string())];
        if let Some(after) = &options.after {
            query.push(("after", after.clone()));
        }
        if let Some(limit) = options.limit {
            query.push(("limit", limit.to_string()));
        }
        let response = self.client.get(&url).query(&query).send().await?;
        if response.status().as_u16() != 200 {
            return Err(convert_batch_error(response, "/v1/batches").await);
        }
        #[derive(Deserialize)]
        struct ListResponse {
            data: Vec<Batch>,
        }
        let list_resp: ListResponse = response.json().await?;
        Ok(list_resp.data)
    }

    /// Retrieve the results of a completed batch job.
    pub async fn retrieve_batch_results(
        &self,
        batch_id: &str,
        provider: &str,
    ) -> Result<BatchResult> {
        let url = format!("{}/v1/batches/{}/results", self.api_base, batch_id);
        let response = self
            .client
            .get(&url)
            .query(&[("provider", provider)])
            .send()
            .await?;
        let path = format!("/v1/batches/{batch_id}/results");
        if response.status().as_u16() != 200 {
            return Err(convert_batch_error(response, &path).await);
        }
        Ok(response.json::<BatchResult>().await?)
    }

    // ----- Moderation -----

    /// Call `POST /v1/moderations` on the gateway.
    ///
    /// Auth headers (`Authorization` / `Otari-Key`) are already injected
    /// as default headers on the inner HTTP client.
    ///
    /// When `params.include_raw` is `true`, `?include_raw=true` is appended
    /// to the URL instead of being sent in the JSON body.
    ///
    /// # Errors
    ///
    /// - [`OtariError::Unsupported`] if the gateway reports the chosen
    ///   upstream provider does not support moderation (or multimodal
    ///   moderation input).
    /// - Other [`OtariError`] variants for standard HTTP error mapping,
    ///   transport failures, and deserialization errors.
    pub async fn moderation(&self, params: ModerationParams) -> Result<ModerationResponse> {
        let mut url = format!("{}/v1/moderations", self.api_base);
        if params.include_raw {
            url.push_str("?include_raw=true");
        }

        let body = serde_json::to_value(&params)?;
        let response = self.client.post(&url).json(&body).send().await?;

        if !response.status().is_success() {
            return Err(convert_error(response).await);
        }

        let body_bytes = response.bytes().await?;
        serde_json::from_slice::<ModerationResponse>(&body_bytes).map_err(OtariError::from)
    }

    // ----- Images -----

    /// Generate images from a text prompt (`POST /v1/images/generations`).
    ///
    /// Returns the gateway's OpenAI-compatible image payload as a raw
    /// [`serde_json::Value`] (`{"created": ..., "data": [...]}`). The generated
    /// core models this response as an opaque object, so the parsed JSON is
    /// returned unchanged. This goes through the generated [`images_api`]
    /// function, reusing this client's already-authenticated `reqwest::Client`
    /// (auth headers apply in both modes).
    pub async fn image_generation(
        &self,
        params: ImageGenerationParams,
    ) -> Result<serde_json::Value> {
        let mut request =
            gen_models::ImageGenerationRequest::new(params.model.clone(), params.prompt.clone());
        request.n = params.n.map(Some);
        request.quality = params.quality.map(Some);
        request.response_format = params.response_format.map(Some);
        request.size = params.size.map(Some);
        request.style = params.style.map(Some);
        request.user = params.user.map(Some);

        images_api::create_image_v1_images_generations_post(&self.gen_config(), request)
            .await
            .map_err(map_error)
    }

    // ----- Audio -----

    /// Synthesize speech (text-to-speech), returning raw audio bytes
    /// (`POST /v1/audio/speech`).
    ///
    /// The gateway returns binary audio (`audio/mpeg` by default) with no JSON
    /// response model, so the generated core (which only decodes JSON) cannot
    /// handle it. This posts over the same raw `reqwest::Client` used by the
    /// streaming path, with the per-mode auth header already baked in, and
    /// returns the response body bytes. Non-2xx responses map through the shared
    /// error table.
    pub async fn speech(&self, params: SpeechParams) -> Result<bytes::Bytes> {
        let mut body = serde_json::json!({
            "model": params.model,
            "input": params.input,
            "voice": params.voice,
        });
        let obj = body.as_object_mut().expect("body is an object");
        if let Some(response_format) = params.response_format {
            obj.insert("response_format".to_string(), response_format.into());
        }
        if let Some(speed) = params.speed {
            obj.insert("speed".to_string(), speed.into());
        }
        if let Some(instructions) = params.instructions {
            obj.insert("instructions".to_string(), instructions.into());
        }
        if let Some(user) = params.user {
            obj.insert("user".to_string(), user.into());
        }

        let request = self
            .client
            .post(format!("{}/v1/audio/speech", self.api_base))
            .json(&body);
        let response = self.send_raw(request).await?;
        response.bytes().await.map_err(OtariError::from)
    }

    /// Transcribe audio to text (`POST /v1/audio/transcriptions`).
    ///
    /// `params.file` is uploaded as multipart form data (the `file` part); the
    /// model and other parameters are sent as form fields. The generated core
    /// types the file as a `String`, so it cannot perform this upload; instead
    /// this posts a `reqwest::multipart::Form` over the same authenticated raw
    /// client used by the streaming path.
    ///
    /// Returns the parsed JSON for JSON response formats (the default), or a
    /// JSON string for the `text` / `srt` / `vtt` formats (the gateway returns
    /// those as `text/plain`, surfaced here as a [`serde_json::Value::String`]).
    pub async fn transcription(&self, params: TranscriptionParams) -> Result<serde_json::Value> {
        let file_part = reqwest::multipart::Part::bytes(params.file).file_name(params.filename);
        let mut form = reqwest::multipart::Form::new()
            .text("model", params.model)
            .part("file", file_part);
        if let Some(language) = params.language {
            form = form.text("language", language);
        }
        if let Some(prompt) = params.prompt {
            form = form.text("prompt", prompt);
        }
        if let Some(response_format) = params.response_format {
            form = form.text("response_format", response_format);
        }
        if let Some(temperature) = params.temperature {
            form = form.text("temperature", temperature.to_string());
        }
        if let Some(user) = params.user {
            form = form.text("user", user);
        }

        let request = self
            .client
            .post(format!("{}/v1/audio/transcriptions", self.api_base))
            .multipart(form);
        let response = self.send_raw(request).await?;

        let is_json = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.contains("application/json"));

        let bytes = response.bytes().await?;
        if is_json {
            serde_json::from_slice::<serde_json::Value>(&bytes).map_err(OtariError::from)
        } else {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            Ok(serde_json::Value::String(text))
        }
    }

    /// Send a pre-built raw request, mapping non-2xx responses through the
    /// shared error table (`x-correlation-id` / `retry-after` honored).
    ///
    /// Used by the audio endpoints (binary speech, multipart transcription),
    /// which do not fit the generated JSON core but still reuse the same
    /// authenticated `reqwest::Client` and error mapping as the rest of the SDK.
    async fn send_raw(&self, request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
        let response = request.send().await?;

        let status = response.status().as_u16();
        if (200..300).contains(&status) {
            return Ok(response);
        }

        let correlation_id = response
            .headers()
            .get("x-correlation-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let text = response.text().await.unwrap_or_default();
        Err(map_response(
            status,
            &text,
            correlation_id.as_deref(),
            retry_after.as_deref(),
        ))
    }

    // ----- Generated-core typed endpoints -----
    //
    // These ergonomic methods mirror the Python reference: shape a JSON body,
    // POST it to the gateway, deserialize the response into the OpenAPI-
    // generated typed response model, and map non-2xx responses to a typed
    // `OtariError`. `list_models` (no request body) goes through the generated
    // GET function directly.
    //
    // NOTE (divergence from the Python reference): the Rust generator collapses
    // the inference *request* unions destructively (e.g. `ChatMessageInput.role`
    // accepts only `function`, `EmbeddingRequest.input` / `ModerationRequest.
    // input` become structs that reject a plain string). So unlike Python's
    // `Model.from_dict`, the generated Rust *request* models cannot be built
    // from a natural JSON body. We therefore send the caller's JSON straight to
    // the wire (the gateway is the source of truth for request validation) and
    // only use the generated models for the typed *responses*, which
    // deserialize correctly (verified for chat / embedding / rerank /
    // moderation / models). `/messages` and `/responses` have no usable typed
    // response model, so they return a raw `serde_json::Value`.

    /// Create a chat completion through the generated typed core.
    ///
    /// Returns the generated [`gen_models::ChatCompletion`]. Use
    /// [`Self::completion`] for the hand-written ergonomic response type, or
    /// [`Self::completion_stream`] for streaming.
    ///
    /// `body` is the request payload (`model`, `messages`, and any optional
    /// fields such as `temperature`, `tools`, `guardrails`).
    pub async fn chat(&self, body: serde_json::Value) -> Result<gen_models::ChatCompletion> {
        self.post_typed("/v1/chat/completions", &body).await
    }

    /// Create a response via the OpenAI-style Responses API.
    ///
    /// The gateway's responses payload has no single typed model, so this
    /// returns the raw [`serde_json::Value`]. For streaming responses, use
    /// [`Self::response_stream`].
    pub async fn response(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        self.post_typed("/v1/responses", &body).await
    }

    /// Create an Anthropic-style message via the gateway `/messages` endpoint.
    ///
    /// This endpoint has no OpenAI-SDK seam and was previously missing from the
    /// SDK. Returns the raw [`serde_json::Value`] response. For streaming, use
    /// [`Self::message_stream`].
    ///
    /// `body` must include `model`, `messages`, and `max_tokens` (required by
    /// `/messages`), plus any optional fields (`system`, `temperature`,
    /// `tools`, `thinking`, ...).
    ///
    /// Returns a raw `Value`: the generated `MessageResponse` model collapses
    /// the Anthropic content-block union into a single over-constrained struct
    /// that cannot deserialize a real response (it requires `text`,
    /// `signature`, `thinking`, `data`, ... all at once, and types `model` as
    /// an empty struct rather than a string).
    pub async fn message(&self, body: serde_json::Value) -> Result<serde_json::Value> {
        self.post_typed("/v1/messages", &body).await
    }

    /// Count input tokens for an Anthropic-style message request via the
    /// gateway `/v1/messages/count_tokens` endpoint.
    ///
    /// Counts the tokens a `/messages` request would consume without generating
    /// a response, so `max_tokens` is not part of the body. Returns the
    /// generated typed [`gen_models::CountTokensResponse`], whose
    /// `input_tokens` field deserializes cleanly (unlike the `/messages`
    /// response).
    ///
    /// `body` must include `model` and `messages`, plus any optional fields
    /// (`system`, `tools`, `tool_choice`, `thinking`, ...).
    pub async fn count_tokens(
        &self,
        body: serde_json::Value,
    ) -> Result<gen_models::CountTokensResponse> {
        self.post_typed("/v1/messages/count_tokens", &body).await
    }

    /// Create embeddings for the given input through the generated typed core.
    pub async fn embedding(
        &self,
        body: serde_json::Value,
    ) -> Result<gen_models::CreateEmbeddingResponse> {
        self.post_typed("/v1/embeddings", &body).await
    }

    /// Classify text against the gateway moderation endpoint, returning the
    /// generated typed [`gen_models::ModerationResponse`].
    ///
    /// This is the generated-core counterpart to [`Self::moderation`] (which
    /// returns the hand-written response type). `include_raw` maps to the
    /// `?include_raw=true` query parameter.
    pub async fn moderate(
        &self,
        body: serde_json::Value,
        include_raw: bool,
    ) -> Result<gen_models::ModerationResponse> {
        let path = if include_raw {
            "/v1/moderations?include_raw=true"
        } else {
            "/v1/moderations"
        };
        self.post_typed(path, &body).await
    }

    /// Rerank documents by relevance, returning the generated typed
    /// [`gen_models::RerankResponse`].
    ///
    /// This is the generated-core counterpart to [`Self::rerank`] (which
    /// returns the hand-written response type).
    pub async fn rerank_typed(
        &self,
        body: serde_json::Value,
    ) -> Result<gen_models::RerankResponse> {
        self.post_typed("/v1/rerank", &body).await
    }

    /// List available models from the gateway.
    ///
    /// Pass `provider` to scope the list to one provider. This goes through the
    /// generated GET function (no request body to shape).
    pub async fn list_models(
        &self,
        provider: Option<&str>,
    ) -> Result<Vec<gen_models::ModelObject>> {
        models_api::list_models_v1_models_get(&self.gen_config(), provider)
            .await
            .map(|resp| resp.data)
            .map_err(map_error)
    }

    /// POST `body` to `path`, deserialize the 2xx response into the typed
    /// generated model `R`, and map non-2xx responses through the shared error
    /// table (`x-correlation-id` / `retry-after` honored). `R = serde_json::
    /// Value` yields the raw response for endpoints with no usable typed model.
    async fn post_typed<R: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<R> {
        let response = self
            .client
            .post(format!("{}{path}", self.api_base))
            .json(body)
            .send()
            .await?;

        let status = response.status().as_u16();
        let correlation_id = response
            .headers()
            .get("x-correlation-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if !(200..300).contains(&status) {
            let text = response.text().await.unwrap_or_default();
            return Err(crate::core::map_response(
                status,
                &text,
                correlation_id.as_deref(),
                retry_after.as_deref(),
            ));
        }

        let bytes = response.bytes().await?;
        serde_json::from_slice::<R>(&bytes).map_err(OtariError::from)
    }

    /// Stream a Responses-API response as raw JSON events.
    ///
    /// The generated core can't stream, so this uses the same hand-written
    /// `reqwest-eventsource` shim as [`Self::completion_stream`], yielding the
    /// raw parsed event values (the responses event stream has no single typed
    /// chunk model). `body` should NOT set `stream`; it is forced on here.
    #[allow(clippy::unused_async)]
    pub async fn response_stream(
        &self,
        body: serde_json::Value,
    ) -> Result<crate::types::RawValueStream> {
        self.raw_stream("/v1/responses", body)
    }

    /// Stream an Anthropic-style `/messages` response as raw JSON events.
    ///
    /// Like [`Self::response_stream`], this uses the `reqwest-eventsource` shim
    /// and yields raw parsed event values (the messages event stream has no
    /// single typed chunk model). `body` must include `max_tokens`.
    #[allow(clippy::unused_async)]
    pub async fn message_stream(
        &self,
        body: serde_json::Value,
    ) -> Result<crate::types::RawValueStream> {
        self.raw_stream("/v1/messages", body)
    }

    /// Open a raw SSE stream against `path`, forcing `stream: true`.
    fn raw_stream(
        &self,
        path: &str,
        mut body: serde_json::Value,
    ) -> Result<crate::types::RawValueStream> {
        if let Some(obj) = body.as_object_mut() {
            obj.insert("stream".to_string(), serde_json::Value::Bool(true));
        }
        // `reqwest-eventsource` sets `Accept: text/event-stream` on the request
        // itself, so we don't add it here (doing so would duplicate the header).
        let request = self
            .client
            .post(format!("{}{path}", self.api_base))
            .json(&body);

        let es = EventSource::new(request).map_err(|e| OtariError::Streaming {
            provider: "otari".into(),
            message: e.to_string().into(),
        })?;

        Ok(models::stream::raw_value_stream(es))
    }
}

/// Resolve auth mode and build the appropriate HTTP headers.
///
/// 1. Explicit `platform_mode=true` -> platform mode, needs a token
/// 2. Platform token env set (`OTARI_AI_TOKEN` / `OTARI_PLATFORM_TOKEN`) and
///    no explicit api_key -> auto-detect platform mode
/// 3. Otherwise -> non-platform mode with optional Otari-Key header
fn resolve_auth(
    api_key: Option<String>,
    platform_token: Option<String>,
    platform_mode: Option<bool>,
    platform_token_env: Option<String>,
) -> Result<(bool, HeaderMap)> {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    // Explicit platform_mode=true
    if platform_mode == Some(true) {
        let token = platform_token
            .or(api_key)
            .or(platform_token_env)
            .ok_or_else(|| OtariError::MissingApiKey {
                provider: "otari".into(),
                env_var: OTARI_AI_TOKEN_ENV.into(),
            })?;

        let val = format!("Bearer {token}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&val)
                .map_err(|e| OtariError::provider_error(format!("Invalid platform token: {e}")))?,
        );
        return Ok((true, headers));
    }

    // Auto-detect: OTARI_PLATFORM_TOKEN set and no explicit api_key
    if platform_mode.is_none() && api_key.is_none() {
        if let Some(token) = platform_token.or(platform_token_env) {
            let val = format!("Bearer {token}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&val).map_err(|e| {
                    OtariError::provider_error(format!("Invalid platform token: {e}"))
                })?,
            );
            return Ok((true, headers));
        }
    }

    // Non-platform mode
    let key = api_key.or_else(api_key_from_env).unwrap_or_default();

    if !key.is_empty() {
        let val = format!("Bearer {key}");
        headers.insert(
            OTARI_HEADER_NAME,
            HeaderValue::from_str(&val)
                .map_err(|e| OtariError::provider_error(format!("Invalid API key: {e}")))?,
        );
    }

    Ok((false, headers))
}

/// Convert an HTTP error response to a typed `OtariError`.
///
/// Extracts `x-correlation-id` and `retry-after` headers and includes
/// them in the error message for debugging.
async fn convert_error(response: reqwest::Response) -> OtariError {
    let status = response.status().as_u16();
    let correlation_id = response
        .headers()
        .get("x-correlation-id")
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let retry_after = response
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .map(String::from);

    let body = response.text().await.unwrap_or_default();

    let message = extract_error_message(&body).unwrap_or_else(|| {
        if body.is_empty() {
            format!("HTTP {status}")
        } else {
            body.clone()
        }
    });

    let detail = match &correlation_id {
        Some(cid) => format!("{message} (correlation_id={cid})"),
        None => message,
    };

    // Detect the locked "unsupported moderation" phrasing emitted by the
    // gateway and map it to a typed `Unsupported` error. The substring
    // check is the real signal; provider-name extraction is best-effort.
    //
    // Accepted phrasings (from the gateway's locked copy):
    //   - "Provider <name> does not support moderation"
    //   - "Provider <name> does not support multimodal moderation input"
    if status == 400 && detail.contains("does not support") && detail.contains("moderation") {
        let provider = parse_unsupported_provider(&detail).unwrap_or_else(|| "unknown".to_string());
        let operation = if detail.contains("multimodal") {
            "multimodal_moderation"
        } else {
            "moderation"
        };
        return OtariError::unsupported_dynamic(provider, operation);
    }

    let detail_with_retry = match &retry_after {
        Some(ra) => format!("{detail} (retry_after={ra})"),
        None => detail,
    };

    match status {
        401 | 403 => OtariError::authentication(detail_with_retry),
        402 => OtariError::provider_error(format!("Insufficient funds: {detail_with_retry}")),
        404 => OtariError::model_not_found(detail_with_retry),
        429 => OtariError::rate_limit(detail_with_retry),
        502 => OtariError::provider_error(format!("Upstream provider error: {detail_with_retry}")),
        504 => OtariError::provider_error(format!("Gateway timeout: {detail_with_retry}")),
        _ => OtariError::provider_error(format!("HTTP {status}: {detail_with_retry}")),
    }
}

/// Extract a message from an OpenAI-style or FastAPI-style error body.
///
/// Recognizes three shapes:
/// - `{"error": {"message": "..."}}` (OpenAI)
/// - `{"error": "..."}`
/// - `{"detail": "..."}` (FastAPI / gateway)
fn extract_error_message(body: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(body).ok()?;
    if let Some(err) = val.get("error") {
        if let Some(msg) = err.get("message").and_then(|m| m.as_str()) {
            return Some(msg.to_string());
        }
        if let Some(s) = err.as_str() {
            return Some(s.to_string());
        }
    }
    if let Some(detail) = val.get("detail").and_then(|d| d.as_str()) {
        return Some(detail.to_string());
    }
    None
}

/// Parse `"Provider <name> does not support [multimodal] moderation..."`
/// into just `<name>`. Returns `None` if the phrasing does not start with
/// `"Provider "`.
fn parse_unsupported_provider(detail: &str) -> Option<String> {
    let after = detail.strip_prefix("Provider ")?;
    let before_does = after.split(" does not").next()?;
    if before_does.is_empty() {
        None
    } else {
        Some(before_does.to_string())
    }
}

/// Convert an HTTP error response from a batch endpoint to a typed `OtariError`.
///
/// Handles batch-specific status codes (409, 404 on batch paths) before
/// falling through to the generic `convert_error` logic.
async fn convert_batch_error(response: reqwest::Response, path: &str) -> OtariError {
    let status = response.status().as_u16();

    // For 409 and batch-404 we need the body *before* delegating, because
    // `convert_error` consumes the response.
    if status == 409 || (status == 404 && path.contains("/v1/batches")) {
        let correlation_id = response
            .headers()
            .get("x-correlation-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let body = response.text().await.unwrap_or_default();
        let message = extract_error_message(&body).unwrap_or_else(|| {
            if body.is_empty() {
                format!("HTTP {status}")
            } else {
                body.clone()
            }
        });

        let detail = match &correlation_id {
            Some(cid) => format!("{message} (correlation_id={cid})"),
            None => message,
        };

        return match status {
            409 => {
                let batch_id = extract_batch_id_from_detail(&detail)
                    .unwrap_or_default();
                let batch_status =
                    extract_batch_status_from_detail(&detail).unwrap_or("unknown".to_string());
                OtariError::BatchNotComplete {
                    batch_id: batch_id.into(),
                    status: batch_status.into(),
                    provider: "otari".into(),
                }
            }
            404 => OtariError::Provider {
                message: format!(
                    "This gateway does not support batch operations. Upgrade your gateway. ({detail})"
                )
                .into(),
                provider: "otari".into(),
            },
            _ => unreachable!(),
        };
    }

    // Fall through to the generic error converter for all other status codes.
    convert_error(response).await
}

/// Extract the batch ID from a gateway 409 error detail string.
///
/// The gateway sends messages like:
/// `"Batch 'batch_abc123' is not yet complete (status: in_progress). ..."`
///
/// This function looks for the pattern `Batch '<id>'` (case-insensitive on
/// the leading `B`) and returns the quoted value.
fn extract_batch_id_from_detail(detail: &str) -> Option<String> {
    // Look for "atch '" which covers both "Batch '" and "batch '"
    let marker = "atch '";
    let start = detail.find(marker)?;
    let value_start = start + marker.len();
    let rest = &detail[value_start..];
    let end = rest.find('\'')?;
    let value = &rest[..end];
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

/// Extract the batch status from a gateway 409 error detail string.
///
/// The gateway sends messages like:
/// `"Batch 'batch_abc123' is not yet complete (status: in_progress). ..."`
///
/// This function looks for the pattern `status: <word>` and returns the
/// status value.
fn extract_batch_status_from_detail(detail: &str) -> Option<String> {
    let marker = "status: ";
    let start = detail.find(marker)?;
    let value_start = start + marker.len();
    let rest = &detail[value_start..];
    let end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    let value = &rest[..end];
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(test)]
mod env_tests {
    use super::*;
    use std::sync::Mutex;

    // Env-var tests mutate global process state, so they must not run
    // concurrently. This SDK has no `serial_test` dependency, so we
    // serialize with a module-local mutex instead.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// All env vars that can influence client construction. Cleared before
    /// and after each test so cases never leak into one another.
    const ALL_ENV_VARS: &[&str] = &[
        OTARI_AI_TOKEN_ENV,
        OTARI_PLATFORM_TOKEN_ENV,
        GATEWAY_API_BASE_ENV,
        OTARI_API_BASE_ENV,
        GATEWAY_API_KEY_ENV,
        OTARI_API_KEY_ENV,
    ];

    fn clear_env() {
        for var in ALL_ENV_VARS {
            std::env::remove_var(var);
        }
    }

    /// Run `body` with a clean env under the serialization lock, restoring a
    /// clean env afterward even on panic.
    fn with_clean_env<T>(body: impl FnOnce() -> T) -> T {
        let guard = ENV_LOCK.lock().unwrap_or_else(|p| p.into_inner());
        clear_env();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        clear_env();
        drop(guard);
        match result {
            Ok(v) => v,
            Err(p) => std::panic::resume_unwind(p),
        }
    }

    #[test]
    fn platform_token_env_defaults_to_hosted_base() {
        with_clean_env(|| {
            std::env::set_var(OTARI_AI_TOKEN_ENV, "tk_hosted");
            let gw = Otari::from_config(Config::default()).unwrap();
            assert!(gw.is_platform_mode());
            assert_eq!(gw.api_base(), HOSTED_API_BASE);
            // The base must produce correct request URLs with /v1 appended.
            assert_eq!(
                format!("{}/v1/chat/completions", gw.api_base()),
                "https://api.otari.ai/v1/chat/completions"
            );
        });
    }

    #[test]
    fn non_platform_no_credentials_no_base_errors() {
        with_clean_env(|| {
            let result = Otari::from_config(Config::default());
            assert!(result.is_err());
        });
    }

    #[test]
    fn otari_ai_token_takes_precedence_over_legacy() {
        with_clean_env(|| {
            std::env::set_var(OTARI_AI_TOKEN_ENV, "tk_canonical");
            std::env::set_var(OTARI_PLATFORM_TOKEN_ENV, "tk_legacy");
            // canonical wins
            assert_eq!(platform_token_from_env().as_deref(), Some("tk_canonical"));
            let gw = Otari::from_config(Config::default()).unwrap();
            assert!(gw.is_platform_mode());
            assert_eq!(gw.api_base(), HOSTED_API_BASE);
        });
    }

    #[test]
    fn legacy_platform_token_still_works() {
        with_clean_env(|| {
            std::env::set_var(OTARI_PLATFORM_TOKEN_ENV, "tk_legacy");
            let gw = Otari::from_config(Config::default()).unwrap();
            assert!(gw.is_platform_mode());
            assert_eq!(gw.api_base(), HOSTED_API_BASE);
        });
    }

    #[test]
    fn canonical_gateway_api_base_is_read() {
        with_clean_env(|| {
            std::env::set_var(GATEWAY_API_BASE_ENV, "http://canonical:8000");
            // Non-platform mode, but base resolved from canonical env var.
            let gw = Otari::from_config(Config::default()).unwrap();
            assert!(!gw.is_platform_mode());
            assert_eq!(gw.api_base(), "http://canonical:8000");
        });
    }

    #[test]
    fn canonical_gateway_api_base_takes_precedence_over_legacy() {
        with_clean_env(|| {
            std::env::set_var(GATEWAY_API_BASE_ENV, "http://canonical:8000");
            std::env::set_var(OTARI_API_BASE_ENV, "http://legacy:8000");
            assert_eq!(
                api_base_from_env().as_deref(),
                Some("http://canonical:8000")
            );
        });
    }

    #[test]
    fn legacy_otari_api_base_still_works() {
        with_clean_env(|| {
            std::env::set_var(OTARI_API_BASE_ENV, "http://legacy:8000");
            let gw = Otari::from_config(Config::default()).unwrap();
            assert_eq!(gw.api_base(), "http://legacy:8000");
        });
    }

    #[test]
    fn canonical_gateway_api_key_takes_precedence_over_legacy() {
        with_clean_env(|| {
            std::env::set_var(GATEWAY_API_KEY_ENV, "canonical-key");
            std::env::set_var(OTARI_API_KEY_ENV, "legacy-key");
            assert_eq!(api_key_from_env().as_deref(), Some("canonical-key"));
        });
    }

    #[test]
    fn legacy_otari_api_key_still_works() {
        with_clean_env(|| {
            std::env::set_var(OTARI_API_KEY_ENV, "legacy-key");
            // API key present -> non-platform mode; base from config.
            let config = Config {
                api_key: None,
                api_base: Some("http://example.com".to_string()),
                extra: Default::default(),
            };
            let gw = Otari::from_config(config).unwrap();
            assert!(!gw.is_platform_mode());
        });
    }

    #[test]
    fn explicit_base_overrides_hosted_default() {
        with_clean_env(|| {
            std::env::set_var(OTARI_AI_TOKEN_ENV, "tk_hosted");
            let config = Config {
                api_key: None,
                api_base: Some("http://explicit:9000".to_string()),
                extra: Default::default(),
            };
            let gw = Otari::from_config(config).unwrap();
            assert!(gw.is_platform_mode());
            assert_eq!(gw.api_base(), "http://explicit:9000");
        });
    }

    #[test]
    fn config_extra_platform_token_defaults_to_hosted_base() {
        with_clean_env(|| {
            let config = Config {
                api_key: None,
                api_base: None,
                extra: [("platform_token".to_string(), "tk_extra".to_string())].into(),
            };
            let gw = Otari::from_config(config).unwrap();
            assert!(gw.is_platform_mode());
            assert_eq!(gw.api_base(), HOSTED_API_BASE);
        });
    }
}
