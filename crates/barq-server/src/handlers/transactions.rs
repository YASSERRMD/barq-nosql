use crate::engine::BarqEngine;
use crate::transaction_manager::TransactionManager;
use crate::transaction::TransactionId;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use barq_core::BarqErrorResponse;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct BeginTxnResponse {
    pub txn_id: u64,
}

pub async fn begin_txn(
    State(engine): State<BarqEngine>,
) -> Result<Json<BeginTxnResponse>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(BeginTxnResponse { txn_id: 0 }))
}

#[derive(Deserialize)]
pub struct WriteTxnRequest {
    pub collection: String,
    pub doc_id: String,
    pub operation: String,
}

pub async fn write_txn(
    Path(txn_id): Path<u64>,
    State(_engine): State<BarqEngine>,
    Json(_req): Json<WriteTxnRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn commit_txn(
    Path(txn_id): Path<u64>,
    State(_engine): State<BarqEngine>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(serde_json::json!({ "status": "committed", "txn_id": txn_id })))
}

pub async fn abort_txn(
    Path(txn_id): Path<u64>,
    State(_engine): State<BarqEngine>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(serde_json::json!({ "status": "aborted", "txn_id": txn_id })))
}

#[derive(Deserialize)]
pub struct ReadTxnRequest {
    pub collection: String,
    pub doc_id: String,
}

pub async fn read_txn(
    Path(txn_id): Path<u64>,
    State(_engine): State<BarqEngine>,
    Json(_req): Json<ReadTxnRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<BarqErrorResponse>)> {
    Ok(Json(serde_json::json!({ "data": null })))
}
