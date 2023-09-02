use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about = "Generates a Python project")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Create {
        #[clap(
            short,
            long,
            help = "If set the default package versions will be used instead of the latest"
        )]
        skip_download_latest_packages: bool,
    },

    Config {},
}
