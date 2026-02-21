use crate::document::DocumentId;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum BarqError {
    #[error("Document not found: {0}")]
    DocumentNotFound(DocumentId),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Query parse error: {0}")]
    QueryParseError(String),

    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    VectorDimensionMismatch { expected: usize, got: usize },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarqErrorResponse {
    pub error: String,
    pub code: String,
}

impl BarqError {
    pub fn to_response(&self) -> BarqErrorResponse {
        BarqErrorResponse {
            error: self.to_string(),
            code: self.error_code().to_string(),
        }
    }

    pub fn error_code(&self) -> &str {
        match self {
            BarqError::DocumentNotFound(_) => "DOCUMENT_NOT_FOUND",
            BarqError::CollectionNotFound(_) => "COLLECTION_NOT_FOUND",
            BarqError::SchemaMismatch(_) => "SCHEMA_MISMATCH",
            BarqError::StorageError(_) => "STORAGE_ERROR",
            BarqError::IndexError(_) => "INDEX_ERROR",
            BarqError::QueryParseError(_) => "QUERY_PARSE_ERROR",
            BarqError::VectorDimensionMismatch { .. } => "VECTOR_DIMENSION_MISMATCH",
            BarqError::IoError(_) => "IO_ERROR",
            BarqError::SerdeError(_) => "SERIALIZATION_ERROR",
        }
    }
}
