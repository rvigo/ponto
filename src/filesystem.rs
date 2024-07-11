use anyhow::{Context, Result};
use log::warn;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

pub trait Filesystem {
    fn is_template(&self) -> Result<bool>;

    fn real_path(&self) -> Result<PathBuf>;
}

impl Filesystem for PathBuf {
    fn is_template(&self) -> Result<bool> {
        if fs::metadata(self)?.is_dir() {
            return Ok(false);
        }

        let mut file = File::open(self).context("open file")?;
        let mut buf = String::new();

        if file.read_to_string(&mut buf).is_err() {
            warn!("file {:?} is not valid UTF-8 - detecting as symlink. Explicitly specify it to silence this message.", self);
            Ok(false)
        } else {
            Ok(buf.contains("{{"))
        }
    }

    fn real_path(&self) -> Result<PathBuf> {
        let path = self.canonicalize()?;
        Ok(path)
    }
}
