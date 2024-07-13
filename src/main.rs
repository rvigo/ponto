mod config;
mod deploy;
mod file_type;
mod filesystem;
mod handlebars;
mod hook;
mod logger;
mod options;
mod symlink;
mod template;

use anyhow::Result;
use clap::Parser;
use options::Options;

fn main() -> Result<()> {
    let opts = Options::parse();

    logger::init(opts.verbosity, opts.quiet)?;

    let config = config::load_config(&opts.config)?;

    deploy::deploy(config, opts)?;

    Ok(())
}
