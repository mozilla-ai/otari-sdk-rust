use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for a rerank request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankParams {
    pub model_id: String,
    pub query: String,
    pub documents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_doc: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

/// A single reranked document score.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RerankResult {
    pub index: u32,
    pub relevance_score: f64,
}

/// Provider-specific billing metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RerankMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billed_units: Option<HashMap<String, f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<HashMap<String, u32>>,
}

/// Normalized token usage for a rerank request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RerankUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,
}

/// Normalized rerank response, provider-agnostic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RerankResponse {
    pub id: String,
    pub results: Vec<RerankResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<RerankMeta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,
}
