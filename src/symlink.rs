use super::file_type::FileType;
use crate::filesystem::Filesystem;
use anyhow::{Context, Result};
use log::debug;
use std::{fmt::Display, path::Path};

pub struct Symlink;

impl Symlink {
    pub fn create(from: &Path, to: &Path, force: bool) -> Result<()> {
        let result = SymlinkState::from(from, FileType::try_from(from)?, FileType::try_from(to)?)
            .context("get symlink state")?;

        // TODO warn if source is missing
        let should_continue = match result {
            SymlinkState::Changed
            | SymlinkState::BothMissing
            | SymlinkState::OnlyTargetExists
            | SymlinkState::TargetNotSymlink => false,
            SymlinkState::OnlySourceExists => true,
            SymlinkState::Identical if force => {
                debug!("forcing symlink creation");
                true
            }
            SymlinkState::Identical => false,
        };
        debug!("{result}");

        if should_continue {
            std::fs::create_dir_all(to.parent().unwrap()).context("create dir all")?;
            if force && to.exists() {
                debug!("removing existing symlink");
                std::fs::remove_file(to).context("remove file")?;
            }
            std::os::unix::fs::symlink(
                from.to_path_buf()
                    .real_path()
                    .context("get real path of source file")?,
                to,
            )
            .context("create symlink")?;
        }
        Ok(())
    }
}

pub enum SymlinkState {
    Identical,
    OnlySourceExists,
    OnlyTargetExists,
    TargetNotSymlink,
    Changed,
    BothMissing,
}

impl SymlinkState {
    pub fn from(
        source_path: &Path,
        source_type: FileType,
        link_type: FileType,
    ) -> Result<SymlinkState> {
        Ok(match (source_type, link_type) {
            (FileType::Missing, FileType::SymbolicLink(_)) => SymlinkState::OnlyTargetExists,
            (_, FileType::SymbolicLink(t)) => {
                if t == source_path
                    .to_path_buf()
                    .real_path()
                    .context("get real path of source")?
                {
                    SymlinkState::Identical
                } else {
                    SymlinkState::Changed
                }
            }
            (FileType::Missing, FileType::Missing) => SymlinkState::BothMissing,
            (_, FileType::Missing) => SymlinkState::OnlySourceExists,
            _ => SymlinkState::TargetNotSymlink,
        })
    }
}

impl Display for SymlinkState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            SymlinkState::Identical => "target points at source",
            SymlinkState::OnlySourceExists => "target missing",
            SymlinkState::OnlyTargetExists => "source is missing",
            SymlinkState::TargetNotSymlink => "target already exists and isn't a symlink",
            SymlinkState::Changed => "target already exists and doesn't point at source",
            SymlinkState::BothMissing => "source and target are missing",
        }
        .fmt(f)
    }
}
