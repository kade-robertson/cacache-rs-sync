//! Functions for writing to cache.
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use serde_json::Value;
use ssri::{Algorithm, Integrity};

use crate::content::write;
use crate::errors::{Error, Internal, Result};
use crate::index;

/// Writes `data` to the `cache` synchronously, indexing it under `key`.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache_sync::Result<()> {
///     let data = cacache_sync::write("./my-cache", "my-key", b"hello")?;
///     Ok(())
/// }
/// ```
pub fn write<P, D, K>(cache: P, key: K, data: D) -> Result<Integrity>
where
    P: AsRef<Path>,
    D: AsRef<[u8]>,
    K: AsRef<str>,
{
    let mut writer = Writer::create(cache.as_ref(), key.as_ref())?;
    writer.write_all(data.as_ref()).with_context(|| {
        format!(
            "Failed to write to cache data for key {} for cache at {:?}",
            key.as_ref(),
            cache.as_ref()
        )
    })?;
    writer.written = data.as_ref().len();
    writer.commit()
}

/// Writes `data` to the `cache` synchronously, skipping associating a key with it.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache_sync::Result<()> {
///     let data = cacache_sync::write_hash("./my-cache", b"hello")?;
///     Ok(())
/// }
/// ```
pub fn write_hash<P, D>(cache: P, data: D) -> Result<Integrity>
where
    P: AsRef<Path>,
    D: AsRef<[u8]>,
{
    let mut writer = WriteOpts::new()
        .algorithm(Algorithm::Sha256)
        .size(data.as_ref().len())
        .open_hash(cache.as_ref())?;
    writer.write_all(data.as_ref()).with_context(|| {
        format!(
            "Failed to write to cache data for cache at {:?}",
            cache.as_ref()
        )
    })?;
    writer.written = data.as_ref().len();
    writer.commit()
}

/// Builder for options and flags for opening a new cache file to write data into.
#[derive(Clone, Default)]
pub struct WriteOpts {
    pub(crate) algorithm: Option<Algorithm>,
    pub(crate) sri: Option<Integrity>,
    pub(crate) size: Option<usize>,
    pub(crate) time: Option<u128>,
    pub(crate) metadata: Option<Value>,
}

impl WriteOpts {
    /// Creates a blank set of cache writing options.
    pub fn new() -> WriteOpts {
        Default::default()
    }

    /// Opens the file handle for writing synchronously, returning a SyncWriter instance.
    pub fn open<P, K>(self, cache: P, key: K) -> Result<Writer>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        Ok(Writer {
            cache: cache.as_ref().to_path_buf(),
            key: Some(String::from(key.as_ref())),
            written: 0,
            writer: write::Writer::new(
                cache.as_ref(),
                *self.algorithm.as_ref().unwrap_or(&Algorithm::Sha256),
                self.size,
            )?,
            opts: self,
        })
    }

    /// Opens the file handle for writing, without a key returning an SyncWriter instance.
    pub fn open_hash<P>(self, cache: P) -> Result<Writer>
    where
        P: AsRef<Path>,
    {
        Ok(Writer {
            cache: cache.as_ref().to_path_buf(),
            key: None,
            written: 0,
            writer: write::Writer::new(
                cache.as_ref(),
                *self.algorithm.as_ref().unwrap_or(&Algorithm::Sha256),
                self.size,
            )?,
            opts: self,
        })
    }

    /// Configures the algorithm to write data under.
    pub fn algorithm(mut self, algo: Algorithm) -> Self {
        self.algorithm = Some(algo);
        self
    }

    /// Sets the expected size of the data to write. If there's a date size
    /// mismatch, `put.commit()` will return an error.
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets arbitrary additional metadata to associate with the index entry.
    pub fn metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the specific time in unix milliseconds to associate with this
    /// entry. This is usually automatically set to the write time, but can be
    /// useful to change for tests and such.
    pub fn time(mut self, time: u128) -> Self {
        self.time = Some(time);
        self
    }

    /// Sets the expected integrity hash of the written data. If there's a
    /// mismatch between this Integrity and the one calculated by the write,
    /// `put.commit()` will error.
    pub fn integrity(mut self, sri: Integrity) -> Self {
        self.sri = Some(sri);
        self
    }
}

/// A reference to an open file writing to the cache.
pub struct Writer {
    cache: PathBuf,
    key: Option<String>,
    written: usize,
    pub(crate) writer: write::Writer,
    opts: WriteOpts,
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self.writer.write(buf)?;
        self.written += written;
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl Writer {
    /// Creates a new writable file handle into the cache.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::prelude::*;
    ///
    /// fn main() -> cacache_sync::Result<()> {
    ///     let mut fd = cacache_sync::Writer::create("./my-cache", "my-key")?;
    ///     fd.write_all(b"hello world").expect("Failed to write to cache");
    ///     // Data is not saved into the cache until you commit it.
    ///     fd.commit()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn create<P, K>(cache: P, key: K) -> Result<Writer>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        WriteOpts::new()
            .algorithm(Algorithm::Sha256)
            .open(cache.as_ref(), key.as_ref())
    }

    /// Closes the Writer handle and writes content and index entries. Also
    /// verifies data against `size` and `integrity` options, if provided.
    /// Must be called manually in order to complete the writing process,
    /// otherwise everything will be thrown out.
    pub fn commit(mut self) -> Result<Integrity> {
        let cache = self.cache;
        let writer_sri = self.writer.close()?;
        if let Some(sri) = &self.opts.sri {
            if sri.matches(&writer_sri).is_none() {
                return Err(ssri::Error::IntegrityCheckError(sri.clone(), writer_sri).into());
            }
        } else {
            self.opts.sri = Some(writer_sri.clone());
        }
        if let Some(size) = self.opts.size {
            if size != self.written {
                return Err(Error::SizeError(size, self.written));
            }
        }
        if let Some(key) = self.key {
            index::insert(&cache, &key, self.opts)
        } else {
            Ok(writer_sri)
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::write(&dir, "hello", b"hello").unwrap();
        let data = crate::read(&dir, "hello").unwrap();
        assert_eq!(data, b"hello");
    }

    #[test]
    fn hash_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let original = format!("hello world{}", 5);
        let integrity = crate::write_hash(&dir, &original)
            .expect("should be able to write a hash synchronously");
        let bytes = crate::read_hash(&dir, &integrity)
            .expect("should be able to read the data we just wrote");
        let result =
            String::from_utf8(bytes).expect("we wrote valid utf8 but did not read valid utf8 back");
        assert_eq!(result, original, "we did not read back what we wrote");
    }
}
