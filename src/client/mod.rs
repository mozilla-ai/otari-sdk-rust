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
use crate::error::{OtariError, Result};
use crate::types::{
    Batch, BatchResult, ChatCompletion, CompletionParams, CompletionStream, CreateBatchParams,
    ListBatchesOptions, ModerationParams, ModerationResponse, RerankParams, RerankResponse,
};

mod models;

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

    /// Build a configured client for the control-plane (management) endpoints
    /// (keys, users, budgets, pricing, usage).
    ///
    /// Those endpoints authenticate with `Authorization: Bearer <admin/master
    /// key>`, distinct from the inference auth. Pass the gateway master key (or
    /// an admin token); use the returned configuration with the generated
    /// functions under [`crate::control_plane`].
    pub fn control_plane(
        &self,
        admin_key: impl Into<String>,
    ) -> crate::control_plane::Configuration {
        let mut config = crate::control_plane::Configuration::new();
        config.base_path = self.api_base.clone();
        config.bearer_access_token = Some(admin_key.into());
        config
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
