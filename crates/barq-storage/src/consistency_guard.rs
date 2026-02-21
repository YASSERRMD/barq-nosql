use crate::consistency::{ConsistencyLevel, ReadOptions};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

pub struct ConsistencyGuard {
    level: ConsistencyLevel,
    version_counter: Arc<AtomicU64>,
}

impl ConsistencyGuard {
    pub fn new(level: ConsistencyLevel, version_counter: Arc<AtomicU64>) -> Self {
        Self { level, version_counter }
    }

    pub fn resolve_version(&self, _options: &ReadOptions) -> u64 {
        match self.level {
            ConsistencyLevel::Strong => {
                self.version_counter.load(Ordering::Acquire)
            }
            ConsistencyLevel::Bounded { max_staleness_ms: _ } => {
                self.version_counter.load(Ordering::Acquire)
            }
            ConsistencyLevel::Eventual => {
                self.version_counter.load(Ordering::Relaxed)
            }
            ConsistencyLevel::Session { ref token: _ } => {
                self.version_counter.load(Ordering::Acquire)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strong_consistency() {
        let counter = Arc::new(AtomicU64::new(42));
        let guard = ConsistencyGuard::new(ConsistencyLevel::Strong, counter);
        let opts = ReadOptions::default();
        
        let version = guard.resolve_version(&opts);
        assert_eq!(version, 42);
    }
}
