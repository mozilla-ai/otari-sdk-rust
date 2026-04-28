use serde::Serialize;

use crate::types::RerankParams;

/// Gateway wire format for a rerank request.
#[derive(Debug, Serialize)]
pub struct GatewayRerankRequest {
    pub model: String,
    pub query: String,
    pub documents: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens_per_doc: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}

impl From<RerankParams> for GatewayRerankRequest {
    fn from(params: RerankParams) -> Self {
        Self {
            model: params.model_id,
            query: params.query,
            documents: params.documents,
            top_n: params.top_n,
            max_tokens_per_doc: params.max_tokens_per_doc,
            user: params.user,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_rerank_params() {
        let params = RerankParams {
            model_id: "cohere:rerank-v3.5".to_string(),
            query: "test query".to_string(),
            documents: vec!["doc1".to_string(), "doc2".to_string()],
            top_n: Some(2),
            max_tokens_per_doc: None,
            user: None,
        };
        let req = GatewayRerankRequest::from(params);
        assert_eq!(req.model, "cohere:rerank-v3.5");
        assert_eq!(req.query, "test query");
        assert_eq!(req.documents.len(), 2);
        assert_eq!(req.top_n, Some(2));
    }

    #[test]
    fn test_serialization_skips_none() {
        let req = GatewayRerankRequest {
            model: "test".to_string(),
            query: "q".to_string(),
            documents: vec!["d".to_string()],
            top_n: None,
            max_tokens_per_doc: None,
            user: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("top_n"));
        assert!(!json.contains("max_tokens_per_doc"));
        assert!(!json.contains("user"));
    }
}
