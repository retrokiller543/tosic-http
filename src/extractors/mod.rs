use thiserror::Error;

pub mod json;
pub mod query;

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("Failed to parse query: {0}")]
    Query(#[from] serde::de::value::Error),
}
