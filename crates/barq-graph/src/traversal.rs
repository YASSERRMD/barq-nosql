use crate::adjacency::AdjacencyStore;
use crate::edge::EdgeDirection;
use barq_core::DocumentId;
use std::collections::{HashSet, VecDeque};

pub struct GraphTraversal<'a> {
    adjacency: &'a AdjacencyStore,
}

impl<'a> GraphTraversal<'a> {
    pub fn new(adjacency: &'a AdjacencyStore) -> Self {
        Self { adjacency }
    }

    pub fn bfs(
        &self,
        start: &DocumentId,
        max_hops: usize,
        label_filter: Option<&str>,
    ) -> Vec<DocumentId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        queue.push_back((start.clone(), 0));
        visited.insert(start.clone());

        while let Some((current, hops)) = queue.pop_front() {
            if hops > max_hops {
                continue;
            }

            if hops > 0 {
                result.push(current.clone());
            }

            let neighbors =
                self.adjacency
                    .get_neighbors(&current, label_filter, EdgeDirection::Outgoing);

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    queue.push_back((neighbor, hops + 1));
                }
            }
        }

        result
    }

    pub fn dfs(
        &self,
        start: &DocumentId,
        max_hops: usize,
        label_filter: Option<&str>,
    ) -> Vec<DocumentId> {
        let mut visited = HashSet::new();
        let mut result = Vec::new();
        let mut stack = vec![(start.clone(), 0)];

        while let Some((current, hops)) = stack.pop() {
            if hops > max_hops {
                continue;
            }

            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());

            if hops > 0 {
                result.push(current.clone());
            }

            let neighbors =
                self.adjacency
                    .get_neighbors(&current, label_filter, EdgeDirection::Outgoing);

            for neighbor in neighbors.into_iter().rev() {
                if !visited.contains(&neighbor) {
                    stack.push((neighbor, hops + 1));
                }
            }
        }

        result
    }

    pub fn shortest_path(&self, from: &DocumentId, to: &DocumentId) -> Option<Vec<DocumentId>> {
        if from == to {
            return Some(vec![from.clone()]);
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut parent: std::collections::HashMap<DocumentId, DocumentId> =
            std::collections::HashMap::new();

        queue.push_back(from.clone());
        visited.insert(from.clone());

        while let Some(current) = queue.pop_front() {
            if current == *to {
                let mut path = Vec::new();
                let mut node = current.clone();

                while let Some(prev) = parent.get(&node) {
                    path.push(node);
                    node = prev.clone();
                }
                path.push(from.clone());

                path.reverse();
                return Some(path);
            }

            let neighbors = self
                .adjacency
                .get_neighbors(&current, None, EdgeDirection::Outgoing);

            for neighbor in neighbors {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor.clone());
                    parent.insert(neighbor.clone(), current.clone());
                    queue.push_back(neighbor);
                }
            }
        }

        None
    }

    pub fn k_hop_neighbors(
        &self,
        doc_id: &DocumentId,
        hops: usize,
    ) -> std::collections::HashSet<DocumentId> {
        let mut neighbors = std::collections::HashSet::new();

        for hop in 1..=hops {
            let hop_neighbors = self.bfs(doc_id, hop, None);
            neighbors.extend(hop_neighbors);
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bfs() {
        let adjacency = AdjacencyStore::new();

        let a = DocumentId::new();
        let b = DocumentId::new();
        let c = DocumentId::new();

        adjacency.add_edge(crate::edge::Edge::with_label(
            a.clone(),
            b.clone(),
            "friend",
        ));
        adjacency.add_edge(crate::edge::Edge::with_label(
            b.clone(),
            c.clone(),
            "friend",
        ));

        let traversal = GraphTraversal::new(&adjacency);
        let result = traversal.bfs(&a, 2, None);

        assert!(result.contains(&b));
        assert!(result.contains(&c));
    }

    #[test]
    fn test_shortest_path() {
        let adjacency = AdjacencyStore::new();

        let a = DocumentId::new();
        let b = DocumentId::new();
        let c = DocumentId::new();

        adjacency.add_edge(crate::edge::Edge::with_label(
            a.clone(),
            b.clone(),
            "friend",
        ));
        adjacency.add_edge(crate::edge::Edge::with_label(
            b.clone(),
            c.clone(),
            "friend",
        ));

        let traversal = GraphTraversal::new(&adjacency);
        let path = traversal.shortest_path(&a, &c);

        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 3);
    }
}
