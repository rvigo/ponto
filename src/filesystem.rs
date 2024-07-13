use anyhow::{Context, Result};
use log::warn;
use std::fs::{self, File};
use std::io::Read;
use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn should_copy_file() -> Result<()> {
        let dir = TempDir::new("filesystem")?;

        let from = dir.path().join("from.txt");
        File::create(&from)?.write_all(b"Hello, world!")?;
        let to = dir.path().join("to.txt");

        Filesystem::copy(&from, &to, false)?;

        let from_content = fs::read_to_string(&from)?;
        let to_content = fs::read_to_string(&to)?;

        assert_eq!(from_content, to_content);

        Ok(())
    }

    #[test]
    fn should_check_if_file_is_template() -> Result<()> {
        let dir = TempDir::new("filesystem")?;

        let file_path = dir.path().join("file.txt");
        File::create(&file_path)?.write_all(b"Hello, {{ name }}!")?;

        assert!(file_path.is_template()?);

        Ok(())
    }
}
