//! Shared glue between the ergonomic [`crate::Otari`] shell and the
//! OpenAPI-generated typed core in the [`otari_client`] crate.
//!
//! Option C (mirroring the Python reference): non-streaming calls go through
//! the generated per-endpoint API functions (returning typed models such as
//! [`otari_client::models::ChatCompletion`]); streaming goes through the
//! hand-written SSE shim (see [`crate::client::models::stream`]); the
//! generated [`otari_client::apis::Error`] is mapped to the SDK's typed
//! [`OtariError`] hierarchy here, in both auth modes.
//!
//! The generated functions read auth from `Configuration.client` (a
//! `reqwest::Client`), not from `Configuration.bearer_access_token` (the spec
//! declares no security scheme). So per-mode auth headers are baked into the
//! reqwest client the [`crate::Otari`] shell already constructs, and reused
//! both for the generated core and for the streaming shim.

use otari_client::apis::configuration::Configuration;

use crate::error::OtariError;

/// Locked phrasing the gateway uses to signal that the selected provider does
/// not support a moderation request. Mirrors the Python reference's regex.
const UNSUPPORTED_MODERATION_MARKER: &str = "does not support";

/// Build a generated-core [`Configuration`] for the gateway root, reusing the
/// shell's already-authenticated `reqwest::Client`.
///
/// The generated operation paths already include the `/v1` prefix, so
/// `base_path` is the gateway root (no trailing `/v1`).
pub(crate) fn make_configuration(gateway_root: &str, client: reqwest::Client) -> Configuration {
    Configuration {
        base_path: gateway_root.to_string(),
        user_agent: Some(crate::client::user_agent().to_string()),
        client,
        basic_auth: None,
        oauth_access_token: None,
        bearer_access_token: None,
        api_key: None,
    }
}

/// Map a generated `Error<T>` to the SDK's typed [`OtariError`].
///
/// Mirrors the Python reference's status -> error table:
///   401/403 -> auth, 402 -> insufficient funds, 404 -> model-not-found,
///   409 -> batch-not-complete, 429 -> rate-limit (+ retry-after),
///   504 -> gateway-timeout, 502/5xx -> upstream-provider, the locked
///   moderation-capability 400 -> unsupported (surfaced in both modes),
///   else -> generic provider error.
///
/// Transport / deserialization failures (the non-`ResponseError` variants)
/// map to the corresponding [`OtariError`] transport variants.
pub(crate) fn map_error<T>(error: otari_client::apis::Error<T>) -> OtariError {
    use otari_client::apis::Error;

    match error {
        Error::Reqwest(e) => OtariError::from(e),
        Error::Serde(e) => OtariError::from(e),
        Error::Io(e) => OtariError::provider_error(format!("IO error: {e}")),
        Error::ResponseError(rc) => map_response(rc.status.as_u16(), &rc.content, None, None),
    }
}

/// Map a status + raw body (+ optional correlation-id / retry-after) to a
/// typed [`OtariError`]. Shared by the generated-core path and the streaming
/// shim's failed-response path so both auth modes get one mapping table.
pub(crate) fn map_response(
    status: u16,
    body: &str,
    correlation_id: Option<&str>,
    retry_after: Option<&str>,
) -> OtariError {
    let message = extract_detail(body).unwrap_or_else(|| {
        if body.is_empty() {
            format!("HTTP {status}")
        } else {
            body.to_string()
        }
    });

    let detail = match correlation_id {
        Some(cid) => format!("{message} (correlation_id={cid})"),
        None => message,
    };

    // Unsupported-capability (moderation) is surfaced regardless of mode.
    if status == 400
        && detail.contains(UNSUPPORTED_MODERATION_MARKER)
        && detail.contains("moderation")
    {
        let provider = parse_unsupported_provider(&detail).unwrap_or_else(|| "unknown".to_string());
        let capability = if detail.contains("multimodal") {
            "multimodal_moderation"
        } else {
            "moderation"
        };
        return OtariError::unsupported_dynamic(provider, capability);
    }

    let detail_with_retry = match retry_after {
        Some(ra) => format!("{detail} (retry_after={ra})"),
        None => detail,
    };

    match status {
        401 | 403 => OtariError::authentication(detail_with_retry),
        402 => OtariError::provider_error(format!("Insufficient funds: {detail_with_retry}")),
        404 => OtariError::model_not_found(detail_with_retry),
        409 => {
            let batch_id = extract_batch_id(&detail_with_retry).unwrap_or_default();
            let batch_status =
                extract_batch_status(&detail_with_retry).unwrap_or_else(|| "unknown".to_string());
            OtariError::BatchNotComplete {
                batch_id: batch_id.into(),
                status: batch_status.into(),
                provider: "otari".into(),
            }
        }
        429 => OtariError::rate_limit(detail_with_retry),
        504 => OtariError::provider_error(format!("Gateway timeout: {detail_with_retry}")),
        500..=599 => {
            OtariError::provider_error(format!("Upstream provider error: {detail_with_retry}"))
        }
        _ => OtariError::provider_error(format!("HTTP {status}: {detail_with_retry}")),
    }
}

/// Pull the gateway's human-readable detail from an error body.
///
/// Recognizes the FastAPI / gateway `{"detail": "..."}` shape and the
/// OpenAI-style `{"error": {"message": "..."}}` / `{"error": "..."}` shapes.
fn extract_detail(body: &str) -> Option<String> {
    let val: serde_json::Value = serde_json::from_str(body).ok()?;
    if let Some(detail) = val.get("detail") {
        if let Some(s) = detail.as_str() {
            return Some(s.to_string());
        }
        return Some(detail.to_string());
    }
    if let Some(err) = val.get("error") {
        if let Some(msg) = err.get("message").and_then(serde_json::Value::as_str) {
            return Some(msg.to_string());
        }
        if let Some(s) = err.as_str() {
            return Some(s.to_string());
        }
    }
    if let Some(msg) = val.get("message").and_then(serde_json::Value::as_str) {
        return Some(msg.to_string());
    }
    None
}

/// Parse `"Provider <name> does not support ..."` into `<name>`.
fn parse_unsupported_provider(detail: &str) -> Option<String> {
    let after = detail.strip_prefix("Provider ")?;
    let before = after.split(" does not").next()?;
    if before.is_empty() {
        None
    } else {
        Some(before.to_string())
    }
}

/// Extract `<id>` from `"Batch '<id>' ..."` (case-insensitive on the leading B).
fn extract_batch_id(detail: &str) -> Option<String> {
    let marker = "atch '";
    let start = detail.find(marker)?;
    let rest = &detail[start + marker.len()..];
    let end = rest.find('\'')?;
    let value = &rest[..end];
    (!value.is_empty()).then(|| value.to_string())
}

/// Extract `<status>` from `"... (status: <status>) ..."`.
fn extract_batch_status(detail: &str) -> Option<String> {
    let marker = "status: ";
    let start = detail.find(marker)?;
    let rest = &detail[start + marker.len()..];
    let end = rest
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(rest.len());
    let value = &rest[..end];
    (!value.is_empty()).then(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_status_codes() {
        assert!(matches!(
            map_response(401, r#"{"detail":"nope"}"#, None, None),
            OtariError::Authentication { .. }
        ));
        assert!(matches!(
            map_response(403, r#"{"detail":"nope"}"#, None, None),
            OtariError::Authentication { .. }
        ));
        assert!(matches!(
            map_response(404, r#"{"detail":"gone"}"#, None, None),
            OtariError::ModelNotFound { .. }
        ));
        assert!(matches!(
            map_response(429, r#"{"detail":"slow"}"#, None, None),
            OtariError::RateLimit { .. }
        ));
        assert!(matches!(
            map_response(
                409,
                r#"{"detail":"Batch 'b1' is not yet complete (status: in_progress)."}"#,
                None,
                None
            ),
            OtariError::BatchNotComplete { .. }
        ));
        assert!(matches!(
            map_response(502, "{}", None, None),
            OtariError::Provider { .. }
        ));
        assert!(matches!(
            map_response(504, "{}", None, None),
            OtariError::Provider { .. }
        ));
        assert!(matches!(
            map_response(418, "{}", None, None),
            OtariError::Provider { .. }
        ));
    }

    #[test]
    fn unsupported_moderation_in_any_mode() {
        let err = map_response(
            400,
            r#"{"detail":"Provider anthropic does not support moderation"}"#,
            None,
            None,
        );
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

    #[test]
    fn batch_not_complete_extracts_fields() {
        let err = map_response(
            409,
            r#"{"detail":"Batch 'batch_abc' is not yet complete (status: validating)."}"#,
            None,
            None,
        );
        match err {
            OtariError::BatchNotComplete {
                batch_id, status, ..
            } => {
                assert_eq!(batch_id, "batch_abc");
                assert_eq!(status, "validating");
            }
            other => panic!("expected BatchNotComplete, got {other:?}"),
        }
    }

    #[test]
    fn correlation_id_in_message() {
        let err = map_response(402, r#"{"detail":"no funds"}"#, Some("abc-123"), None);
        assert!(err.to_string().contains("abc-123"));
    }
}
