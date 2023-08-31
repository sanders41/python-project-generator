use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about = "Generates a Python project")]
pub struct Args {
    #[clap(
        short,
        long,
        help = "If set the default package versions will be used instead of the latest"
    )]
    pub skip_download_latest_packages: bool,
}
