use crate::btree_index::{BTreeIndex, OrderedFloat};
use crate::hnsw_index::{DistanceMetric, HnswIndex};
use crate::inverted_index::InvertedIndex;
use barq_core::{BarqError, Document, DocumentId, FieldType, Value};
use dashmap::DashMap;
use std::sync::Arc;

pub struct IndexManager {
    btree_indexes: Arc<DashMap<String, BTreeIndexType>>,
    inverted_indexes: Arc<DashMap<String, InvertedIndex>>,
    hnsw_indexes: Arc<DashMap<String, HnswIndex>>,
}

enum BTreeIndexType {
    Int(BTreeIndex<i64, Vec<DocumentId>>),
    Float(BTreeIndex<OrderedFloat, Vec<DocumentId>>),
    String(BTreeIndex<String, Vec<DocumentId>>),
}

impl IndexManager {
    pub fn new() -> Self {
        Self {
            btree_indexes: Arc::new(DashMap::new()),
            inverted_indexes: Arc::new(DashMap::new()),
            hnsw_indexes: Arc::new(DashMap::new()),
        }
    }

    pub fn register_btree_index(
        &self,
        collection: &str,
        field: &str,
        field_type: FieldType,
    ) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);

        match field_type {
            FieldType::Int => {
                self.btree_indexes
                    .insert(key, BTreeIndexType::Int(BTreeIndex::new()));
                Ok(())
            }
            FieldType::Float => {
                self.btree_indexes
                    .insert(key, BTreeIndexType::Float(BTreeIndex::new()));
                Ok(())
            }
            FieldType::Text => {
                self.btree_indexes
                    .insert(key, BTreeIndexType::String(BTreeIndex::new()));
                Ok(())
            }
            _ => Err(BarqError::IndexError(format!(
                "Field type {:?} not supported for BTree index",
                field_type
            ))),
        }
    }

    pub fn register_inverted_index(&self, collection: &str, field: &str) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);
        self.inverted_indexes.insert(key, InvertedIndex::new());
        Ok(())
    }

    pub fn register_hnsw_index(
        &self,
        collection: &str,
        field: &str,
        dim: usize,
        metric: DistanceMetric,
    ) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);
        self.hnsw_indexes.insert(key, HnswIndex::new(dim, metric));
        Ok(())
    }

    pub fn update_btree_index(
        &self,
        collection: &str,
        field: &str,
        value: &barq_core::Value,
        doc_id: DocumentId,
    ) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);

        if let Some(mut index_entry) = self.btree_indexes.get_mut(&key) {
            match (index_entry.value_mut(), value) {
                (BTreeIndexType::Int(idx), barq_core::Value::Int(v)) => {
                    idx.insert(*v, vec![doc_id]);
                }
                (BTreeIndexType::Float(idx), barq_core::Value::Float(v)) => {
                    idx.insert(OrderedFloat::new(*v), vec![doc_id]);
                }
                (BTreeIndexType::String(idx), barq_core::Value::String(v)) => {
                    idx.insert(v.clone(), vec![doc_id]);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn update_inverted_index(
        &self,
        collection: &str,
        field: &str,
        value: &barq_core::Value,
        doc_id: DocumentId,
    ) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);

        if let Some(mut index) = self.inverted_indexes.get_mut(&key) {
            if let barq_core::Value::String(text) = value {
                index.insert_doc(doc_id, text);
            }
        }

        Ok(())
    }

    pub fn update_hnsw_index(
        &self,
        collection: &str,
        field: &str,
        value: &barq_core::Value,
        doc_id: DocumentId,
    ) -> Result<(), BarqError> {
        let key = format!("{}.{}", collection, field);

        if let Some(mut index) = self.hnsw_indexes.get_mut(&key) {
            if let barq_core::Value::Vector(vec) = value {
                index.insert(doc_id, vec.clone())?;
            }
        }

        Ok(())
    }

    pub fn update_indexes(
        &self,
        collection: &str,
        doc_id: DocumentId,
        doc: &Document,
    ) -> Result<(), BarqError> {
        for (field, value) in &doc.data {
            self.update_btree_index(collection, field, value, doc_id.clone())?;
            self.update_inverted_index(collection, field, value, doc_id.clone())?;
            self.update_hnsw_index(collection, field, value, doc_id.clone())?;
        }

        Ok(())
    }

    pub fn search_hnsw(
        &self,
        collection: &str,
        field: &str,
        query: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(DocumentId, f32)>, BarqError> {
        let key = format!("{}.{}", collection, field);

        if let Some(index) = self.hnsw_indexes.get(&key) {
            index.search(query, k)
        } else {
            Err(BarqError::IndexError(format!(
                "HNSW index not found for {}.{}",
                collection, field
            )))
        }
    }

    pub fn search_inverted(
        &self,
        collection: &str,
        field: &str,
        query: &str,
    ) -> Result<Vec<DocumentId>, BarqError> {
        let key = format!("{}.{}", collection, field);

        if let Some(index) = self.inverted_indexes.get(&key) {
            Ok(index.search(query))
        } else {
            Err(BarqError::IndexError(format!(
                "Inverted index not found for {}.{}",
                collection, field
            )))
        }
    }

    pub fn drop_index(&self, collection: &str, field: &str) -> Result<(), BarqError> {
        let btree_key = format!("{}.{}", collection, field);
        let inverted_key = format!("{}.{}", collection, field);
        let hnsw_key = format!("{}.{}", collection, field);

        self.btree_indexes.remove(&btree_key);
        self.inverted_indexes.remove(&inverted_key);
        self.hnsw_indexes.remove(&hnsw_key);

        Ok(())
    }

    pub fn list_indexes(&self, collection: &str) -> Vec<String> {
        let prefix = format!("{}.", collection);
        let mut indexes = Vec::new();

        let btree_keys: Vec<String> = self.btree_indexes.iter().map(|k| k.key().clone()).collect();
        for key in btree_keys {
            if key.starts_with(&prefix) {
                indexes.push(format!("btree:{}", key));
            }
        }

        let inverted_keys: Vec<String> = self
            .inverted_indexes
            .iter()
            .map(|k| k.key().clone())
            .collect();
        for key in inverted_keys {
            if key.starts_with(&prefix) {
                indexes.push(format!("inverted:{}", key));
            }
        }

        let hnsw_keys: Vec<String> = self.hnsw_indexes.iter().map(|k| k.key().clone()).collect();
        for key in hnsw_keys {
            if key.starts_with(&prefix) {
                indexes.push(format!("hnsw:{}", key));
            }
        }

        indexes
    }
}

impl Default for IndexManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_hnsw_index() {
        let manager = IndexManager::new();

        manager
            .register_hnsw_index("items", "embedding", 128, DistanceMetric::Cosine)
            .unwrap();

        let key = "items.embedding";
        assert!(manager.hnsw_indexes.contains_key(key));
    }

    #[test]
    fn test_update_and_search_hnsw() {
        let manager = IndexManager::new();

        manager
            .register_hnsw_index("items", "emb", 4, DistanceMetric::Euclidean)
            .unwrap();

        let doc = Document::with_data(
            DocumentId::new(),
            vec![("emb".to_string(), Value::Vector(vec![1.0, 2.0, 3.0, 4.0]))]
                .into_iter()
                .collect(),
        );

        manager
            .update_indexes("items", doc.id.clone(), &doc)
            .unwrap();

        let results = manager
            .search_hnsw("items", "emb", vec![1.0, 2.0, 3.0, 4.0], 1)
            .unwrap();

        assert!(!results.is_empty());
    }
}
