use super::handlebars::create_new_handlebars;
use crate::{
    config::Configuration,
    filesystem::Filesystem,
    hook::{self, Hook},
    option::Options,
    symlink::Symlink,
    template::Template,
};
use anyhow::{Context, Result};
use log::info;

pub fn deploy(config: Configuration, opts: Options) -> Result<()> {
    let handlebars = create_new_handlebars().context("initialize handlebars")?;

    // pre hook
    hook::Pre::run(&opts.pre, &handlebars, &config.variables)?;

    if opts.force {
        info!("forcing symlink creation (due to --force option)");
    }

    // deploy files
    for (_, package) in config.ordered_by_dependencies() {
        for (from, to) in package.files {
            if from.is_template()? {
                info!("rendering template file from {from:?} to {to:?}");
                Template::render(&from, &to, &handlebars, &package.variables)
                    .context("rendering template")?;
            } else {
                info!("creating symlink from {from:?} to {to:?}");
                Symlink::create(&from, &to, opts.force).context("creating symlink")?;
            }
        }
    }

    // post hook
    hook::Post::run(&opts.post, &handlebars, &config.variables)?;
    // delete templated files
    hook::remove_templated_files().context("deleting templated files")?;
    Ok(())
}
