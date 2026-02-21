use crate::adjacency::AdjacencyStore;
use crate::edge::Edge;
use crate::traversal::GraphTraversal;
use barq_core::{DocumentId, Value};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Edge not found: {0}")]
    EdgeNotFound(uuid::Uuid),
    #[error("Node not found: {0}")]
    NodeNotFound(DocumentId),
    #[error("Graph error: {0}")]
    Generic(String),
}

pub struct GraphEngine {
    adjacency: Arc<AdjacencyStore>,
}

impl GraphEngine {
    pub fn new() -> Self {
        Self {
            adjacency: Arc::new(AdjacencyStore::new()),
        }
    }

    pub fn with_adjacency(adjacency: Arc<AdjacencyStore>) -> Self {
        Self { adjacency }
    }

    pub fn link(
        &self,
        from: DocumentId,
        to: DocumentId,
        label: String,
        properties: HashMap<String, Value>,
    ) -> Result<Edge, GraphError> {
        let edge = Edge::new(from.clone(), to.clone(), label, 1.0, properties);
        self.adjacency.add_edge(edge.clone());
        Ok(edge)
    }

    pub fn link_with_weight(
        &self,
        from: DocumentId,
        to: DocumentId,
        label: String,
        weight: f32,
        properties: HashMap<String, Value>,
    ) -> Result<Edge, GraphError> {
        let edge = Edge::new(from.clone(), to.clone(), label, weight, properties);
        self.adjacency.add_edge(edge.clone());
        Ok(edge)
    }

    pub fn unlink(&self, edge_id: uuid::Uuid) -> Result<(), GraphError> {
        self.adjacency
            .remove_edge(edge_id)
            .map(|_| ())
            .ok_or(GraphError::EdgeNotFound(edge_id))
    }

    pub fn neighbors(&self, doc_id: &DocumentId, hops: usize) -> Vec<DocumentId> {
        let traversal = GraphTraversal::new(&self.adjacency);
        traversal.bfs(doc_id, hops, None)
    }

    pub fn neighbors_by_label(
        &self,
        doc_id: &DocumentId,
        hops: usize,
        label: &str,
    ) -> Vec<DocumentId> {
        let traversal = GraphTraversal::new(&self.adjacency);
        traversal.bfs(doc_id, hops, Some(label))
    }

    pub fn shortest_path(&self, from: &DocumentId, to: &DocumentId) -> Option<Vec<DocumentId>> {
        let traversal = GraphTraversal::new(&self.adjacency);
        traversal.shortest_path(from, to)
    }

    pub fn get_edges(&self, doc_id: &DocumentId) -> Vec<Edge> {
        self.adjacency
            .get_edges(doc_id, crate::edge::EdgeDirection::Both)
    }

    pub fn has_edge(&self, from: &DocumentId, to: &DocumentId) -> bool {
        self.adjacency.has_edge(from, to)
    }

    pub fn degree(&self, doc_id: &DocumentId) -> usize {
        self.adjacency
            .degree(doc_id, crate::edge::EdgeDirection::Both)
    }

    pub fn adjacency(&self) -> &Arc<AdjacencyStore> {
        &self.adjacency
    }

    pub fn hybrid_query(
        &self,
        doc_id: &DocumentId,
        hops: usize,
        vector_scores: Vec<(DocumentId, f32)>,
        k: usize,
    ) -> Vec<(DocumentId, f32)> {
        let graph_neighbors: std::collections::HashSet<DocumentId> =
            self.neighbors(doc_id, hops).into_iter().collect();

        let mut filtered: Vec<(DocumentId, f32)> = vector_scores
            .into_iter()
            .filter(|(doc_id, _)| graph_neighbors.contains(doc_id))
            .collect();

        filtered.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        filtered.truncate(k);

        filtered
    }
}

impl Default for GraphEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_unlink() {
        let engine = GraphEngine::new();

        let from = DocumentId::new();
        let to = DocumentId::new();

        let edge = engine
            .link(
                from.clone(),
                to.clone(),
                "friend".to_string(),
                HashMap::new(),
            )
            .unwrap();

        assert!(engine.has_edge(&from, &to));

        engine.unlink(edge.id).unwrap();

        assert!(!engine.has_edge(&from, &to));
    }

    #[test]
    fn test_neighbors() {
        let engine = GraphEngine::new();

        let a = DocumentId::new();
        let b = DocumentId::new();
        let c = DocumentId::new();

        engine
            .link(a.clone(), b.clone(), "friend".to_string(), HashMap::new())
            .unwrap();
        engine
            .link(b.clone(), c.clone(), "friend".to_string(), HashMap::new())
            .unwrap();

        let neighbors = engine.neighbors(&a, 2);

        assert!(neighbors.contains(&b));
        assert!(neighbors.contains(&c));
    }
}
