use crate::edge::{Edge, EdgeDirection};
use barq_core::DocumentId;
use dashmap::DashMap;
use std::sync::Arc;

pub struct AdjacencyStore {
    outgoing: Arc<DashMap<DocumentId, Vec<Edge>>>,
    incoming: Arc<DashMap<DocumentId, Vec<Edge>>>,
}

impl AdjacencyStore {
    pub fn new() -> Self {
        Self {
            outgoing: Arc::new(DashMap::new()),
            incoming: Arc::new(DashMap::new()),
        }
    }

    pub fn add_edge(&self, edge: Edge) {
        self.outgoing
            .entry(edge.from.clone())
            .or_insert_with(Vec::new)
            .push(edge.clone());

        self.incoming
            .entry(edge.to.clone())
            .or_insert_with(Vec::new)
            .push(edge);
    }

    pub fn remove_edge(&self, edge_id: uuid::Uuid) -> Option<Edge> {
        for entry in self.outgoing.iter() {
            let key = entry.key().clone();
            if let Some(mut edges) = self.outgoing.get_mut(&key) {
                if let Some(pos) = edges.iter().position(|e| e.id == edge_id) {
                    return Some(edges.remove(pos));
                }
            }
        }

        for entry in self.incoming.iter() {
            let key = entry.key().clone();
            if let Some(mut edges) = self.incoming.get_mut(&key) {
                if let Some(pos) = edges.iter().position(|e| e.id == edge_id) {
                    return Some(edges.remove(pos));
                }
            }
        }

        None
    }

    pub fn get_neighbors(
        &self,
        doc_id: &DocumentId,
        label: Option<&str>,
        direction: EdgeDirection,
    ) -> Vec<DocumentId> {
        let mut neighbors = Vec::new();

        match direction {
            EdgeDirection::Outgoing | EdgeDirection::Both => {
                if let Some(edges) = self.outgoing.get(doc_id) {
                    for edge in edges.value() {
                        if label.is_none() || label == Some(&edge.label) {
                            neighbors.push(edge.to.clone());
                        }
                    }
                }
            }
            EdgeDirection::Incoming => {}
        }

        match direction {
            EdgeDirection::Incoming | EdgeDirection::Both => {
                if let Some(edges) = self.incoming.get(doc_id) {
                    for edge in edges.value() {
                        if label.is_none() || label == Some(&edge.label) {
                            neighbors.push(edge.from.clone());
                        }
                    }
                }
            }
            EdgeDirection::Outgoing => {}
        }

        neighbors
    }

    pub fn get_edges(&self, doc_id: &DocumentId, direction: EdgeDirection) -> Vec<Edge> {
        let mut edges = Vec::new();

        match direction {
            EdgeDirection::Outgoing | EdgeDirection::Both => {
                if let Some(out) = self.outgoing.get(doc_id) {
                    edges.extend(out.value().clone());
                }
            }
            EdgeDirection::Incoming => {}
        }

        match direction {
            EdgeDirection::Incoming | EdgeDirection::Both => {
                if let Some(incoming) = self.incoming.get(doc_id) {
                    edges.extend(incoming.value().clone());
                }
            }
            EdgeDirection::Outgoing => {}
        }

        edges
    }

    pub fn has_edge(&self, from: &DocumentId, to: &DocumentId) -> bool {
        if let Some(edges) = self.outgoing.get(from) {
            edges.value().iter().any(|e| &e.to == to)
        } else {
            false
        }
    }

    pub fn degree(&self, doc_id: &DocumentId, direction: EdgeDirection) -> usize {
        match direction {
            EdgeDirection::Outgoing => self
                .outgoing
                .get(doc_id)
                .map(|e| e.value().len())
                .unwrap_or(0),
            EdgeDirection::Incoming => self
                .incoming
                .get(doc_id)
                .map(|e| e.value().len())
                .unwrap_or(0),
            EdgeDirection::Both => {
                self.outgoing
                    .get(doc_id)
                    .map(|e| e.value().len())
                    .unwrap_or(0)
                    + self
                        .incoming
                        .get(doc_id)
                        .map(|e| e.value().len())
                        .unwrap_or(0)
            }
        }
    }

    pub fn clear(&self) {
        self.outgoing.clear();
        self.incoming.clear();
    }
}

impl Default for AdjacencyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let store = AdjacencyStore::new();

        let from = DocumentId::new();
        let to = DocumentId::new();

        store.add_edge(Edge::with_label(from.clone(), to.clone(), "friend"));

        let neighbors = store.get_neighbors(&from, None, EdgeDirection::Outgoing);
        assert_eq!(neighbors.len(), 1);
        assert_eq!(neighbors[0], to);
    }

    #[test]
    fn test_remove_edge() {
        let store = AdjacencyStore::new();

        let from = DocumentId::new();
        let to = DocumentId::new();

        store.add_edge(Edge::with_label(from.clone(), to.clone(), "friend"));
        store.remove_edge(uuid::Uuid::new_v4());

        let neighbors = store.get_neighbors(&from, None, EdgeDirection::Outgoing);
        assert!(neighbors.is_empty());
    }
}
