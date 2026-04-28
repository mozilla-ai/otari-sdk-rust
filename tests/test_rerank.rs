use otari::types::{RerankMeta, RerankParams, RerankResponse, RerankResult, RerankUsage};
use pretty_assertions::assert_eq;
use std::collections::HashMap;

#[test]
fn test_rerank_response_serde_roundtrip() {
    let response = RerankResponse {
        id: "rerank-123".to_string(),
        results: vec![
            RerankResult {
                index: 0,
                relevance_score: 0.95,
            },
            RerankResult {
                index: 2,
                relevance_score: 0.80,
            },
        ],
        meta: Some(RerankMeta {
            billed_units: Some(HashMap::from([("search_units".to_string(), 1.0)])),
            tokens: Some(HashMap::from([("input_tokens".to_string(), 100)])),
        }),
        usage: Some(RerankUsage {
            total_tokens: Some(100),
        }),
    };

    let json = serde_json::to_string(&response).unwrap();
    let deserialized: RerankResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response, deserialized);
}

#[test]
fn test_rerank_response_minimal() {
    let json = r#"{"id": "r-1", "results": []}"#;
    let response: RerankResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.id, "r-1");
    assert!(response.results.is_empty());
    assert!(response.meta.is_none());
    assert!(response.usage.is_none());
}

#[test]
fn test_rerank_params_serialization() {
    let params = RerankParams {
        model_id: "cohere:rerank-v3.5".to_string(),
        query: "test query".to_string(),
        documents: vec!["doc1".to_string(), "doc2".to_string()],
        top_n: Some(2),
        max_tokens_per_doc: None,
        user: None,
    };
    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"model_id\""));
    assert!(json.contains("\"top_n\":2"));
    assert!(!json.contains("max_tokens_per_doc"));
    assert!(!json.contains("\"user\""));
}

#[test]
fn test_rerank_result_ordering() {
    let json = r#"{
        "id": "r-1",
        "results": [
            {"index": 2, "relevance_score": 0.30},
            {"index": 0, "relevance_score": 0.95},
            {"index": 1, "relevance_score": 0.80}
        ]
    }"#;
    let response: RerankResponse = serde_json::from_str(json).unwrap();
    // The SDK deserializes as-is. Sorting is the server's responsibility.
    assert_eq!(response.results[0].index, 2);
    assert_eq!(response.results[1].index, 0);
}
