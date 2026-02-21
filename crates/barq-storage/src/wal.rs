use barq_core::CollectionId;
use barq_core::{Document, DocumentId};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::path::PathBuf;

pub const WAL_FILE_NAME: &str = "wal.log";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WalOp {
    Insert,
    Update,
    Delete,
    TxnBatch(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalEntry {
    pub op: WalOp,
    pub collection: CollectionId,
    pub doc_id: DocumentId,
    pub data: Option<Document>,
    pub timestamp: u64,
    pub checksum: u32,
}

impl WalEntry {
    pub fn new(
        op: WalOp,
        collection: CollectionId,
        doc_id: DocumentId,
        data: Option<Document>,
        timestamp: u64,
    ) -> Self {
        let checksum = Self::calculate_checksum(&op, &collection, &doc_id, &data);
        Self {
            op,
            collection,
            doc_id,
            data,
            timestamp,
            checksum,
        }
    }

    fn calculate_checksum(
        op: &WalOp,
        collection: &CollectionId,
        doc_id: &DocumentId,
        data: &Option<Document>,
    ) -> u32 {
        use crc32fast::Hasher as Crc32;

        let mut hasher = Crc32::new();

        let op_bytes = serde_json::to_vec(op).unwrap_or_default();
        hasher.update(&op_bytes);

        let col_bytes = serde_json::to_vec(collection).unwrap_or_default();
        hasher.update(&col_bytes);

        let id_bytes = serde_json::to_vec(doc_id).unwrap_or_default();
        hasher.update(&id_bytes);

        if let Some(doc) = data {
            let doc_bytes = serde_json::to_vec(doc).unwrap_or_default();
            hasher.update(&doc_bytes);
        }

        hasher.finalize()
    }

    pub fn verify_checksum(&self) -> bool {
        let computed =
            Self::calculate_checksum(&self.op, &self.collection, &self.doc_id, &self.data);
        computed == self.checksum
    }
}

pub struct WalWriter {
    file: std::fs::File,
    path: PathBuf,
}

impl WalWriter {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let path = path.into();
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        Ok(Self { file, path })
    }

    pub fn write(&mut self, entry: &WalEntry) -> Result<(), std::io::Error> {
        let serialized = serde_json::to_vec(entry)?;
        let len = serialized.len() as u32;

        self.file.write_all(&len.to_le_bytes())?;
        self.file.write_all(&serialized)?;
        self.file.flush()?;

        Ok(())
    }

    pub fn sync(&mut self) -> Result<(), std::io::Error> {
        self.file.sync_all()
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

pub struct WalReader {
    file: std::fs::File,
}

impl WalReader {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let path = path.into();
        let file = std::fs::File::open(path)?;
        Ok(Self { file })
    }

    pub fn replay(&mut self) -> Result<Vec<WalEntry>, std::io::Error> {
        let mut entries = Vec::new();
        let mut buffer = Vec::new();

        loop {
            let mut len_buf = [0u8; 4];
            match self.file.read_exact(&mut len_buf) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };

            let len = u32::from_le_bytes(len_buf) as usize;
            buffer.resize(len, 0);
            self.file.read_exact(&mut buffer)?;

            let entry: WalEntry = serde_json::from_slice(&buffer)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            if !entry.verify_checksum() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "WAL checksum mismatch",
                ));
            }

            entries.push(entry);
        }

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_wal_write_read() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test_wal.log");

        let mut writer = WalWriter::new(&wal_path).unwrap();

        let entry = WalEntry::new(
            WalOp::Insert,
            CollectionId::new("users"),
            DocumentId::new(),
            Some(Document::new(DocumentId::new())),
            12345,
        );

        writer.write(&entry).unwrap();
        writer.sync().unwrap();

        let mut reader = WalReader::new(&wal_path).unwrap();
        let entries = reader.replay().unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].op, WalOp::Insert);
    }
}
