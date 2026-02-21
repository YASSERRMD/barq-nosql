use crate::engine::{BarqEngine, Metrics};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use barq_core::{BarqError, BarqErrorResponse, Document, DocumentId};
use barq_query::{BarqQuery, QueryExecutor, QueryPlanner};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "version": "0.1.0"
    }))
}

pub async fn metrics_handler(
    State(engine): State<Arc<BarqEngine>>,
) -> Result<Json<Metrics>, (StatusCode, Json<BarqErrorResponse>)> {
    let metrics = engine.get_metrics().await;
    Ok(Json(metrics))
}

pub async fn list_collections_handler(
    State(engine): State<Arc<BarqEngine>>,
) -> Json<Vec<String>> {
    Json(engine.list_collections().await)
}

#[derive(Deserialize)]
pub struct CreateCollectionRequest {
    name: String,
}

pub async fn create_collection_handler(
    State(engine): State<Arc<BarqEngine>>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    engine
        .create_collection(req.name.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(serde_json::json!({
        "collection": req.name,
        "status": "created"
    })))
}

pub async fn drop_collection_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(name): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    engine
        .drop_collection(&name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(serde_json::json!({
        "collection": name,
        "status": "dropped"
    })))
}

pub async fn insert_document_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
    Json(doc): Json<Document>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    let doc_id = engine
        .insert_document(&collection, doc)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(serde_json::json!({
        "id": doc_id.0,
        "status": "inserted"
    })))
}

pub async fn get_document_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<Document>, (StatusCode, Json<BarqErrorResponse>)> {
    let doc_id = DocumentId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(BarqError::QueryParseError("Invalid UUID".to_string()).to_response()))
        })?,
    );
    
    let doc = engine
        .get_document(&collection, &doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, Json(BarqError::DocumentNotFound(doc_id).to_response()))
        })?;
    
    Ok(Json(doc))
}

pub async fn list_documents_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
) -> Result<Json<Vec<Document>>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(Vec::new()))
}

pub async fn upsert_document_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path((collection, id)): Path<(String, String)>,
    Json(doc): Json<Document>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    let doc_id = DocumentId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(BarqError::QueryParseError("Invalid UUID".to_string()).to_response()))
        })?,
    );
    
    engine
        .upsert_document(&collection, &doc_id, doc)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(serde_json::json!({
        "id": doc_id.0,
        "status": "upserted"
    })))
}

pub async fn delete_document_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path((collection, id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    let doc_id = DocumentId::from_uuid(
        uuid::Uuid::parse_str(&id).map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(BarqError::QueryParseError("Invalid UUID".to_string()).to_response()))
        })?,
    );
    
    engine
        .delete_document(&collection, &doc_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(serde_json::json!({
        "status": "deleted"
    })))
}

pub async fn query_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
    Json(query): Json<BarqQuery>,
) -> Result<Json<Vec<Document>>, (StatusCode, Json<BarqErrorResponse>)> {
    let plan = QueryPlanner::plan(&query);
    
    let documents = std::collections::HashMap::new();
    let results = QueryExecutor::execute(plan, &documents)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    Ok(Json(results))
}

#[derive(Deserialize)]
pub struct VectorSearchRequest {
    field: String,
    query: Vec<f32>,
    k: Option<usize>,
}

pub async fn vector_search_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
    Json(req): Json<VectorSearchRequest>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<BarqErrorResponse>)> {
    let k = req.k.unwrap_or(10);
    
    let results = engine
        .vector_search(&collection, &req.field, req.query, k)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    let response: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(id, score)| {
            serde_json::json!({
                "id": id.0,
                "score": score
            })
        })
        .collect();
    
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct GraphTraverseRequest {
    from: String,
    hops: Option<usize>,
    label: Option<String>,
}

pub async fn graph_traverse_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
    Json(req): Json<GraphTraverseRequest>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<BarqErrorResponse>)> {
    let from_id = DocumentId::from_uuid(
        uuid::Uuid::parse_str(&req.from).map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(BarqError::QueryParseError("Invalid UUID".to_string()).to_response()))
        })?,
    );
    
    let hops = req.hops.unwrap_or(1);
    
    let results = engine
        .graph_traverse(&from_id, hops, req.label.as_deref())
        .await;
    
    let response: Vec<serde_json::Value> = results
        .into_iter()
        .map(|id| serde_json::json!({ "id": id.0 }))
        .collect();
    
    Ok(Json(response))
}

#[derive(Deserialize)]
pub struct HybridRequest {
    from: String,
    hops: usize,
    field: String,
    query: Vec<f32>,
    k: Option<usize>,
}

pub async fn hybrid_handler(
    State(engine): State<Arc<BarqEngine>>,
    Path(collection): Path<String>,
    Json(req): Json<HybridRequest>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<BarqErrorResponse>)> {
    let from_id = DocumentId::from_uuid(
        uuid::Uuid::parse_str(&req.from).map_err(|_| {
            (StatusCode::BAD_REQUEST, Json(BarqError::QueryParseError("Invalid UUID".to_string()).to_response()))
        })?,
    );
    
    let k = req.k.unwrap_or(10);
    
    let results = engine
        .hybrid_search(&from_id, req.hops, &req.field, req.query, k)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(e.to_response())))?;
    
    let response: Vec<serde_json::Value> = results
        .into_iter()
        .map(|(id, score)| {
            serde_json::json!({
                "id": id.0,
                "score": score
            })
        })
        .collect();
    
    Ok(Json(response))
}
