//! Functions for removing things from the cache.
use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::rm;
use crate::errors::{Internal, Result};
use crate::index;

/// Removes an individual index entry synchronously. The associated content
/// will be left in the cache.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache_sync::Result<()> {
///     let sri = cacache_sync::write("./my-cache", "my-key", b"hello")?;
///
///     cacache_sync::remove("./my-cache", "my-key")?;
///
///     // This fails:
///     cacache_sync::read("./my-cache", "my-key")?;
///
///     // But this succeeds:
///     cacache_sync::read_hash("./my-cache", &sri)?;
///
///     Ok(())
/// }
/// ```
pub fn remove<P, K>(cache: P, key: K) -> Result<()>
where
    P: AsRef<Path>,
    K: AsRef<str>,
{
    index::delete(cache.as_ref(), key.as_ref())
}

/// Removes an individual content entry synchronously. Any index entries
/// pointing to this content will become invalidated.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache_sync::Result<()> {
///     let sri = cacache_sync::write("./my-cache", "my-key", b"hello")?;
///
///     cacache_sync::remove_hash("./my-cache", &sri)?;
///
///     // These fail:
///     cacache_sync::read("./my-cache", "my-key")?;
///     cacache_sync::read_hash("./my-cache", &sri)?;
///
///     // But this succeeds:
///     cacache_sync::metadata("./my-cache", "my-key")?;
///
///     Ok(())
/// }
/// ```
pub fn remove_hash<P: AsRef<Path>>(cache: P, sri: &Integrity) -> Result<()> {
    rm::rm(cache.as_ref(), sri)
}

/// Removes entire contents of the cache synchronously, including temporary
/// files, the entry index, and all content data.
///
/// ## Example
/// ```no_run
/// use std::io::Read;
///
/// fn main() -> cacache_sync::Result<()> {
///     let sri = cacache_sync::write("./my-cache", "my-key", b"hello")?;
///
///     cacache_sync::clear("./my-cache")?;
///
///     // These all fail:
///     cacache_sync::read("./my-cache", "my-key")?;
///     cacache_sync::read_hash("./my-cache", &sri)?;
///     cacache_sync::metadata("./my-cache", "my-key")?;
///
///     Ok(())
/// }
/// ```
pub fn clear<P: AsRef<Path>>(cache: P) -> Result<()> {
    for entry in (cache.as_ref().read_dir().to_internal()?).flatten() {
        fs::remove_dir_all(entry.path()).to_internal()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_remove() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write(&dir, "key", b"my-data").unwrap();

        crate::remove(&dir, "key").unwrap();

        let new_entry = crate::metadata(&dir, "key").unwrap();
        assert!(new_entry.is_none());

        let data_exists = crate::exists(&dir, &sri);
        assert!(data_exists);
    }

    #[test]
    fn test_remove_data() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write(&dir, "key", b"my-data").unwrap();

        crate::remove_hash(&dir, &sri).unwrap();

        let entry = crate::metadata(&dir, "key").unwrap();
        assert!(entry.is_some());

        let data_exists = crate::exists(&dir, &sri);
        assert!(!data_exists);
    }

    #[test]
    fn test_clear() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let sri = crate::write(&dir, "key", b"my-data").unwrap();

        crate::clear(&dir).unwrap();

        let entry = crate::metadata(&dir, "key").unwrap();
        assert_eq!(entry, None);

        let data_exists = crate::exists(&dir, &sri);
        assert!(!data_exists);
    }
}
