use thiserror::Error;

pub mod json;

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
