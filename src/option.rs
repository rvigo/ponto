use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser, Default, Clone)]
#[clap(author, version, about, long_about = None)]
pub struct Options {
    #[clap(short, long, value_parser, default_value = "ponto/config.yaml")]
    pub config: PathBuf,

    #[clap(long, value_parser, default_value = "ponto/pre.sh")]
    pub pre: PathBuf,

    #[clap(long, value_parser, default_value = "ponto/post.sh")]
    pub post: PathBuf,

    #[clap(short, long, value_parser)]
    pub force: bool,

    #[clap(short, long, value_parser)]
    pub quiet: bool,

    #[clap(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbosity: u8,
}
