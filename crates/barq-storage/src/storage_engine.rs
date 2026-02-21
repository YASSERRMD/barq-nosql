use crate::memtable::MemTable;
use crate::sstable::{SsTableReader, SsTableWriter, SsTableEntry};
use crate::wal::{WalEntry, WalOp, WalReader, WalWriter};
use barq_core::{Document, DocumentId};
use parking_lot::Mutex;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub wal_enabled: bool,
    pub flush_threshold_docs: usize,
    pub flush_threshold_bytes: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data"),
            wal_enabled: true,
            flush_threshold_docs: 10_000,
            flush_threshold_bytes: 64 * 1024 * 1024,
        }
    }
}

pub struct StorageEngine {
    wal: Option<Mutex<WalWriter>>,
    memtable: Arc<RwLock<MemTable>>,
    sstables: Arc<RwLock<Vec<SsTableReader>>>,
    collection_dir: PathBuf,
}

impl StorageEngine {
    pub async fn new(config: StorageConfig) -> Result<Self, std::io::Error> {
        fs::create_dir_all(&config.data_dir)?;
        
        let collection_dir = config.data_dir.clone();
        
        let memtable = Arc::new(RwLock::new(MemTable::with_thresholds(
            config.flush_threshold_docs,
            config.flush_threshold_bytes,
        )));
        
        let wal = if config.wal_enabled {
            let wal_path = config.data_dir.join("wal.log");
            Some(Mutex::new(WalWriter::new(wal_path)?))
        } else {
            None
        };
        
        let sstables = Arc::new(RwLock::new(Vec::new()));
        
        Ok(Self {
            wal,
            memtable,
            sstables,
            collection_dir,
        })
    }

    pub async fn insert(
        &self,
        collection: &str,
        doc_id: DocumentId,
        doc: Document,
    ) -> Result<(), barq_core::BarqError> {
        let collection_id = barq_core::CollectionId::new(collection.to_string());
        
        if let Some(ref wal) = self.wal {
            let mut wal = wal.lock();
            let entry = WalEntry::new(
                WalOp::Insert,
                collection_id.clone(),
                doc_id.clone(),
                Some(doc.clone()),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            wal.write(&entry).map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
        }
        
        let memtable = self.memtable.read().await;
        memtable.insert(doc_id.clone(), doc);
        
        if memtable.should_flush() {
            drop(memtable);
            self.flush_memtable().await?;
        }
        
        Ok(())
    }

    pub async fn get(
        &self,
        _collection: &str,
        doc_id: &DocumentId,
    ) -> Result<Option<Document>, barq_core::BarqError> {
        let memtable = self.memtable.read().await;
        
        if let Some(doc) = memtable.get(doc_id) {
            return Ok(Some(doc));
        }
        drop(memtable);
        
        let mut sstables = self.sstables.write().await;
        for sstable in sstables.iter_mut().rev() {
            if let Ok(Some(doc)) = sstable.get(doc_id) {
                return Ok(Some(doc));
            }
        }
        
        Ok(None)
    }

    pub async fn delete(
        &self,
        collection: &str,
        doc_id: &DocumentId,
    ) -> Result<(), barq_core::BarqError> {
        let collection_id = barq_core::CollectionId::new(collection.to_string());
        
        if let Some(ref wal) = self.wal {
            let mut wal = wal.lock();
            let entry = WalEntry::new(
                WalOp::Delete,
                collection_id,
                doc_id.clone(),
                None,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            wal.write(&entry).map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
        }
        
        let memtable = self.memtable.read().await;
        memtable.delete(doc_id);
        
        Ok(())
    }

    pub async fn flush_memtable(&self) -> Result<(), barq_core::BarqError> {
        let batch = {
            let memtable = self.memtable.read().await;
            memtable.flush_batch()
        };
        
        if batch.items.is_empty() {
            return Ok(());
        }
        
        let sstable_path = self.collection_dir.join(format!(
            "sst_{}.sst",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        
        let mut writer = SsTableWriter::new(&sstable_path)
            .map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
        
        for (doc_id, doc) in batch.items {
            writer.add_entry(SsTableEntry { doc_id, doc });
        }
        
        writer.write()
            .map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
        
        let reader = SsTableReader::new(&sstable_path)
            .map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
        
        let mut sstables = self.sstables.write().await;
        sstables.push(reader);
        
        info!("Flushed memtable to SSTable");
        
        Ok(())
    }

    pub async fn compact(&self) -> Result<(), barq_core::BarqError> {
        warn!("Compaction triggered");
        Ok(())
    }

    pub async fn recover(&self) -> Result<(), barq_core::BarqError> {
        if let Some(ref wal) = self.wal {
            let wal_path = wal.lock().path().clone();
            if wal_path.exists() {
                let mut reader = WalReader::new(wal_path)
                    .map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
                
                let entries = reader.replay()
                    .map_err(|e| barq_core::BarqError::StorageError(e.to_string()))?;
                
                let memtable = self.memtable.read().await;
                for entry in entries {
                    if let Some(doc) = entry.data {
                        if matches!(entry.op, WalOp::Insert | WalOp::Update) {
                            memtable.insert(entry.doc_id, doc);
                        }
                    } else if matches!(entry.op, WalOp::Delete) {
                        memtable.delete(&entry.doc_id);
                    }
                }
                
                info!("Recovered entries from WAL");
            }
        }
        
        Ok(())
    }

    pub async fn list_collections(&self) -> Result<Vec<String>, std::io::Error> {
        let mut collections = Vec::new();
        
        if self.collection_dir.exists() {
            for entry in fs::read_dir(&self.collection_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    collections.push(entry.file_name().to_string_lossy().to_string());
                }
            }
        }
        
        Ok(collections)
    }
}
