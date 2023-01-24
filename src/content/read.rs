use std::fs::{self, File};
use std::path::Path;

use ssri::{Algorithm, Integrity, IntegrityChecker};

use crate::content::path;
use crate::errors::{Internal, Result};

pub struct Reader {
    fd: File,
    checker: IntegrityChecker,
}

impl std::io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let amt = self.fd.read(buf)?;
        self.checker.input(&buf[..amt]);
        Ok(amt)
    }
}

impl Reader {
    pub fn check(self) -> Result<Algorithm> {
        Ok(self.checker.result()?)
    }
}

pub fn open(cache: &Path, sri: Integrity) -> Result<Reader> {
    let cpath = path::content_path(cache, &sri);
    Ok(Reader {
        fd: File::open(cpath).to_internal()?,
        checker: IntegrityChecker::new(sri),
    })
}

pub fn read(cache: &Path, sri: &Integrity) -> Result<Vec<u8>> {
    let cpath = path::content_path(cache, sri);
    let ret = fs::read(cpath).to_internal()?;
    sri.check(&ret)?;
    Ok(ret)
}

pub fn copy(cache: &Path, sri: &Integrity, to: &Path) -> Result<u64> {
    let cpath = path::content_path(cache, sri);
    let ret = fs::copy(&cpath, to).to_internal()?;
    let data = fs::read(cpath).to_internal()?;
    sri.check(data)?;
    Ok(ret)
}

pub fn has_content(cache: &Path, sri: &Integrity) -> Option<Integrity> {
    if path::content_path(cache, sri).exists() {
        Some(sri.clone())
    } else {
        None
    }
}
