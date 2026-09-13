use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::Write;
use std::ops;
use std::path::Path;

use lz4_flex::decompress_into;

use porter_macros::assert_size;

use crate::StringReadExt;
use crate::StringWriteExt;
use crate::StructReadExt;
use crate::StructWriteExt;
use crate::VecExt;
use crate::VecReadExt;

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

assert_size!(NameDatabaseHeader, 16);

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
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        if header.entries == 0 {
            return Ok(Self::new());
        }

        let compressed: Vec<u8> = file.read_vec(header.compressed_size as _)?;

        let mut decompressed = Vec::try_new_zeroed(header.decompressed_size as usize)?;

        decompress_into(&compressed, &mut decompressed)
            .map_err(|_| io::Error::from(io::ErrorKind::InvalidData))?;

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

        let mut entries: Vec<(&u64, &String)> = self.inner.iter().collect();
        let mut decompressed: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        entries.sort_unstable_by(|a, b| a.1.cmp(b.1));

        for (_, value) in &entries {
            decompressed.write_null_terminated_string(value)?;
        }

        for (key, _) in &entries {
            decompressed.write_all(&key.to_le_bytes())?;
        }

        let decompressed = decompressed.into_inner();
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
