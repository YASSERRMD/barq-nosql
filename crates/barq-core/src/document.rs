use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Vector(Vec<f32>),
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct DocumentId(pub Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DocumentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocumentId,
    pub data: HashMap<String, Value>,
}

impl Document {
    pub fn new(id: DocumentId) -> Self {
        Self {
            id,
            data: HashMap::new(),
        }
    }

    pub fn with_data(id: DocumentId, data: HashMap<String, Value>) -> Self {
        Self { id, data }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    pub fn insert(&mut self, key: String, value: Value) -> Option<Value> {
        self.data.insert(key, value)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.data.remove(key)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.data.values()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_id_new() {
        let id1 = DocumentId::new();
        let id2 = DocumentId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_document_insert_get() {
        let mut doc = Document::new(DocumentId::new());
        doc.insert("name".to_string(), Value::String("Alice".to_string()));

        assert_eq!(doc.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(doc.get("age"), None);
    }

    #[test]
    fn test_value_serialization() {
        let doc = Document::with_data(
            DocumentId::new(),
            vec![
                ("name".to_string(), Value::String("Bob".to_string())),
                ("age".to_string(), Value::Int(30)),
                ("score".to_string(), Value::Float(95.5)),
                ("active".to_string(), Value::Bool(true)),
            ]
            .into_iter()
            .collect(),
        );

        let json = serde_json::to_string(&doc).unwrap();
        let parsed: Document = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.get("name"), Some(&Value::String("Bob".to_string())));
    }
}
