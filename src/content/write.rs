use std::fs::DirBuilder;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use memmap2::MmapMut;
use ssri::{Algorithm, Integrity, IntegrityOpts};
use tempfile::NamedTempFile;

use crate::content::path;
use crate::errors::{Internal, Result};

pub const MAX_MMAP_SIZE: usize = 1024 * 1024;

pub struct Writer {
    cache: PathBuf,
    builder: IntegrityOpts,
    mmap: Option<MmapMut>,
    tmpfile: NamedTempFile,
}

impl Writer {
    pub fn new(cache: &Path, algo: Algorithm, size: Option<usize>) -> Result<Writer> {
        let cache_path = cache.to_path_buf();
        let mut tmp_path = cache_path.clone();
        tmp_path.push("tmp");
        DirBuilder::new()
            .recursive(true)
            .create(&tmp_path)
            .to_internal()?;
        let mut tmpfile = NamedTempFile::new_in(tmp_path).to_internal()?;
        let mmap = if let Some(size) = size {
            if size <= MAX_MMAP_SIZE {
                tmpfile.as_file_mut().set_len(size as u64).to_internal()?;
                unsafe { MmapMut::map_mut(tmpfile.as_file()).ok() }
            } else {
                None
            }
        } else {
            None
        };
        Ok(Writer {
            cache: cache_path,
            builder: IntegrityOpts::new().algorithm(algo),
            tmpfile,
            mmap,
        })
    }

    pub fn close(self) -> Result<Integrity> {
        let sri = self.builder.result();
        let cpath = path::content_path(&self.cache, &sri);
        DirBuilder::new()
            .recursive(true)
            // Safe unwrap. cpath always has multiple segments
            .create(cpath.parent().unwrap())
            .to_internal()?;
        let res = self.tmpfile.persist(&cpath).to_internal();
        if res.is_err() {
            // We might run into conflicts sometimes when persisting files.
            // This is ok. We can deal. Let's just make sure the destination
            // file actually exists, and we can move on.
            std::fs::metadata(cpath).to_internal()?;
        }
        Ok(sri)
    }
}

impl Write for Writer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.builder.input(buf);
        if let Some(mmap) = &mut self.mmap {
            mmap.copy_from_slice(buf);
            Ok(buf.len())
        } else {
            self.tmpfile.write(buf)
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tmpfile.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile;
    #[test]
    fn basic_write() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_owned();
        let mut writer = Writer::new(&dir, Algorithm::Sha256, None).unwrap();
        writer.write_all(b"hello world").unwrap();
        let sri = writer.close().unwrap();
        assert_eq!(sri.to_string(), Integrity::from(b"hello world").to_string());
        assert_eq!(
            std::fs::read(path::content_path(&dir, &sri)).unwrap(),
            b"hello world"
        );
    }
}
