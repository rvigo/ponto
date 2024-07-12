use crate::{config::Variables, file_type::FileType};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::trace;
use std::{
    fmt::Display,
    fs::{self},
    io::Write,
    path::Path,
};

pub struct Template;

impl Template {
    pub fn render(
        from: &Path,
        to: &Path,
        handlebars: &Handlebars<'_>,
        variables: &Variables,
        force: bool,
    ) -> Result<()> {
        let template_type = TemplateState::from(FileType::try_from(from)?, FileType::try_from(to)?);
        trace!("{template_type}");

        let should_continue = match template_type {
            TemplateState::TargetNotRegularFile | TemplateState::BothMissing => false,
            TemplateState::OnlySourceExists | TemplateState::Changed => true,
            TemplateState::Identical if force => {
                trace!("forcing template rendering");
                true
            }
            TemplateState::Identical => false,
        };

        if should_continue {
            if force && to.exists() {
                trace!("removing existing file");
                std::fs::remove_file(to).context("remove file")?;
            }
            let content = fs::read_to_string(from).context("read to string")?;
            let rendered = handlebars
                .render_template(&content, variables)
                .context("render template")?;
            std::fs::create_dir_all(to.parent().unwrap()).context("create dir all")?;
            let mut file = fs::File::create(to).context("create file")?;
            file.write_all(rendered.as_bytes()).context("write all")?;
        }

        Ok(())
    }
}

pub enum TemplateState {
    Identical,
    OnlySourceExists,
    Changed,
    TargetNotRegularFile,
    BothMissing,
}

impl TemplateState {
    pub fn from(source_type: FileType, templated: FileType) -> TemplateState {
        match (source_type, templated) {
            (FileType::File(t), FileType::File(c)) => {
                if t == c {
                    TemplateState::Identical
                } else {
                    TemplateState::Changed
                }
            }
            (FileType::File(_), FileType::Missing) => TemplateState::OnlySourceExists,
            (FileType::Missing, FileType::Missing) => TemplateState::BothMissing,
            _ => TemplateState::TargetNotRegularFile,
        }
    }
}

impl Display for TemplateState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            TemplateState::Identical => "source and templated file contents are equal",
            TemplateState::OnlySourceExists => "templated file doesn't exist",
            TemplateState::Changed => "source contents were changed",
            TemplateState::TargetNotRegularFile => "target already exists and isn't a regular file",
            TemplateState::BothMissing => "templated file and source are missing",
        }
        .fmt(f)
    }
}
