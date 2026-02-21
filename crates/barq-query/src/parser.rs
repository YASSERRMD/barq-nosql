use barq_core::{BarqError, Document, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BarqQuery {
    pub collection: String,
    #[serde(default)]
    pub filter: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub vector_search: Option<VectorSearchSpec>,
    #[serde(default)]
    pub graph_expand: Option<GraphExpandSpec>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub offset: usize,
    #[serde(default)]
    pub sort: Option<HashMap<String, SortOrder>>,
}

fn default_limit() -> usize {
    100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchSpec {
    pub field: String,
    pub query: Vec<f32>,
    #[serde(default = "default_k")]
    pub k: usize,
    #[serde(default)]
    pub metric: Option<String>,
}

fn default_k() -> usize {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphExpandSpec {
    pub from: String,
    #[serde(default = "default_hops")]
    pub hops: usize,
    #[serde(default)]
    pub label: Option<String>,
}

fn default_hops() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

impl BarqQuery {
    pub fn parse(json: serde_json::Value) -> Result<Self, BarqError> {
        serde_json::from_value(json).map_err(|e| BarqError::QueryParseError(e.to_string()))
    }
}

pub type FilterExpr = serde_json::Value;
