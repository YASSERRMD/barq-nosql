use crate::engine::BarqEngine;
use crate::handlers::*;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use std::sync::Arc;

pub fn router(engine: Arc<BarqEngine>) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .route("/collections", get(list_collections_handler))
        .route("/collections", post(create_collection_handler))
        .route("/collections/:name", delete(drop_collection_handler))
        .route(
            "/collections/:name/documents",
            post(insert_document_handler),
        )
        .route("/collections/:name/documents", get(list_documents_handler))
        .route(
            "/collections/:name/documents/:id",
            get(get_document_handler),
        )
        .route(
            "/collections/:name/documents/:id",
            put(upsert_document_handler),
        )
        .route(
            "/collections/:name/documents/:id",
            delete(delete_document_handler),
        )
        .route("/collections/:name/query", post(query_handler))
        .route("/collections/:name/vector", post(vector_search_handler))
        .route("/collections/:name/graph", post(graph_traverse_handler))
        .route("/collections/:name/hybrid", post(hybrid_handler))
        .with_state(engine)
}
