use clap::{Parser, Subcommand, ValueEnum};

use crate::project_info::LicenseType;

#[derive(Clone, Debug, ValueEnum)]
pub enum ApplicationOrLib {
    Application,
    Lib,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum BooleanChoice {
    True,
    False,
}

#[derive(Debug, Parser)]
#[clap(author, version, about = "Generates a Python project")]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new project
    Create {
        #[clap(
            short,
            long,
            help = "If set the default package versions will be used instead of the latest"
        )]
        skip_download_latest_packages: bool,
    },

    /// Save default config values
    Config(Config),
}

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(subcommand)]
    pub param: Param,
}

#[derive(Debug, Subcommand)]
pub enum Param {
    /// Save a default creator
    Creator {
        #[clap(help = "Default name to use for Creator")]
        value: String,
    },

    /// Save a default creator email address
    CreatorEmail {
        #[clap(help = "Default name to use for Creator Email")]
        value: String,
    },

    /// Save a default license
    License {
        #[clap(help = "Default license")]
        value: LicenseType,
    },

    /// Save a default Python version
    PythonVersion {
        #[clap(help = "Default Python version")]
        value: String,
    },

    /// Save a default minimum Python version
    MinPythonVersion {
        #[clap(help = "Default minimum Python version")]
        value: String,
    },

    /// Use pyo3
    UsePyo3 {
        #[clap(help = "Use pyo3")]
        value: BooleanChoice,
    },

    /// Save a default value for Is Application
    ApplicationOrLibrary {
        #[clap(help = "Default Is Application value")]
        value: ApplicationOrLib,
    },

    /// Save default Python versions for GitHub Action testing
    GithubActionPythonTestVersions {
        #[clap(help = "Default Python versions for GitHub Action testing")]
        value: String,
    },

    /// Save a default maximum line length
    MaxLineLength {
        #[clap(help = "Default maximum line length")]
        value: u8,
    },

    /// Save a default value for Use Dependabot
    UseDependabot {
        #[clap(help = "Default value for Use Dependabot")]
        value: BooleanChoice,
    },

    /// Save a default value for Use Continuous Deployment
    UseContinuousDeployment {
        #[clap(help = "Default value for Use Continuous Deployment")]
        value: BooleanChoice,
    },

    /// Save a default value for Use Release Drafter
    UseReleaseDrafter {
        #[clap(help = "Default value for Use Release Drafter")]
        value: BooleanChoice,
    },

    /// Save a default value for Use Multi OS CI
    UseMultiOsCi {
        #[clap(help = "Default value for Use Multi OS CI")]
        value: BooleanChoice,
    },

    /// Save a default value for Download Latest Packages
    DownloadLatestPackages {
        #[clap(help = "Default value for Download Latest Packages")]
        value: BooleanChoice,
    },

    /// Rerset the config to the default values
    Reset,

    /// View the current config values
    Show,
}
