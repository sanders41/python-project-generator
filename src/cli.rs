use clap::{Parser, Subcommand, ValueEnum};

use crate::project_info::{Day, DependabotSchedule, LicenseType, ProjectManager};

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
        #[clap(
            short,
            long,
            help = "Use saved configuration and default values instead of prompting where possible"
        )]
        default: bool,
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
    Creator { value: String },

    /// Save a default creator email address
    CreatorEmail { value: String },

    /// Save a default license
    License { value: LicenseType },

    /// Save a default Python version
    PythonVersion { value: String },

    /// Save a default minimum Python version
    MinPythonVersion { value: String },

    /// Save a default project manager
    ProjectManager { value: ProjectManager },

    /// Save a default value for is async project
    IsAsyncProject { value: BooleanChoice },

    /// Save a default value for Is Application
    ApplicationOrLibrary { value: ApplicationOrLib },

    /// Save default Python versions for GitHub Action testing
    GithubActionPythonTestVersions { value: String },

    /// Save a default maximum line length
    MaxLineLength { value: u8 },

    /// Save a default value for Use Dependabot
    UseDependabot { value: BooleanChoice },

    /// Save a default value for Dependabot Schedule
    DependabotSchedule { value: DependabotSchedule },

    /// Save a default value for Dependabot Day
    DependabotDay { value: Day },

    /// Save a default value for Use Continuous Deployment
    UseContinuousDeployment { value: BooleanChoice },

    /// Save a default value for Use Release Drafter
    UseReleaseDrafter { value: BooleanChoice },

    /// Save a default value for Use Multi OS CI
    UseMultiOsCi { value: BooleanChoice },

    /// Setup docs
    IncludeDocs { value: BooleanChoice },

    /// Save a default value for Download Latest Packages
    DownloadLatestPackages { value: BooleanChoice },

    /// Rerset the config to the default values
    Reset,

    /// View the current config values
    Show,
}
