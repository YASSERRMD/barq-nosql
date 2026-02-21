use barq_core::{DocumentId, FieldType, Value};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecondaryIndexType {
    Hash,
    BTree,
    FullText,
    Vector { dim: usize },
}

pub struct SecondaryIndex {
    pub collection: String,
    pub field: String,
    pub index_type: SecondaryIndexType,
    data: DashMap<Value, Vec<DocumentId>>,
}

impl SecondaryIndex {
    pub fn new(collection: &str, field: &str, index_type: SecondaryIndexType) -> Self {
        Self {
            collection: collection.to_string(),
            field: field.to_string(),
            index_type,
            data: DashMap::new(),
        }
    }

    pub fn insert(&self, value: Value, doc_id: DocumentId) {
        self.data.entry(value).or_insert_with(Vec::new).push(doc_id);
    }

    pub fn remove(&self, value: &Value, doc_id: &DocumentId) {
        if let Some(docs) = self.data.get_mut(value) {
            docs.retain(|id| id != doc_id);
        }
    }

    pub fn search(&self, value: &Value) -> Vec<DocumentId> {
        self.data.get(value).map(|v| v.clone()).unwrap_or_default()
    }

    pub fn range(&self, lo: &Value, hi: &Value) -> Vec<DocumentId> {
        let mut results = Vec::new();
        for (val, docs) in self.data.iter() {
            if val >= lo && val < hi {
                results.extend(docs.clone());
            }
        }
        results
    }
}

#[derive(Default)]
pub struct SecondaryIndexRegistry {
    indexes: DashMap<(String, String), SecondaryIndex>,
}

impl SecondaryIndexRegistry {
    pub fn create_index(
        &self,
        collection: &str,
        field: &str,
        index_type: SecondaryIndexType,
    ) -> Result<(), String> {
        let index = SecondaryIndex::new(collection, field, index_type);
        self.indexes
            .insert((collection.to_string(), field.to_string()), index);
        Ok(())
    }

    pub fn drop_index(&self, collection: &str, field: &str) -> Result<(), String> {
        if self
            .indexes
            .remove(&(collection.to_string(), field.to_string()))
            .is_some()
        {
            Ok(())
        } else {
            Err(format!("Index not found for {}.{}", collection, field))
        }
    }

    pub fn get_index(&self, collection: &str, field: &str) -> Option<SecondaryIndex> {
        self.indexes
            .get(&(collection.to_string(), field.to_string()))
            .map(|i| i.clone())
    }

    pub fn list_indexes(&self, collection: &str) -> Vec<String> {
        self.indexes
            .iter()
            .filter(|(k, _)| k.0 == collection)
            .map(|(k, _)| k.1.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_index() {
        let registry = SecondaryIndexRegistry::default();
        registry
            .create_index("users", "age", SecondaryIndexType::BTree)
            .unwrap();

        let indexes = registry.list_indexes("users");
        assert!(indexes.contains(&"age".to_string()));
    }
}
