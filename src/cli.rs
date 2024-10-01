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

    /// Remove the saved config for creator
    ResetCreator,

    /// Save a default creator email address
    CreatorEmail { value: String },

    /// Remove the saved config for creator email
    ResetCreatorEmail,

    /// Save a default license
    License { value: LicenseType },

    /// Remove the saved license
    ResetLicense,

    /// Save a default Python version
    PythonVersion { value: String },

    /// Remove the saved Python version
    ResetPythonVersion,

    /// Save a default minimum Python version
    MinPythonVersion { value: String },

    /// Remove the saved minimum Python version
    ResetMinPythonVersion,

    /// Save a default project manager
    ProjectManager { value: ProjectManager },

    /// Remove the saved project manager
    ResetProjectManager,

    /// Save a default value for is async project
    IsAsyncProject { value: BooleanChoice },

    /// Remove the saved async project value
    ResetIsAsyncProject,

    /// Save a default value for Is Application
    ApplicationOrLibrary { value: ApplicationOrLib },

    /// Remove the saved application or libary value
    ResetApplicationOrLibrary,

    /// Save default Python versions for GitHub Action testing
    GithubActionPythonTestVersions { value: String },

    /// Remove the saved github actions test versions
    ResetGithubActionPythonTestVersions,

    /// Save a default maximum line length
    MaxLineLength { value: u8 },

    /// Remove the saved max line length
    ResetMaxLineLength,

    /// Save a default value for Use Dependabot
    UseDependabot { value: BooleanChoice },

    /// Remove the saved use dependabot value
    ResetUseDependabot,

    /// Save a default value for Dependabot Schedule
    DependabotSchedule { value: DependabotSchedule },

    /// Remove the saved dependabot schedule
    ResetDependabotSchedule,

    /// Save a default value for Dependabot Day
    DependabotDay { value: Day },

    /// Remove the saved dependabot day
    ResetDependabotDay,

    /// Save a default value for Use Continuous Deployment
    UseContinuousDeployment { value: BooleanChoice },

    /// Remove the saved use continuous deployment value
    ResetUseContinuousDeployment,

    /// Save a default value for Use Release Drafter
    UseReleaseDrafter { value: BooleanChoice },

    /// Remove the saved use release drafter value
    ResetUseReleaseDrafter,

    /// Save a default value for Use Multi OS CI
    UseMultiOsCi { value: BooleanChoice },

    /// Remove the esaved use multi os ci value
    ResetUseMultiOsCi,

    /// Setup docs
    IncludeDocs { value: BooleanChoice },

    /// Remove the saved include docs value
    ResetIncludeDocs,

    /// Save a default value for Download Latest Packages
    DownloadLatestPackages { value: BooleanChoice },

    /// Remove the save download latest packages value
    ResetDownloadLatestPackages,

    /// Rerset the config to the default values
    Reset,

    /// View the current config values
    Show,
}
