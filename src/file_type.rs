use anyhow::Context;
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

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
