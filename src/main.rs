use anyhow::Result;
use clap::Parser;
use option::Options;

mod config;
mod deploy;
mod file_type;
mod filesystem;
mod handlebars;
mod hook;
mod logger;
mod option;
mod symlink;
mod template;

fn main() -> Result<()> {
    let opts = Options::parse();

    logger::init(opts.verbosity, opts.quiet)?;

    let config = config::load_config(&opts.config)?;

    deploy::deploy(config, opts)?;

    Ok(())
}
