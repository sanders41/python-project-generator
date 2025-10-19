mod cli;
mod config;
mod file_manager;
mod github_actions;
mod licenses;
mod package_version;
mod project_generator;
mod project_info;
mod python_files;
mod rust_files;
mod utils;

#[cfg(feature = "fastapi")]
mod fastapi;

use std::{fs::remove_dir_all, process::exit, time::Duration};

use anyhow::{Error, Result};
use clap::Parser;
use cli::ApplicationOrLib;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    cli::{Args, BooleanChoice, Command, Param},
    config::Config,
    project_generator::generate_project,
    project_info::{get_project_info, ProjectInfo},
};

#[cfg(feature = "fastapi")]
use crate::fastapi::{
    fastapi_files::generate_fastapi, fastapi_installer::install_fastapi_dependencies,
};

fn create(project_info: &ProjectInfo) -> Result<()> {
    generate_project(project_info)?;
    std::process::Command::new("git")
        .args(["init", &project_info.project_slug])
        .output()
        .expect("Failed to initialize git");

    #[cfg(feature = "fastapi")]
    if project_info.is_fastapi_project {
        install_fastapi_dependencies(project_info)?;
        generate_fastapi(project_info)?;
    }

    Ok(())
}

fn print_error(err: Error) {
    eprintln!("\n{}", err.to_string().red());
}

fn delete_slug(project_info: &ProjectInfo) -> Result<()> {
    let dir = &project_info.base_dir();

    if dir.exists() {
        remove_dir_all(dir)?;
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Create {
            skip_download_latest_packages,
            default,
        } => {
            let mut project_info = match get_project_info(default) {
                Ok(pi) => pi,
                Err(e) => {
                    print_error(e);
                    exit(1);
                }
            };
            project_info.download_latest_packages = !skip_download_latest_packages;

            let create_result: Result<()>;
            if let Ok(progress_style) = ProgressStyle::with_template("{spinner:.green} {msg}") {
                let pb = ProgressBar::new_spinner();
                pb.enable_steady_tick(Duration::from_millis(80));
                pb.set_style(
                    progress_style.tick_strings(&["⣷", "⣯", "⣟", "⡿", "⢿", "⣻", "⣽", "⣾"]),
                );
                pb.set_message("Generating Project...");
                create_result = create(&project_info);
                pb.finish_and_clear();
            } else {
                create_result = create(&project_info);
            }

            match create_result {
                Ok(_) => {
                    let success_message = format!(
                        "\nProject created in the {} directory",
                        project_info.project_slug
                    );
                    println!("{}", success_message.green());
                }
                Err(e) => {
                    print_error(e);
                    if let Err(e) = delete_slug(&project_info) {
                        print_error(e);
                    };
                    exit(1);
                }
            };
        }
        Command::Config(config) => match config.param {
            Param::Creator { value } => {
                if let Err(e) = Config::default().save_creator(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetCreator => {
                if let Err(e) = Config::default().reset_creator() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::CreatorEmail { value } => {
                if let Err(e) = Config::default().save_creator_email(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetCreatorEmail => {
                if let Err(e) = Config::default().reset_creator_email() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::License { value } => {
                if let Err(e) = Config::default().save_license(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetLicense => {
                if let Err(e) = Config::default().reset_license() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::PythonVersion { value } => {
                if let Err(e) = Config::default().save_python_version(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetPythonVersion => {
                if let Err(e) = Config::default().reset_python_version() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::MinPythonVersion { value } => {
                if let Err(e) = Config::default().save_min_python_version(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetMinPythonVersion => {
                if let Err(e) = Config::default().reset_min_python_version() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ProjectManager { value } => {
                if let Err(e) = Config::default().save_project_manager(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetProjectManager => {
                if let Err(e) = Config::default().reset_project_manager() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::Pyo3PythonManager { value } => {
                if let Err(e) = Config::default().save_pyo3_python_manager(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetPyo3PythonManager => {
                if let Err(e) = Config::default().reset_pyo3_python_manager() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ApplicationOrLibrary { value } => match value {
                ApplicationOrLib::Application => {
                    if let Err(e) = Config::default().save_is_application(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                ApplicationOrLib::Lib => {
                    if let Err(e) = Config::default().save_is_application(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetApplicationOrLibrary => {
                if let Err(e) = Config::default().reset_is_application() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::IsAsyncProject { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_is_async_project(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_is_async_project(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetIsAsyncProject => {
                if let Err(e) = Config::default().reset_is_async_project() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::GithubActionPythonTestVersions { value } => {
                if let Err(e) = Config::default().save_github_actions_python_test_versions(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetGithubActionPythonTestVersions => {
                if let Err(e) = Config::default().reset_github_actions_python_test_versions() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::MaxLineLength { value } => {
                if let Err(e) = Config::default().save_max_line_length(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetMaxLineLength => {
                if let Err(e) = Config::default().reset_max_line_length() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::UseDependabot { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_use_dependabot(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_use_dependabot(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetUseDependabot => {
                if let Err(e) = Config::default().reset_use_dependabot() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::DependabotSchedule { value } => {
                if let Err(e) = Config::default().save_dependabot_schedule(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetDependabotSchedule => {
                if let Err(e) = Config::default().reset_dependabot_schedule() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::DependabotDay { value } => {
                if let Err(e) = Config::default().save_dependabot_day(value) {
                    print_error(e);
                    exit(1);
                }
            }
            Param::ResetDependabotDay => {
                if let Err(e) = Config::default().reset_dependabot_day() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::UseContinuousDeployment { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_use_continuous_deployment(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_use_continuous_deployment(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetUseContinuousDeployment => {
                if let Err(e) = Config::default().reset_use_continuous_deployment() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::UseReleaseDrafter { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_use_release_drafter(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_use_release_drafter(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetUseReleaseDrafter => {
                if let Err(e) = Config::default().reset_use_release_drafter() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::UseMultiOsCi { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_use_multi_os_ci(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_use_multi_os_ci(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetUseMultiOsCi => {
                if let Err(e) = Config::default().reset_use_multi_os_ci() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::IncludeDocs { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_include_docs(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_include_docs(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetIncludeDocs => {
                if let Err(e) = Config::default().reset_include_docs() {
                    print_error(e);
                    exit(1);
                }
            }
            Param::DownloadLatestPackages { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_download_latest_packages(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_download_latest_packages(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },
            Param::ResetDownloadLatestPackages => {
                if let Err(e) = Config::default().reset_download_latest_packages() {
                    print_error(e);
                    exit(1);
                }
            }

            #[cfg(feature = "fastapi")]
            Param::IsFastapiProject { value } => match value {
                BooleanChoice::True => {
                    if let Err(e) = Config::default().save_is_fastapi_project(true) {
                        print_error(e);
                        exit(1);
                    }
                }
                BooleanChoice::False => {
                    if let Err(e) = Config::default().save_is_fastapi_project(false) {
                        print_error(e);
                        exit(1);
                    }
                }
            },

            #[cfg(feature = "fastapi")]
            Param::ResetIsFastapiProject => {
                if let Err(e) = Config::default().reset_is_fastapi_project() {
                    print_error(e);
                    exit(1);
                }
            }

            /* #[cfg(feature = "fastapi")]
            Param::Database { value } => {
                if let Err(e) = Config::default().save_database(value) {
                    print_error(e);
                    exit(1);
                }
            }

            #[cfg(feature = "fastapi")]
            Param::ResetDatabase => {
                if let Err(e) = Config::default().reset_database() {
                    print_error(e);
                    exit(1);
                }
            }

            #[cfg(feature = "fastapi")]
            Param::DatabaseManager { value } => {
                if let Err(e) = Config::default().save_database_manager(value) {
                    print_error(e);
                    exit(1);
                }
            }

            #[cfg(feature = "fastapi")]
            Param::ResetDatabaseManager => {
                if let Err(e) = Config::default().reset_database_manager() {
                    print_error(e);
                    exit(1);
                }
            } */
            Param::Reset => {
                if Config::reset().is_err() {
                    let message = "Error resetting config.";
                    eprintln!("{}", message.red());
                    exit(1);
                }
            }
            Param::Show => Config::default().show(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::project_info::{LicenseType, ProjectManager};
    use super::*;
    use std::fs::create_dir_all;
    use tmp_path::tmp_path;

    #[test]
    #[tmp_path]
    fn test_delete_slug() {
        let project_slug = "test-project";
        let slug_dir = tmp_path.join(project_slug);
        let project_info = ProjectInfo {
            project_name: "My project".to_string(),
            project_slug: project_slug.to_string(),
            source_dir: "my_project".to_string(),
            project_description: "This is a test".to_string(),
            creator: "Arthur Dent".to_string(),
            creator_email: "authur@heartofgold.com".to_string(),
            license: LicenseType::Mit,
            copyright_year: Some("2023".to_string()),
            version: "0.1.0".to_string(),
            python_version: "3.12".to_string(),
            min_python_version: "3.10".to_string(),
            project_manager: ProjectManager::Poetry,
            pyo3_python_manager: None,
            is_application: true,
            is_async_project: false,
            github_actions_python_test_versions: vec![
                "3.10".to_string(),
                "3.11".to_string(),
                "3.12".to_string(),
                "3.13".to_string(),
                "3.14".to_string(),
            ],
            max_line_length: 100,
            use_dependabot: true,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: true,
            use_release_drafter: true,
            use_multi_os_ci: true,
            include_docs: false,
            docs_info: None,
            download_latest_packages: false,
            project_root_dir: Some(tmp_path),

            #[cfg(feature = "fastapi")]
            is_fastapi_project: false,

            #[cfg(feature = "fastapi")]
            database_manager: None,
        };
        create_dir_all(&slug_dir).unwrap();
        assert!(slug_dir.exists());
        delete_slug(&project_info).unwrap();
        assert!(!slug_dir.exists());
    }
}
