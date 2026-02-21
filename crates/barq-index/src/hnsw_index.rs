use barq_core::DocumentId;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DistanceMetric {
    Cosine,
    DotProduct,
    Euclidean,
}

#[derive(Debug, Clone)]
struct HnswNode {
    doc_id: DocumentId,
    vector: Vec<f32>,
    edges: Vec<usize>,
}

pub struct HnswIndex {
    layers: Vec<Vec<HnswNode>>,
    ef_construction: usize,
    m: usize,
    dim: usize,
    metric: DistanceMetric,
    entry_point: Option<usize>,
}

impl HnswIndex {
    pub fn new(dim: usize, metric: DistanceMetric) -> Self {
        Self {
            layers: Vec::new(),
            ef_construction: 200,
            m: 16,
            dim,
            metric,
            entry_point: None,
        }
    }

    pub fn with_params(
        dim: usize,
        m: usize,
        ef_construction: usize,
        metric: DistanceMetric,
    ) -> Self {
        Self {
            layers: Vec::new(),
            ef_construction,
            m,
            dim,
            metric,
            entry_point: None,
        }
    }

    pub fn insert(
        &mut self,
        doc_id: DocumentId,
        vector: Vec<f32>,
    ) -> Result<(), barq_core::BarqError> {
        if vector.len() != self.dim {
            return Err(barq_core::BarqError::VectorDimensionMismatch {
                expected: self.dim,
                got: vector.len(),
            });
        }

        if let Some(last_layer) = self.layers.last_mut() {
            last_layer.push(HnswNode {
                doc_id,
                vector,
                edges: Vec::new(),
            });
        } else {
            self.layers.push(Vec::new());
            self.layers[0].push(HnswNode {
                doc_id,
                vector: vector.clone(),
                edges: Vec::new(),
            });
        }

        if self.entry_point.is_none() {
            self.entry_point = Some(0);
        }

        Ok(())
    }

    fn distance(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.metric {
            DistanceMetric::Cosine => {
                let dot = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>();
                let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
                let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
                if norm_a == 0.0 || norm_b == 0.0 {
                    1.0
                } else {
                    1.0 - dot / (norm_a * norm_b)
                }
            }
            DistanceMetric::DotProduct => -a.iter().zip(b.iter()).map(|(x, y)| x * y).sum::<f32>(),
            DistanceMetric::Euclidean => a
                .iter()
                .zip(b.iter())
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f32>()
                .sqrt(),
        }
    }

    pub fn search(
        &self,
        query: Vec<f32>,
        k: usize,
    ) -> Result<Vec<(DocumentId, f32)>, barq_core::BarqError> {
        if query.len() != self.dim {
            return Err(barq_core::BarqError::VectorDimensionMismatch {
                expected: self.dim,
                got: query.len(),
            });
        }

        if self.layers.is_empty() {
            return Ok(Vec::new());
        }

        let ep = match self.entry_point {
            Some(idx) => idx,
            None => return Ok(Vec::new()),
        };

        let nodes = &self.layers[0];
        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let mut results: Vec<(usize, f32)> = nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (i, self.distance(&query, &n.vector)))
            .collect();

        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        results.truncate(k);

        let search_results: Vec<(DocumentId, f32)> = results
            .into_iter()
            .filter_map(|(idx, dist)| {
                if idx < nodes.len() {
                    Some((nodes[idx].doc_id.clone(), dist))
                } else {
                    None
                }
            })
            .collect();

        Ok(search_results)
    }

    pub fn len(&self) -> usize {
        if self.layers.is_empty() {
            0
        } else {
            self.layers[0].len()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn metric(&self) -> DistanceMetric {
        self.metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_insert() {
        let mut index = HnswIndex::new(128, DistanceMetric::Cosine);

        let vector = vec![0.1; 128];
        index.insert(DocumentId::new(), vector).unwrap();

        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_hnsw_search() {
        let mut index = HnswIndex::new(4, DistanceMetric::Euclidean);

        let v1 = vec![0.0, 0.0, 0.0, 0.0];
        let v2 = vec![1.0, 1.0, 1.0, 1.0];
        let v3 = vec![2.0, 2.0, 2.0, 2.0];

        index.insert(DocumentId::new(), v1).unwrap();
        index.insert(DocumentId::new(), v2).unwrap();
        index.insert(DocumentId::new(), v3).unwrap();

        let query = vec![0.5, 0.5, 0.5, 0.5];
        let results = index.search(query, 2).unwrap();

        assert_eq!(results.len(), 2);
    }
}
