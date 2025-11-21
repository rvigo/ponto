use anyhow::{Context, Result};
use simple_logger::SimpleLogger;

pub fn init(verbosity: u8, quiet: bool) -> Result<()> {
    let level = match (verbosity, quiet) {
        (0, false) => log::LevelFilter::Info,
        (1, false) => log::LevelFilter::Debug,
        (2, false) => log::LevelFilter::Trace,
        (_, true) => log::LevelFilter::Error,
        _ => unreachable!("invalid verbosity level"),
    };

    SimpleLogger::new()
        .with_level(log::LevelFilter::Error)
        .with_module_level("ponto", level)
        .init()
        .context("cannot set logger")
}
