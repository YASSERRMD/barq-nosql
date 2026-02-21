use barq_core::{DocumentId, Value};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeDirection {
    Outgoing,
    Incoming,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: Uuid,
    pub from: DocumentId,
    pub to: DocumentId,
    pub label: String,
    pub weight: f32,
    pub properties: HashMap<String, Value>,
}

impl Edge {
    pub fn new(
        from: DocumentId,
        to: DocumentId,
        label: String,
        weight: f32,
        properties: HashMap<String, Value>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            from,
            to,
            label,
            weight,
            properties,
        }
    }

    pub fn with_label(from: DocumentId, to: DocumentId, label: impl Into<String>) -> Self {
        Self::new(from, to, label.into(), 1.0, HashMap::new())
    }

    pub fn with_weight(
        from: DocumentId,
        to: DocumentId,
        label: impl Into<String>,
        weight: f32,
    ) -> Self {
        Self::new(from, to, label.into(), weight, HashMap::new())
    }

    pub fn get_property(&self, key: &str) -> Option<&Value> {
        self.properties.get(key)
    }

    pub fn set_property(&mut self, key: String, value: Value) -> Option<Value> {
        self.properties.insert(key, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let from = DocumentId::new();
        let to = DocumentId::new();

        let edge = Edge::with_label(from.clone(), to.clone(), "friend");

        assert_eq!(edge.from, from);
        assert_eq!(edge.to, to);
        assert_eq!(edge.label, "friend");
        assert_eq!(edge.weight, 1.0);
    }
}
