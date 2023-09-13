use std::fs::{create_dir_all, read_to_string, File};
use std::path::PathBuf;

use anyhow::{bail, Result};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::project_info::{
    is_valid_python_version, Day, DependabotSchedule, LicenseType, ProjectManager,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub creator: Option<String>,
    pub creator_email: Option<String>,
    pub license: Option<LicenseType>,
    pub python_version: Option<String>,
    pub min_python_version: Option<String>,
    pub project_manager: Option<ProjectManager>,
    pub is_application: Option<bool>,
    pub github_actions_python_test_versions: Option<Vec<String>>,
    pub max_line_length: Option<u8>,
    pub use_dependabot: Option<bool>,
    pub dependabot_schedule: Option<DependabotSchedule>,
    pub dependabot_day: Option<Day>,
    pub use_continuous_deployment: Option<bool>,
    pub use_release_drafter: Option<bool>,
    pub use_multi_os_ci: Option<bool>,
    pub download_latest_packages: Option<bool>,
}

impl Config {
    pub fn new() -> Self {
        Self {
            creator: None,
            creator_email: None,
            license: None,
            python_version: None,
            min_python_version: None,
            project_manager: None,
            is_application: None,
            github_actions_python_test_versions: None,
            max_line_length: None,
            use_dependabot: None,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: None,
            use_release_drafter: None,
            use_multi_os_ci: None,
            download_latest_packages: None,
        }
    }

    pub fn load_config() -> Result<Self> {
        if let Some(config_file) = config_file_path() {
            if config_file.exists() {
                if let Ok(config_str) = read_to_string(config_file) {
                    if let Ok(config) = serde_json::from_str::<Self>(&config_str) {
                        return Ok(config);
                    }
                }
            }
        };

        Ok(Self::new())
    }

    pub fn reset() -> Result<()> {
        let config = Config::new();
        config.save()?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        match config_dir() {
            Some(c) => {
                if !c.exists() {
                    create_dir_all(c)?;
                }

                match config_file_path() {
                    Some(c) => {
                        let config_file = File::create(c)?;
                        serde_json::to_writer(config_file, &self)?;
                    }
                    None => {
                        bail!("Unable to save config file.");
                    }
                }
            }
            None => {
                bail!("Unable to save config file.");
            }
        }

        Ok(())
    }

    pub fn save_creator(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.creator = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_creator_email(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.creator_email = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_license(value: LicenseType) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.license = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_python_version(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.python_version = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_min_python_version(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.min_python_version = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_project_manager(value: ProjectManager) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.project_manager = Some(value);
            if config.save().is_err() {
                raise_error()?;
            }
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_is_application(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.is_application = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_github_actions_python_test_versions(value: String) -> Result<()> {
        let versions = value
            .replace(' ', "")
            .split(',')
            .map(|s| s.to_string())
            .collect::<Vec<String>>();

        for version in &versions {
            if !is_valid_python_version(&version.replace('"', "")) {
                bail!(format!("{} is not a valid Python Version", version));
            }
        }

        if let Ok(mut config) = Config::load_config() {
            config.github_actions_python_test_versions = Some(versions);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_max_line_length(value: u8) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.max_line_length = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_dependabot(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_dependabot = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_dependabot_schedule(value: DependabotSchedule) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.dependabot_schedule = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_dependabot_day(value: Day) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.dependabot_day = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_continuous_deployment(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_continuous_deployment = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_release_drafter(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_release_drafter = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_multi_os_ci(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_multi_os_ci = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_download_latest_packages(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.download_latest_packages = Some(value);
            if config.save().is_err() {
                raise_error()?;
            };
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn show() {
        let config = match Config::load_config() {
            Ok(c) => c,
            Err(_) => Config::new(),
        };

        let creator_label = "Creator";
        if let Some(creator) = config.creator {
            println!("{}: {creator}", creator_label.blue());
        } else {
            println!("{}: null", creator_label.blue());
        }

        let creator_email_label = "Creator Email";
        if let Some(creator_email) = config.creator_email {
            println!("{}: {creator_email}", creator_email_label.blue());
        } else {
            println!("{}: null", creator_email_label.blue());
        }

        let license_label = "License";
        if let Some(license) = config.license {
            println!("{}: {:?}", license_label.blue(), license);
        } else {
            println!("{}: null", license_label.blue());
        }

        let python_version_label = "Python Version";
        if let Some(python_version) = config.python_version {
            println!("{}: {python_version}", python_version_label.blue());
        } else {
            println!("{}: null", python_version_label.blue());
        }

        let min_python_version_label = "Min Python Version";
        if let Some(min_python_version) = config.min_python_version {
            println!("{}: {min_python_version}", min_python_version_label.blue());
        } else {
            println!("{}: null", min_python_version_label.blue());
        }

        let is_application_label = "Application or Library";
        if let Some(is_application) = config.is_application {
            if is_application {
                println!("{}: application", is_application_label.blue());
            } else {
                println!("{}: lib", is_application_label.blue());
            }
        } else {
            println!("{}: null", is_application_label.blue());
        }

        let gha_python_label = "Python Versions for Github Actions Testing";
        if let Some(gha_python) = config.github_actions_python_test_versions {
            let gha_python_str = gha_python.join(", ");
            println!("{}: {gha_python_str}", gha_python_label.blue());
        } else {
            println!("{}: null", gha_python_label.blue());
        }

        let project_manager_label = "Project Manager";
        if let Some(project_manager) = config.project_manager {
            println!("{}: {:?}", project_manager_label.blue(), project_manager);
        } else {
            println!("{}: null", project_manager_label.blue());
        }

        let max_line_length_label = "Max Line Length";
        if let Some(max_line_length) = config.max_line_length {
            println!("{}: {max_line_length}", max_line_length_label.blue());
        } else {
            println!("{}: null", max_line_length_label.blue());
        }

        let use_dependabot_label = "Use Dependabot";
        if let Some(use_dependabot) = config.use_dependabot {
            println!("{}: {use_dependabot}", use_dependabot_label.blue());
        } else {
            println!("{}: null", use_dependabot_label.blue());
        }

        let dependabot_schedule_label = "Dependabot Schedule";
        if let Some(dependabot_schedule) = config.dependabot_schedule {
            println!(
                "{}: {:?}",
                dependabot_schedule_label.blue(),
                dependabot_schedule
            );
        } else {
            println!("{}: null", dependabot_schedule_label.blue());
        }

        let dependabot_day_label = "Dependabot Day";
        if let Some(dependabot_day) = config.dependabot_day {
            println!("{}: {:?}", dependabot_day_label.blue(), dependabot_day);
        } else {
            println!("{}: null", dependabot_day_label.blue());
        }

        let use_continuous_deployment_label = "Use Continuous Deployment";
        if let Some(use_continuous_deployment) = config.use_continuous_deployment {
            println!(
                "{}: {use_continuous_deployment}",
                use_continuous_deployment_label.blue()
            );
        } else {
            println!("{}: null", use_continuous_deployment_label.blue());
        }

        let use_release_drafter_label = "Use Release Drafter";
        if let Some(use_release_drafter) = config.use_release_drafter {
            println!(
                "{}: {use_release_drafter}",
                use_release_drafter_label.blue()
            );
        } else {
            println!("{}: null", use_release_drafter_label.blue());
        }

        let use_multi_os_ci_label = "Use Multi OS CI";
        if let Some(use_multi_os_ci) = config.use_multi_os_ci {
            println!("{}: {use_multi_os_ci}", use_multi_os_ci_label.blue());
        } else {
            println!("{}: null", use_multi_os_ci_label.blue());
        }

        let download_latest_packages_label = "Download Latest Packages";
        if let Some(download_latest_packages) = config.download_latest_packages {
            println!(
                "{}: {download_latest_packages}",
                download_latest_packages_label.blue()
            );
        } else {
            println!("{}: null", download_latest_packages_label.blue());
        }
    }
}

fn config_dir() -> Option<PathBuf> {
    let config_dir: Option<PathBuf> = dirs::config_dir();

    if let Some(mut c) = config_dir {
        c.push("python-project-generator");
        return Some(c);
    }

    None
}

fn config_file_path() -> Option<PathBuf> {
    if let Some(mut c) = config_dir() {
        c.push("config.json");
        return Some(c);
    };

    None
}

fn raise_error() -> Result<()> {
    bail!("Error saving config")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let config_dir = config_dir();
        assert_ne!(config_dir, None);

        if let Some(c) = config_dir {
            let last = c.file_name();
            assert_ne!(last, None);
            if let Some(l) = last {
                assert_eq!(l, "python-project-generator");
            }
        }
    }

    #[test]
    fn test_config_file_path() {
        let config_file_path = config_file_path();
        assert_ne!(config_file_path, None);

        if let Some(mut c) = config_file_path {
            let last = c.file_name();
            assert_ne!(last, None);
            if let Some(l) = last {
                assert_eq!(l, "config.json");
            }

            c.pop();
            let dir = c.file_name();
            assert_ne!(dir, None);
            if let Some(d) = dir {
                assert_eq!(d, "python-project-generator");
            }
        }
    }
}
