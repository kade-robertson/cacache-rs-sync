//! Functions for reading from cache.
use std::path::Path;

use ssri::{Algorithm, Integrity};

use crate::content::read;
use crate::errors::{Error, Result};
use crate::index::{self, Metadata};

// ---------------
// Synchronous API
// ---------------

/// File handle for reading data synchronously.
///
/// Make sure to call `get.check()` when done reading
/// to verify that the extracted data passes integrity
/// verification.
pub struct SyncReader {
    reader: read::Reader,
}

impl std::io::Read for SyncReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
}

impl SyncReader {
    /// Checks that data read from disk passes integrity checks. Returns the
    /// algorithm that was used verified the data. Should be called only after
    /// all data has been read from disk.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::Read;
    ///
    /// fn main() -> cacache::Result<()> {
    ///     let mut fd = cacache::SyncReader::open("./my-cache", "my-key")?;
    ///     let mut str = String::new();
    ///     fd.read_to_string(&mut str).expect("Failed to read to string");
    ///     // Remember to check that the data you got was correct!
    ///     fd.check()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn check(self) -> Result<Algorithm> {
        self.reader.check()
    }

    /// Opens a new synchronous file handle into the cache, looking it up in the
    /// index using `key`.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::Read;
    ///
    /// fn main() -> cacache::Result<()> {
    ///     let mut fd = cacache::SyncReader::open("./my-cache", "my-key")?;
    ///     let mut str = String::new();
    ///     fd.read_to_string(&mut str).expect("Failed to parse string");
    ///     // Remember to check that the data you got was correct!
    ///     fd.check()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open<P, K>(cache: P, key: K) -> Result<SyncReader>
    where
        P: AsRef<Path>,
        K: AsRef<str>,
    {
        if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
            SyncReader::open_hash(cache, entry.integrity)
        } else {
            return Err(Error::EntryNotFound(
                cache.as_ref().to_path_buf(),
                key.as_ref().into(),
            ));
        }
    }

    /// Opens a new synchronous file handle into the cache, based on its integrity address.
    ///
    /// ## Example
    /// ```no_run
    /// use std::io::Read;
    ///
    /// fn main() -> cacache::Result<()> {
    ///     let sri = cacache::write_sync("./my-cache", "key", b"hello world")?;
    ///     let mut fd = cacache::SyncReader::open_hash("./my-cache", sri)?;
    ///     let mut str = String::new();
    ///     fd.read_to_string(&mut str).expect("Failed to read to string");
    ///     // Remember to check that the data you got was correct!
    ///     fd.check()?;
    ///     Ok(())
    /// }
    /// ```
    pub fn open_hash<P>(cache: P, sri: Integrity) -> Result<SyncReader>
    where
        P: AsRef<Path>,
    {
        Ok(SyncReader {
            reader: read::open(cache.as_ref(), sri)?,
        })
    }
}

/// Reads the entire contents of a cache file synchronously into a bytes
/// vector, looking the data up by key.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let data = cacache::read_sync("./my-cache", "my-key")?;
///     Ok(())
/// }
/// ```
pub fn read_sync<P, K>(cache: P, key: K) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        read_hash_sync(cache, &entry.integrity)
    } else {
        return Err(Error::EntryNotFound(
            cache.as_ref().to_path_buf(),
            key.as_ref().into(),
        ));
    }
}

/// Reads the entire contents of a cache file synchronously into a bytes
/// vector, looking the data up by its content address.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let sri = cacache::write_sync("./my-cache", "my-key", b"hello")?;
///     let data = cacache::read_hash_sync("./my-cache", &sri)?;
///     Ok(())
/// }
/// ```
pub fn read_hash_sync<P>(cache: P, sri: &Integrity) -> Result<Vec<u8>>
where
    P: AsRef<Path>,
{
    read::read(cache.as_ref(), sri)
}

/// Copies a cache entry by key to a specified location. Returns the number of
/// bytes copied.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     cacache::copy_sync("./my-cache", "my-key", "./my-hello.txt")?;
///     Ok(())
/// }
/// ```
pub fn copy_sync<P, K, Q>(cache: P, key: K, to: Q) -> Result<u64>
where
    P: AsRef<Path>,
    K: AsRef<str>,
    Q: AsRef<Path>,
{
    if let Some(entry) = index::find(cache.as_ref(), key.as_ref())? {
        copy_hash_sync(cache, &entry.integrity, to)
    } else {
        return Err(Error::EntryNotFound(
            cache.as_ref().to_path_buf(),
            key.as_ref().into(),
        ));
    }
}

/// Copies a cache entry by integrity address to a specified location. Returns
/// the number of bytes copied.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache::Result<()> {
///     let sri = cacache::write_sync("./my-cache", "my-key", b"hello")?;
///     cacache::copy_hash_sync("./my-cache", &sri, "./my-hello.txt")?;
///     Ok(())
/// }
/// ```
pub fn copy_hash_sync<P, Q>(cache: P, sri: &Integrity, to: Q) -> Result<u64>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    read::copy(cache.as_ref(), sri, to.as_ref())
}

/// Gets metadata for a certain key.
///
/// Note that the existence of a metadata entry is not a guarantee that the
/// underlying data exists, since they are stored and managed independently.
/// To verify that the underlying associated data exists, use `exists_sync()`.
pub fn metadata_sync<P, K>(cache: P, key: K) -> Result<Option<Metadata>>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::find(cache.as_ref(), key.as_ref())
}

/// Returns true if the given hash exists in the cache.
pub fn exists_sync<P: AsRef<Path>>(cache: P, sri: &Integrity) -> bool {
    read::has_content(cache.as_ref(), sri).is_some()
}

#[cfg(test)]
mod tests {
    use std::fs;

    #[test]
    fn test_open_sync() {
        use std::io::prelude::*;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::write_sync(&dir, "my-key", b"hello world").unwrap();

        let mut handle = crate::SyncReader::open(&dir, "my-key").unwrap();
        let mut str = String::new();
        handle.read_to_string(&mut str).unwrap();
        handle.check().unwrap();
        assert_eq!(str, String::from("hello world"));
    }

    #[test]
    fn test_open_hash_sync() {
        use std::io::prelude::*;
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write_sync(&dir, "my-key", b"hello world").unwrap();

        let mut handle = crate::SyncReader::open_hash(&dir, sri).unwrap();
        let mut str = String::new();
        handle.read_to_string(&mut str).unwrap();
        handle.check().unwrap();
        assert_eq!(str, String::from("hello world"));
    }

    #[test]
    fn test_read_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        crate::write_sync(&dir, "my-key", b"hello world").unwrap();

        let data = crate::read_sync(&dir, "my-key").unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_read_hash_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write_sync(&dir, "my-key", b"hello world").unwrap();

        let data = crate::read_hash_sync(&dir, &sri).unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_copy_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let dest = dir.join("data");
        crate::write_sync(dir, "my-key", b"hello world").unwrap();

        crate::copy_sync(dir, "my-key", &dest).unwrap();
        let data = fs::read(&dest).unwrap();
        assert_eq!(data, b"hello world");
    }

    #[test]
    fn test_copy_hash_sync() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path();
        let dest = dir.join("data");
        let sri = crate::write_sync(dir, "my-key", b"hello world").unwrap();

        crate::copy_hash_sync(dir, &sri, &dest).unwrap();
        let data = fs::read(&dest).unwrap();
        assert_eq!(data, b"hello world");
    }
}
