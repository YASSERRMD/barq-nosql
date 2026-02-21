use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaMode {
    RejectWrites,
    WarnAndContinue,
    SoftLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionQuota {
    pub max_documents: usize,
    pub max_storage_bytes: usize,
    pub max_indexes: usize,
    pub max_daily_writes: usize,
    pub enforcement: QuotaMode,
}

impl Default for CollectionQuota {
    fn default() -> Self {
        Self {
            max_documents: 1_000_000,
            max_storage_bytes: 10 * 1024 * 1024 * 1024,
            max_indexes: 10,
            max_daily_writes: 1_000_000,
            enforcement: QuotaMode::WarnAndContinue,
        }
    }
}

impl CollectionQuota {
    pub fn new(max_documents: usize, max_storage_bytes: usize) -> Self {
        Self {
            max_documents,
            max_storage_bytes,
            max_indexes: 10,
            max_daily_writes: 1_000_000,
            enforcement: QuotaMode::WarnAndContinue,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionUsage {
    pub documents: usize,
    pub storage_bytes: usize,
    pub indexes: usize,
    pub daily_writes: usize,
}

impl Default for CollectionUsage {
    fn default() -> Self {
        Self {
            documents: 0,
            storage_bytes: 0,
            indexes: 0,
            daily_writes: 0,
        }
    }
}

#[derive(Debug)]
pub enum QuotaError {
    DocumentsExceeded,
    StorageExceeded,
    IndexesExceeded,
    WritesExceeded,
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaError::DocumentsExceeded => write!(f, "Maximum documents quota exceeded"),
            QuotaError::StorageExceeded => write!(f, "Maximum storage quota exceeded"),
            QuotaError::IndexesExceeded => write!(f, "Maximum indexes quota exceeded"),
            QuotaError::WritesExceeded => write!(f, "Maximum daily writes quota exceeded"),
        }
    }
}

impl std::error::Error for QuotaError {}

pub trait QuotaEnforcer {
    fn check_write(&self, collection: &str) -> Result<(), QuotaError>;
    fn get_usage(&self, collection: &str) -> CollectionUsage;
    fn set_quota(&self, collection: &str, quota: CollectionQuota);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_quota() {
        let quota = CollectionQuota::default();
        assert_eq!(quota.max_documents, 1_000_000);
    }
}
