use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemRegion {
    MemTable,
    IndexCache,
    VectorCache,
}

pub struct MemoryBudget {
    pub max_memtable_bytes: usize,
    pub max_index_cache_bytes: usize,
    pub max_vector_cache_bytes: usize,
}

impl Default for MemoryBudget {
    fn default() -> Self {
        Self {
            max_memtable_bytes: 256 * 1024 * 1024,
            max_index_cache_bytes: 512 * 1024 * 1024,
            max_vector_cache_bytes: 1024 * 1024 * 1024,
        }
    }
}

pub struct MemoryTracker {
    current_memtable_bytes: AtomicUsize,
    current_index_bytes: AtomicUsize,
    current_vector_bytes: AtomicUsize,
    budget: MemoryBudget,
}

impl MemoryTracker {
    pub fn new(budget: MemoryBudget) -> Self {
        Self {
            current_memtable_bytes: AtomicUsize::new(0),
            current_index_bytes: AtomicUsize::new(0),
            current_vector_bytes: AtomicUsize::new(0),
            budget,
        }
    }

    pub fn track_alloc(&self, region: MemRegion, bytes: usize) {
        match region {
            MemRegion::MemTable => {
                self.current_memtable_bytes
                    .fetch_add(bytes, Ordering::Relaxed);
            }
            MemRegion::IndexCache => {
                self.current_index_bytes.fetch_add(bytes, Ordering::Relaxed);
            }
            MemRegion::VectorCache => {
                self.current_vector_bytes
                    .fetch_add(bytes, Ordering::Relaxed);
            }
        }
    }

    pub fn track_free(&self, region: MemRegion, bytes: usize) {
        match region {
            MemRegion::MemTable => {
                self.current_memtable_bytes
                    .fetch_sub(bytes, Ordering::Relaxed);
            }
            MemRegion::IndexCache => {
                self.current_index_bytes.fetch_sub(bytes, Ordering::Relaxed);
            }
            MemRegion::VectorCache => {
                self.current_vector_bytes
                    .fetch_sub(bytes, Ordering::Relaxed);
            }
        }
    }

    pub fn is_pressure(&self, region: MemRegion) -> bool {
        let (current, max) = match region {
            MemRegion::MemTable => (
                self.current_memtable_bytes.load(Ordering::Relaxed),
                self.budget.max_memtable_bytes,
            ),
            MemRegion::IndexCache => (
                self.current_index_bytes.load(Ordering::Relaxed),
                self.budget.max_index_cache_bytes,
            ),
            MemRegion::VectorCache => (
                self.current_vector_bytes.load(Ordering::Relaxed),
                self.budget.max_vector_cache_bytes,
            ),
        };
        current * 100 >= max * 95
    }

    pub fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "memtable_bytes": self.current_memtable_bytes.load(Ordering::Relaxed),
            "index_cache_bytes": self.current_index_bytes.load(Ordering::Relaxed),
            "vector_cache_bytes": self.current_vector_bytes.load(Ordering::Relaxed),
            "pressure": {
                "memtable": self.is_pressure(MemRegion::MemTable),
                "index": self.is_pressure(MemRegion::IndexCache),
                "vector": self.is_pressure(MemRegion::VectorCache),
            }
        })
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new(MemoryBudget::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        let tracker = MemoryTracker::default();
        tracker.track_alloc(MemRegion::MemTable, 1024);
        assert!(!tracker.is_pressure(MemRegion::MemTable));
        tracker.track_free(MemRegion::MemTable, 1024);
    }
}
