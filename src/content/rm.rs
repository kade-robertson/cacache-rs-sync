use std::fs;
use std::path::Path;

use ssri::Integrity;

use crate::content::path;
use crate::errors::{Internal, Result};

pub fn rm(cache: &Path, sri: &Integrity) -> Result<()> {
    fs::remove_file(path::content_path(cache, sri)).to_internal()?;
    Ok(())
}
