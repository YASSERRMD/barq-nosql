use barq_core::{Document, DocumentId, Value};
use crc32fast::Hasher as Crc32;
use lz4_flex::compress_prepend_size;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

const BLOCK_SIZE: usize = 4096;
const INDEX_INTERVAL: usize = 128;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsTableEntry {
    pub doc_id: DocumentId,
    pub doc: Document,
}

pub struct SsTableWriter {
    file: File,
    path: PathBuf,
    entries: Vec<SsTableEntry>,
    index: BTreeMap<DocumentId, u64>,
    next_index_offset: u64,
    entry_count: usize,
}

impl SsTableWriter {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let path = path.into();
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path)?;

        Ok(Self {
            file,
            path,
            entries: Vec::new(),
            index: BTreeMap::new(),
            next_index_offset: 0,
            entry_count: 0,
        })
    }

    pub fn add_entry(&mut self, entry: SsTableEntry) {
        if self.entry_count % INDEX_INTERVAL == 0 {
            self.index
                .insert(entry.doc_id.clone(), self.next_index_offset);
        }
        self.entries.push(entry);
        self.entry_count += 1;
    }

    pub fn write(&mut self) -> Result<(), std::io::Error> {
        let mut block_buffer = Vec::new();

        let entries = std::mem::take(&mut self.entries);
        let count = entries.len();

        for (i, entry) in entries.into_iter().enumerate() {
            let serialized = serde_json::to_vec(&entry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

            let compressed = compress_prepend_size(&serialized);
            let mut hasher = Crc32::new();
            hasher.update(&serialized);
            let checksum = hasher.finalize();

            let mut block = Vec::new();
            block.write_all(&checksum.to_le_bytes())?;
            let len = compressed.len() as u32;
            block.write_all(&len.to_le_bytes())?;
            block.extend_from_slice(&compressed);

            block_buffer.extend_from_slice(&block);

            if block_buffer.len() >= BLOCK_SIZE || i == count - 1 {
                self.file.write_all(&block_buffer)?;
                self.next_index_offset += block_buffer.len() as u64;
                block_buffer.clear();
            }
        }

        let index_data = serde_json::to_vec(&self.index)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        let index_compressed = compress_prepend_size(&index_data);
        let mut hasher = Crc32::new();
        hasher.update(&index_data);
        let index_checksum = hasher.finalize();

        self.file.write_all(&index_checksum.to_le_bytes())?;
        let index_len = index_compressed.len() as u32;
        self.file.write_all(&index_len.to_le_bytes())?;
        self.file.write_all(&index_compressed)?;

        let meta_offset = self.file.stream_position()?;
        let meta = SsTableMeta {
            index_offset: meta_offset,
            entry_count: self.entry_count,
        };
        let meta_data = serde_json::to_vec(&meta)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.file.write_all(&meta_data)?;

        self.file.sync_all()?;
        Ok(())
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsTableMeta {
    pub index_offset: u64,
    pub entry_count: usize,
}

pub struct SsTableReader {
    file: File,
    path: PathBuf,
    index: BTreeMap<DocumentId, u64>,
    meta: SsTableMeta,
}

impl SsTableReader {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, std::io::Error> {
        let path = path.into();
        let mut file = OpenOptions::new().read(true).open(&path)?;

        let meta_offset = file.metadata()?.len() - 1024;
        file.seek(SeekFrom::Start(meta_offset))?;

        let mut meta_len_buf = [0u8; 4];
        file.read_exact(&mut meta_len_buf)?;
        let meta_len = u32::from_le_bytes(meta_len_buf) as usize;

        let mut meta_buf = vec![0u8; meta_len];
        file.read_exact(&mut meta_buf)?;
        let meta: SsTableMeta = serde_json::from_slice(&meta_buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        file.seek(SeekFrom::Start(meta.index_offset - 8))?;
        let mut index_len_buf = [0u8; 4];
        file.read_exact(&mut index_len_buf)?;
        let index_len = u32::from_le_bytes(index_len_buf) as usize;

        let mut index_checksum_buf = [0u8; 4];
        file.read_exact(&mut index_checksum_buf)?;

        let mut index_compressed = vec![0u8; index_len];
        file.read_exact(&mut index_compressed)?;

        let index_data = lz4_flex::decompress_size_prepended(&index_compressed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let index: BTreeMap<DocumentId, u64> = serde_json::from_slice(&index_data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Self {
            file,
            path,
            index,
            meta,
        })
    }

    pub fn get(&mut self, doc_id: &DocumentId) -> Result<Option<Document>, std::io::Error> {
        let offset = match self.index.get(doc_id) {
            Some(offset) => *offset,
            None => return Ok(None),
        };

        self.file.seek(SeekFrom::Start(offset))?;

        let mut checksum_buf = [0u8; 4];
        self.file.read_exact(&mut checksum_buf)?;
        let expected_checksum = u32::from_le_bytes(checksum_buf);

        let mut len_buf = [0u8; 4];
        self.file.read_exact(&mut len_buf)?;
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut compressed = vec![0u8; len];
        self.file.read_exact(&mut compressed)?;

        let data = lz4_flex::decompress_size_prepended(&compressed)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let mut hasher = Crc32::new();
        hasher.update(&data);
        let computed_checksum = hasher.finalize();
        if computed_checksum != expected_checksum {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Checksum mismatch",
            ));
        }

        let entry: SsTableEntry = serde_json::from_slice(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        Ok(Some(entry.doc))
    }

    pub fn range_query(
        &mut self,
        start: &DocumentId,
        end: &DocumentId,
    ) -> Result<Vec<(DocumentId, Document)>, std::io::Error> {
        let mut results = Vec::new();

        let keys: Vec<DocumentId> = self
            .index
            .range(start..end)
            .map(|(k, _)| k.clone())
            .collect();

        for doc_id in keys {
            if let Some(doc) = self.get(&doc_id)? {
                results.push((doc_id, doc));
            }
        }

        Ok(results)
    }

    pub fn entry_count(&self) -> usize {
        self.meta.entry_count
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sstable_write_read() {
        let temp_dir = TempDir::new().unwrap();
        let sstable_path = temp_dir.path().join("test.sst");

        let mut writer = SsTableWriter::new(&sstable_path).unwrap();

        for i in 0..10 {
            let doc = Document::with_data(
                DocumentId::new(),
                vec![("idx".to_string(), Value::Int(i))]
                    .into_iter()
                    .collect(),
            );
            writer.add_entry(SsTableEntry {
                doc_id: doc.id.clone(),
                doc,
            });
        }

        writer.write().unwrap();

        let mut reader = SsTableReader::new(&sstable_path).unwrap();
        let doc = reader.get(&DocumentId::default()).unwrap();
        assert!(doc.is_some());
    }
}
