//! Gateway provider implementation.
//!
//! Connects to an [any-llm gateway](https://github.com/mozilla-ai/any-llm)
//! server, which exposes an OpenAI-compatible API that proxies to multiple
//! upstream LLM providers.
//!
//! # Auth modes
//!
//! - **Platform mode**: uses `Authorization: Bearer <token>`. Activated by
//!   setting `GATEWAY_PLATFORM_TOKEN` env var, or by passing
//!   `platform_token` in `ProviderConfig::extra`.
//! - **Non-platform mode**: uses the `AnyLLM-Key: Bearer <key>` header.
//!   The key is optional (the gateway may allow unauthenticated access).

use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use reqwest_eventsource::EventSource;

use serde::Deserialize;

use crate::error::{AnyLLMError, Result};
use crate::provider::{CompletionStream, Provider, ProviderConfig};
use crate::types::{
    Batch, BatchResult, ChatCompletion, CompletionParams, CreateBatchParams, ListBatchesOptions,
};

mod models;

use models::request::GatewayRequest;
use models::response::GatewayResponse;
use models::stream::GatewayStream;

const GATEWAY_HEADER_NAME: &str = "AnyLLM-Key";
const GATEWAY_PLATFORM_TOKEN_ENV: &str = "GATEWAY_PLATFORM_TOKEN";
const GATEWAY_API_BASE_ENV: &str = "GATEWAY_API_BASE";

/// Gateway provider.
///
/// # Examples
///
/// ```rust,no_run
/// use any_llm::{completion, Message, CompletionOptions, providers::Gateway};
///
/// # async fn example() -> any_llm::Result<()> {
/// let options = CompletionOptions::with_api_key("tk_my_platform_token")
///     .api_base("http://localhost:8000");
///
/// let response = completion::<Gateway>(
///     "openai:gpt-4o-mini",
///     vec![Message::user("Hello!")],
///     options,
/// ).await?;
///
/// println!("{}", response.content().unwrap_or_default());
/// # Ok(())
/// # }
/// ```
pub struct Gateway {
    client: Client,
    api_base: String,
    platform_mode: bool,
}

impl Gateway {
    /// Returns `true` if the client is using platform mode authentication.
    pub fn is_platform_mode(&self) -> bool {
        self.platform_mode
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
}

impl Provider for Gateway {
    const NAME: &'static str = "gateway";
    const ENV_VAR: &'static str = "GATEWAY_API_KEY";
    const DOCS_URL: &'static str = "https://github.com/mozilla-ai/any-llm";

    // The gateway proxies to any backend, so all features are nominally supported.
    const SUPPORTS_STREAMING: bool = true;
    const SUPPORTS_TOOLS: bool = true;
    const SUPPORTS_IMAGES: bool = true;
    const SUPPORTS_REASONING: bool = true;
    const SUPPORTS_PDF: bool = true;

    fn from_config(config: ProviderConfig) -> Result<Self> {
        let api_base = config
            .api_base
            .or_else(|| std::env::var(GATEWAY_API_BASE_ENV).ok())
            .ok_or_else(|| {
                AnyLLMError::provider_error::<Self>(format!(
                    "api_base is required (set via config or {GATEWAY_API_BASE_ENV} env var)"
                ))
            })?
            .trim_end_matches('/')
            .to_string();

        let platform_token_env = std::env::var(GATEWAY_PLATFORM_TOKEN_ENV).ok();
        let explicit_platform_token = config.extra.get("platform_token").cloned();
        let explicit_platform_mode = config.extra.get("platform_mode").map(|v| v == "true");

        let (platform_mode, headers) = resolve_auth(
            config.api_key,
            explicit_platform_token,
            explicit_platform_mode,
            platform_token_env,
        )?;

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| {
                AnyLLMError::provider_error::<Self>(format!("Failed to build HTTP client: {e}"))
            })?;

        Ok(Self {
            client,
            api_base,
            platform_mode,
        })
    }

    async fn completion_fn(&self, params: CompletionParams) -> Result<ChatCompletion> {
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

    async fn completion_stream_fn(&self, params: CompletionParams) -> Result<CompletionStream> {
        let model = params.model_id.clone();
        let body = TryInto::<GatewayRequest>::try_into(params)?.stream();

        let request = self
            .client
            .post(format!("{}/v1/chat/completions", self.api_base))
            .json(&body);

        let es = EventSource::new(request).map_err(|e| AnyLLMError::Streaming {
            provider: Self::NAME.into(),
            message: e.to_string().into(),
        })?;

        GatewayStream::new(es, model).try_into()
    }
}

/// Resolve auth mode and build the appropriate HTTP headers.
///
/// This mirrors the Python `GatewayProvider.__init__` logic:
/// 1. Explicit `platform_mode=true` -> platform mode, needs a token
/// 2. `GATEWAY_PLATFORM_TOKEN` set + no explicit api_key -> auto-detect platform
/// 3. Otherwise -> non-platform mode with optional AnyLLM-Key header
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
            .ok_or_else(|| AnyLLMError::MissingApiKey {
                provider: "gateway".into(),
                env_var: GATEWAY_PLATFORM_TOKEN_ENV.into(),
            })?;

        let val = format!("Bearer {token}");
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&val).map_err(|e| {
                AnyLLMError::provider_error::<Gateway>(format!("Invalid platform token: {e}"))
            })?,
        );
        return Ok((true, headers));
    }

    // Auto-detect: GATEWAY_PLATFORM_TOKEN set and no explicit api_key
    if platform_mode.is_none() && api_key.is_none() {
        if let Some(token) = platform_token.or(platform_token_env) {
            let val = format!("Bearer {token}");
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&val).map_err(|e| {
                    AnyLLMError::provider_error::<Gateway>(format!("Invalid platform token: {e}"))
                })?,
            );
            return Ok((true, headers));
        }
    }

    // Non-platform mode
    let key = api_key
        .or_else(|| std::env::var("GATEWAY_API_KEY").ok())
        .unwrap_or_default();

    if !key.is_empty() {
        let val = format!("Bearer {key}");
        headers.insert(
            GATEWAY_HEADER_NAME,
            HeaderValue::from_str(&val).map_err(|e| {
                AnyLLMError::provider_error::<Gateway>(format!("Invalid API key: {e}"))
            })?,
        );
    }

    Ok((false, headers))
}

/// Convert an HTTP error response to a typed `AnyLLMError`.
///
/// Extracts `x-correlation-id` and `retry-after` headers and includes
/// them in the error message for debugging.
async fn convert_error(response: reqwest::Response) -> AnyLLMError {
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

    let detail_with_retry = match &retry_after {
        Some(ra) => format!("{detail} (retry_after={ra})"),
        None => detail,
    };

    match status {
        401 | 403 => AnyLLMError::authentication::<Gateway>(detail_with_retry),
        402 => AnyLLMError::provider_error::<Gateway>(format!(
            "Insufficient funds: {detail_with_retry}"
        )),
        404 => AnyLLMError::model_not_found::<Gateway>(detail_with_retry),
        429 => AnyLLMError::rate_limit::<Gateway>(detail_with_retry),
        502 => AnyLLMError::provider_error::<Gateway>(format!(
            "Upstream provider error: {detail_with_retry}"
        )),
        504 => {
            AnyLLMError::provider_error::<Gateway>(format!("Gateway timeout: {detail_with_retry}"))
        }
        _ => AnyLLMError::provider_error::<Gateway>(format!("HTTP {status}: {detail_with_retry}")),
    }
}

/// Extract a message from an OpenAI-style error JSON body.
fn extract_error_message(body: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(body).ok()?;
    let err = val.get("error")?;
    if let Some(msg) = err.get("message").and_then(|m| m.as_str()) {
        return Some(msg.to_string());
    }
    err.as_str().map(String::from)
}

/// Convert an HTTP error response from a batch endpoint to a typed `AnyLLMError`.
///
/// Handles batch-specific status codes (409, 404 on batch paths) before
/// falling through to the generic `convert_error` logic.
async fn convert_batch_error(response: reqwest::Response, path: &str) -> AnyLLMError {
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
                AnyLLMError::BatchNotComplete {
                    batch_id: batch_id.into(),
                    status: batch_status.into(),
                    provider: Gateway::NAME.into(),
                }
            }
            404 => AnyLLMError::Provider {
                message: format!(
                    "This gateway does not support batch operations. Upgrade your gateway. ({detail})"
                )
                .into(),
                provider: Gateway::NAME.into(),
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
