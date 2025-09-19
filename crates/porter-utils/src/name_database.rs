use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::Write;
use std::ops;
use std::path::Path;

use lz4_flex::decompress_into;

use crate::StringReadExt;
use crate::StructReadExt;
use crate::StructWriteExt;
use crate::VecExt;

/// A database of asset hash:name pairs used to link a packed asset to it's source name.
#[repr(transparent)]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct NameDatabase {
    inner: HashMap<u64, String>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NameDatabaseHeader {
    magic: u32,
    entries: u32,
    compressed_size: u32,
    decompressed_size: u32,
}

impl NameDatabase {
    /// Constructs a new name database.
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    /// Extends a name database from the entries of another.
    pub fn extend_from(&mut self, database: Self) {
        self.inner.extend(database.inner);
    }

    /// Loads a name database from the given file path.
    pub fn load<P: AsRef<Path>>(file: P) -> Result<Self, io::Error> {
        let mut file = File::open(file.as_ref())?;

        let header: NameDatabaseHeader = file.read_struct()?;

        if header.magic != 0x42444E50 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        if header.entries == 0 {
            return Ok(Self::new());
        }

        let mut compressed = Vec::try_new_with_value(0, header.compressed_size as usize)?;

        file.read_exact(&mut compressed)?;

        let mut decompressed = Vec::try_new_with_value(0, header.decompressed_size as usize)?;

        decompress_into(&compressed, &mut decompressed)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let mut keys: Vec<u64> = Vec::try_with_exact_capacity(header.entries as usize)?;
        let mut values: Vec<String> = Vec::try_with_exact_capacity(header.entries as usize)?;

        let mut file = Cursor::new(decompressed);

        for _ in 0..header.entries {
            values.push(file.read_null_terminated_string()?);
        }

        for _ in 0..header.entries {
            keys.push(file.read_struct()?);
        }

        if keys.len() != values.len() {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }

        Ok(Self {
            inner: keys.into_iter().zip(values).collect(),
        })
    }

    /// Saves a name database with the current entries to the given file path.
    pub fn save<P: AsRef<Path>>(&self, file: P) -> Result<(), io::Error> {
        let mut file = File::create(file.as_ref())?;

        let mut keys: Vec<u64> = Vec::try_with_exact_capacity(self.inner.len())?;

        let mut decompressed: Vec<u8> = Vec::new();

        for entry in self.inner.iter() {
            keys.push(*entry.0);

            decompressed.extend_from_slice(entry.1.as_bytes());
            decompressed.extend_from_slice(&[0]);
        }

        for key in keys.into_iter() {
            decompressed.extend_from_slice(&key.to_le_bytes());
        }

        let compressed = lz4_flex::compress(&decompressed);

        let header = NameDatabaseHeader {
            magic: 0x42444E50,
            entries: self.inner.len() as u32,
            compressed_size: compressed.len() as u32,
            decompressed_size: decompressed.len() as u32,
        };

        file.write_struct(header)?;
        file.write_all(&compressed)?;

        Ok(())
    }
}

impl ops::Deref for NameDatabase {
    type Target = HashMap<u64, String>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ops::DerefMut for NameDatabase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
