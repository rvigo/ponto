use anyhow::{Context, Result};
use log::warn;
use std::{
    fs::{self, File},
    io::Read,
    path::PathBuf,
};

pub struct Filesystem;

impl Filesystem {
    pub fn copy(from: &PathBuf, to: &PathBuf, force: bool) -> Result<()> {
        if to.exists() && !force {
            warn!("file {:?} already exists, skipping", to);
            return Ok(());
        }

        fs::create_dir_all(to.parent().unwrap()).context("creating parent directory")?;
        fs::copy(from, to).context("copying file")?;
        Ok(())
    }
}

pub trait FilesystemExt {
    fn is_template(&self) -> Result<bool>;

    fn real_path(&self) -> Result<PathBuf>;
}

impl FilesystemExt for PathBuf {
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
