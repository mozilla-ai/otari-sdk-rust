//! Batch operation types for the gateway provider.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status of a batch job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchStatus {
    /// Batch is being validated.
    Validating,
    /// Batch validation or processing failed.
    Failed,
    /// Batch is currently being processed.
    InProgress,
    /// Batch is being finalized.
    Finalizing,
    /// Batch has completed successfully.
    Completed,
    /// Batch has expired.
    Expired,
    /// Batch is being cancelled.
    Cancelling,
    /// Batch has been cancelled.
    Cancelled,
}

/// Request counts for a batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestCounts {
    /// Total number of requests in the batch.
    pub total: u32,
    /// Number of completed requests.
    pub completed: u32,
    /// Number of failed requests.
    pub failed: u32,
}

/// A batch job returned by the gateway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Batch {
    /// Unique identifier for the batch.
    pub id: String,
    /// The object type (always "batch").
    pub object: String,
    /// The API endpoint used for the batch requests.
    pub endpoint: String,
    /// Current status of the batch.
    pub status: BatchStatus,
    /// Unix timestamp of when the batch was created.
    pub created_at: i64,
    /// The time window for batch completion.
    pub completion_window: String,
    /// The provider that is processing this batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    /// The input file ID, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_file_id: Option<String>,
    /// The output file ID, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_file_id: Option<String>,
    /// The error file ID, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_file_id: Option<String>,
    /// Counts of requests by status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_counts: Option<BatchRequestCounts>,
    /// User-provided metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Unix timestamp of when the batch started processing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_progress_at: Option<i64>,
    /// Unix timestamp of when the batch completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
}

/// Parameters for creating a batch job.
#[derive(Debug, Clone, Serialize)]
pub struct CreateBatchParams {
    /// The model to use, in `provider:model` format (e.g., "openai:gpt-4o-mini").
    pub model: String,
    /// The list of requests to process.
    pub requests: Vec<BatchRequestItem>,
    /// The time window for batch completion (e.g., "24h").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_window: Option<String>,
    /// User-provided metadata key-value pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl CreateBatchParams {
    /// Create new batch params with required fields.
    pub fn new(model: impl Into<String>, requests: Vec<BatchRequestItem>) -> Self {
        Self {
            model: model.into(),
            requests,
            completion_window: None,
            metadata: None,
        }
    }

    /// Set the completion window (e.g., "24h").
    pub fn completion_window(mut self, window: impl Into<String>) -> Self {
        self.completion_window = Some(window.into());
        self
    }

    /// Set metadata key-value pairs.
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// A single request within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequestItem {
    /// A unique identifier for this request within the batch.
    pub custom_id: String,
    /// The request body (typically containing `messages`, `max_tokens`, etc.).
    pub body: serde_json::Value,
}

/// Options for listing batch jobs.
#[derive(Debug, Clone, Default)]
pub struct ListBatchesOptions {
    /// Cursor for pagination — return batches after this ID.
    pub after: Option<String>,
    /// Maximum number of batches to return.
    pub limit: Option<u32>,
}

/// An error for a single request within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultError {
    /// The error code.
    pub code: String,
    /// A human-readable error message.
    pub message: String,
}

/// The result of a single request within a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResultItem {
    /// The custom ID matching the original request.
    pub custom_id: String,
    /// The successful completion result, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<super::completion::ChatCompletion>,
    /// The error, if the request failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<BatchResultError>,
}

/// The results of a completed batch job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    /// The list of results, one per request in the batch.
    pub results: Vec<BatchResultItem>,
}
