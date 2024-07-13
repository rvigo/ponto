use anyhow::Context;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub enum FileType {
    /// None if file is invalid UTF-8
    File(Option<String>),
    SymbolicLink(PathBuf),
    Directory,
    Missing,
}

impl TryFrom<&Path> for FileType {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> std::result::Result<Self, Self::Error> {
        if let Ok(target) = fs::read_link(value) {
            return Ok(FileType::SymbolicLink(target));
        }

        if value.is_dir() {
            return Ok(FileType::Directory);
        }

        match fs::read_to_string(value) {
            Ok(f) => Ok(FileType::File(Some(f))),
            Err(e) if e.kind() == ErrorKind::InvalidData => Ok(FileType::File(None)),
            Err(e) if e.kind() == ErrorKind::NotFound => Ok(FileType::Missing),
            Err(e) => Err(e).context("read contents of file that isn't symbolic or directory")?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};
    use std::fs::File;
    use std::io::Write;
    use tempdir::TempDir;

    #[test]
    fn should_return_the_file_type_as_symbolic_link() -> Result<()> {
        let dir = TempDir::new("file_type")?;

        let target_path = dir.path().join("target.txt");
        File::create(&target_path)?.write_all(b"Hello, world!")?;
        let link_path = dir.path().join("link.txt");
        std::os::unix::fs::symlink(&target_path, &link_path)?;

        let file_type = FileType::try_from(link_path.as_path())?;
        assert_eq!(file_type, FileType::SymbolicLink(target_path));

        Ok(())
    }

    #[test]
    fn should_return_the_file_type_as_directory() -> Result<()> {
        let dir = TempDir::new("file_type")?;

        let file_type = FileType::try_from(dir.path())?;
        assert_eq!(file_type, FileType::Directory);

        Ok(())
    }

    #[test]
    fn should_return_the_file_type_as_valid_file() -> Result<()> {
        let dir = TempDir::new("file_type")?;

        let file_path = dir.path().join("file.txt");
        File::create(&file_path)?.write_all(b"Hello, world!")?;
        let file_type = FileType::try_from(file_path.as_path())?;
        assert_eq!(file_type, FileType::File(Some("Hello, world!".to_string())));

        Ok(())
    }

    #[test]
    fn should_return_the_file_type_as_invalid_utf8_file() -> Result<()> {
        let dir = TempDir::new("file_type")?;

        let file_path = dir.path().join("file.txt");
        let mut file = File::create(&file_path)?;
        file.write_all(&[0xff, 0xff, 0xff])?;

        let file_type = FileType::try_from(file_path.as_path())?;
        assert_eq!(file_type, FileType::File(None));

        Ok(())
    }

    #[test]
    fn should_return_the_file_type_as_missing_file() -> Result<()> {
        let dir = TempDir::new("file_type")?;

        let missing_path = dir.path().join("missing.txt");

        let file_type = FileType::try_from(missing_path.as_path())?;
        assert_eq!(file_type, FileType::Missing);

        Ok(())
    }
}
