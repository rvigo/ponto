use super::handlebars::init;
use crate::{
    config::{Configuration, FileSpec, Variables},
    filesystem::{Filesystem, FilesystemExt},
    hook::{self, Hook},
    option::Options,
    symlink::Symlink,
    template::Template,
};
use anyhow::{Context, Result};
use handlebars::Handlebars;
use log::{debug, info};
use std::path::PathBuf;

pub fn deploy(config: Configuration, opts: Options) -> Result<()> {
    let handlebars = init().context("initialize handlebars")?;

    // pre hook
    hook::Pre::run(&opts.pre, &handlebars, &config.variables)?;

    // deploy files
    info!(
        "deploying files{}",
        if opts.force { " (forced)" } else { "" }
    );
    for (_, package) in config.ordered_by_dependencies() {
        for (from, to) in package.files {
            match to {
                FileSpec::Simple(to) => {
                    process_simple(&from, &to, &handlebars, &config.variables, opts.force)?
                }
                FileSpec::WithSpec(spec) => process_with_spec(
                    &from,
                    &spec.to,
                    spec.symlink,
                    &handlebars,
                    &package.variables,
                    opts.force,
                )?,
            }
        }
    }
    info!("files deployed");

    // post hook
    hook::Post::run(&opts.post, &handlebars, &config.variables)?;
    // delete templated files
    hook::remove_templated_scripts().context("deleting templated files")?;
    Ok(())
}

fn process_simple(
    from: &PathBuf,
    to: &PathBuf,
    handlebars: &Handlebars<'_>,
    variables: &Variables,
    force: bool,
) -> Result<()> {
    if from.is_template().context("check if template")? {
        debug!("rendering template file from {from:?} to {to:?}");
        Template::render(from, to, handlebars, variables, force).context("rendering template")?;
    } else {
        debug!("creating symlink from {from:?} to {to:?}");
        Symlink::create(from, to, force).context("creating symlink")?;
    }
    Ok(())
}

fn process_with_spec(
    from: &PathBuf,
    to: &PathBuf,
    is_symlink: bool,
    handlebars: &Handlebars<'_>,
    variables: &Variables,
    force: bool,
) -> Result<()> {
    if from.is_template()? {
        debug!("rendering template file from {from:?} to {to:?}");
        Template::render(from, to, handlebars, variables, force).context("rendering template")?;
    } else if !is_symlink {
        debug!("copying file from {from:?} to {to:?}");
        Filesystem::copy(from, to, force).context("copying file")?;
    } else {
        debug!("creating symlink from {from:?} to {to:?}");
        Symlink::create(from, to, force).context("creating symlink")?;
    }
    Ok(())
}
