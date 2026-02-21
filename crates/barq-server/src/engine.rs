use barq_core::{BarqError, CollectionId, Document, DocumentId};
use barq_graph::GraphEngine;
use barq_index::IndexManager;
use barq_storage::StorageEngine;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

pub struct BarqEngine {
    storage: Arc<RwLock<StorageEngine>>,
    index_manager: Arc<IndexManager>,
    graph_engine: Arc<GraphEngine>,
    collections: Arc<RwLock<HashMap<String, CollectionMetadata>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetadata {
    pub name: String,
    pub document_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub total_collections: usize,
    pub total_documents: usize,
    pub storage_size_bytes: u64,
}

impl BarqEngine {
    pub async fn new(data_dir: String) -> Result<Self, BarqError> {
        let config = barq_storage::StorageConfig {
            data_dir: std::path::PathBuf::from(&data_dir),
            wal_enabled: true,
            flush_threshold_docs: 10_000,
            flush_threshold_bytes: 64 * 1024 * 1024,
        };

        let storage = Arc::new(RwLock::new(
            StorageEngine::new(config)
                .await
                .map_err(|e| BarqError::StorageError(e.to_string()))?,
        ));

        let index_manager = Arc::new(IndexManager::new());
        let graph_engine = Arc::new(GraphEngine::new());
        let collections = Arc::new(RwLock::new(HashMap::new()));

        info!("BarqEngine initialized with data directory: {}", data_dir);

        Ok(Self {
            storage,
            index_manager,
            graph_engine,
            collections,
        })
    }

    pub async fn create_collection(&self, name: String) -> Result<(), BarqError> {
        let mut collections = self.collections.write().await;
        collections.insert(
            name.clone(),
            CollectionMetadata {
                name: name.clone(),
                document_count: 0,
            },
        );
        info!("Created collection: {}", name);
        Ok(())
    }

    pub async fn drop_collection(&self, name: &str) -> Result<(), BarqError> {
        let mut collections = self.collections.write().await;
        collections.remove(name);
        info!("Dropped collection: {}", name);
        Ok(())
    }

    pub async fn list_collections(&self) -> Vec<String> {
        let collections = self.collections.read().await;
        collections.keys().cloned().collect()
    }

    pub async fn insert_document(
        &self,
        collection: &str,
        doc: Document,
    ) -> Result<DocumentId, BarqError> {
        let doc_id = doc.id.clone();
        
        self.storage
            .write()
            .await
            .insert(collection, doc_id.clone(), doc.clone())
            .await?;
        
        self.index_manager
            .update_indexes(collection, doc_id.clone(), &doc)?;
        
        let mut collections = self.collections.write().await;
        if let Some(meta) = collections.get_mut(collection) {
            meta.document_count += 1;
        }
        
        Ok(doc_id)
    }

    pub async fn get_document(
        &self,
        collection: &str,
        doc_id: &DocumentId,
    ) -> Result<Option<Document>, BarqError> {
        self.storage.read().await.get(collection, doc_id).await
    }

    pub async fn upsert_document(
        &self,
        collection: &str,
        doc_id: &DocumentId,
        doc: Document,
    ) -> Result<(), BarqError> {
        self.storage
            .write()
            .await
            .insert(collection, doc_id.clone(), doc.clone())
            .await?;
        
        self.index_manager
            .update_indexes(collection, doc_id.clone(), &doc)
    }

    pub async fn delete_document(
        &self,
        collection: &str,
        doc_id: &DocumentId,
    ) -> Result<(), BarqError> {
        self.storage
            .write()
            .await
            .delete(collection, doc_id)
            .await?;
        
        let mut collections = self.collections.write().await;
        if let Some(meta) = collections.get_mut(collection) {
            meta.document_count = meta.document_count.saturating_sub(1);
        }
        
        Ok(())
    }

    pub async fn vector_search(
        &self,
        collection: &str,
        field: &str,
        query: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(DocumentId, f32)>, BarqError> {
        self.index_manager
            .search_hnsw(collection, field, query, k)
    }

    pub async fn graph_traverse(
        &self,
        from: &DocumentId,
        hops: usize,
        label: Option<&str>,
    ) -> Vec<DocumentId> {
        self.graph_engine.neighbors_by_label(from, hops, label.unwrap_or(""))
    }

    pub async fn hybrid_search(
        &self,
        from: &DocumentId,
        hops: usize,
        field: &str,
        query: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(DocumentId, f32)>, BarqError> {
        let vector_results = self
            .index_manager
            .search_hnsw("", field, query.clone(), k * 2)?;
        
        let filtered = self.graph_engine.hybrid_query(from, hops, vector_results, k);
        
        Ok(filtered)
    }

    pub fn graph_engine(&self) -> &Arc<GraphEngine> {
        &self.graph_engine
    }

    pub fn index_manager(&self) -> &Arc<IndexManager> {
        &self.index_manager
    }

    pub async fn get_metrics(&self) -> Metrics {
        let collections = self.collections.read().await;
        let total_documents: usize = collections.values().map(|m| m.document_count).sum();
        
        Metrics {
            total_collections: collections.len(),
            total_documents,
            storage_size_bytes: 0,
        }
    }
}
