use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    Strong,
    Bounded { max_staleness_ms: u64 },
    Eventual,
    Session { token: String },
}

impl Default for ConsistencyLevel {
    fn default() -> Self {
        ConsistencyLevel::Strong
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOptions {
    pub consistency: ConsistencyLevel,
    pub max_staleness_ms: Option<u64>,
    pub session_token: Option<String>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            consistency: ConsistencyLevel::Strong,
            max_staleness_ms: None,
            session_token: None,
        }
    }
}

impl ReadOptions {
    pub fn strong() -> Self {
        Self::default()
    }

    pub fn bounded(max_staleness_ms: u64) -> Self {
        Self {
            consistency: ConsistencyLevel::Bounded { max_staleness_ms },
            max_staleness_ms: Some(max_staleness_ms),
            session_token: None,
        }
    }

    pub fn eventual() -> Self {
        Self {
            consistency: ConsistencyLevel::Eventual,
            max_staleness_ms: None,
            session_token: None,
        }
    }

    pub fn session(token: String) -> Self {
        Self {
            consistency: ConsistencyLevel::Session {
                token: token.clone(),
            },
            max_staleness_ms: None,
            session_token: Some(token),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_strong() {
        let opts = ReadOptions::default();
        assert!(matches!(opts.consistency, ConsistencyLevel::Strong));
    }

    #[test]
    fn test_bounded() {
        let opts = ReadOptions::bounded(500);
        assert!(matches!(opts.consistency, ConsistencyLevel::Bounded { .. }));
    }
}
