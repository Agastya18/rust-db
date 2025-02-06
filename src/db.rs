use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};

#[derive(Serialize, Deserialize, Debug)]
pub struct KVPair {
    pub key: String,
    pub value: String,
}

pub struct SimpleDB {
    pub data: HashMap<String, u64>, // In-memory index: key to file offset
    pub file: File,                 // Persistent storage file
    pub path: String,               // File path
}

impl SimpleDB {
    /// Opens (or creates) the database file and loads the in-memory index.
    pub fn open(path: &str) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(path)?;
        let mut db = SimpleDB {
            data: HashMap::new(),
            file,
            path: path.to_string(),
        };
        db.load_index()?;
        Ok(db)
    }

    /// Loads the in-memory index by scanning the file.
    fn load_index(&mut self) -> std::io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut reader = BufReader::new(&self.file);
        let mut offset = 0;
        let mut line = String::new();

        while reader.read_line(&mut line)? > 0 {
            let trimmed = line.trim_end();
            if !trimmed.is_empty() {
                let entry: KVPair = serde_json::from_str(trimmed)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
                self.data.insert(entry.key, offset);
            }
            offset += line.len() as u64;
            line.clear();
        }
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }

    /// Appends a key‑value pair to the file and updates the in-memory index.
    pub fn set(&mut self, key: String, value: String) -> std::io::Result<()> {
        let entry = KVPair { key: key.clone(), value };
        let json_line = serde_json::to_string(&entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let offset = self.file.seek(SeekFrom::End(0))?;
        writeln!(self.file, "{}", json_line)?;
        self.file.flush()?;

        self.data.insert(key, offset);
        Ok(())
    }

    /// Retrieves a key's value by seeking to its stored offset.
    pub fn get(&mut self, key: &str) -> std::io::Result<String> {
        if let Some(&offset) = self.data.get(key) {
            self.file.seek(SeekFrom::Start(offset))?;
            let mut reader = BufReader::new(&self.file);
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let entry: KVPair = serde_json::from_str(line.trim_end())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            Ok(entry.value)
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))
        }
    }

    /// Deletes a key by removing it from the in-memory index.
    pub fn delete(&mut self, key: &str) -> std::io::Result<()> {
        if self.data.remove(key).is_some() {
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Key not found"))
        }
    }
}
