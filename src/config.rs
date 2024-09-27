use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, File},
    path::PathBuf,
};

use anyhow::{bail, Result};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::project_info::{
    is_valid_python_version, Day, DependabotSchedule, LicenseType, ProjectManager,
};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub creator: Option<String>,
    pub creator_email: Option<String>,
    pub license: Option<LicenseType>,
    pub python_version: Option<String>,
    pub min_python_version: Option<String>,
    pub project_manager: Option<ProjectManager>,
    pub is_async_project: Option<bool>,
    pub is_application: Option<bool>,
    pub github_actions_python_test_versions: Option<Vec<String>>,
    pub max_line_length: Option<u8>,
    pub use_dependabot: Option<bool>,
    pub dependabot_schedule: Option<DependabotSchedule>,
    pub dependabot_day: Option<Day>,
    pub use_continuous_deployment: Option<bool>,
    pub use_release_drafter: Option<bool>,
    pub use_multi_os_ci: Option<bool>,
    pub include_docs: Option<bool>,
    pub download_latest_packages: Option<bool>,
}

impl Config {
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

        Ok(Self::default())
    }

    pub fn reset() -> Result<()> {
        let config = Config::default();
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
                        bail!("Error saving config file");
                    }
                }
            }
            None => {
                bail!("Error saving config file");
            }
        }

        Ok(())
    }

    pub fn save_creator(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.creator = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_creator_email(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.creator_email = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_license(value: LicenseType) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.license = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_python_version(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.python_version = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_min_python_version(value: String) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.min_python_version = Some(value);
            config.save()?;
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

    pub fn save_is_async_project(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.is_async_project = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_is_application(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.is_application = Some(value);
            config.save()?;
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
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_max_line_length(value: u8) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.max_line_length = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_dependabot(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_dependabot = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_dependabot_schedule(value: DependabotSchedule) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.dependabot_schedule = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_dependabot_day(value: Day) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.dependabot_day = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_continuous_deployment(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_continuous_deployment = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_release_drafter(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_release_drafter = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_use_multi_os_ci(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.use_multi_os_ci = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_include_docs(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.include_docs = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn save_download_latest_packages(value: bool) -> Result<()> {
        if let Ok(mut config) = Config::load_config() {
            config.download_latest_packages = Some(value);
            config.save()?;
        } else {
            raise_error()?;
        }

        Ok(())
    }

    pub fn show() {
        let config = Config::load_config().unwrap_or_default();
        print_config_value("Creator", &config.creator);
        print_config_value("Creator Email", &config.creator_email);
        print_config_value("License", &config.license);
        print_config_value("Python Version", &config.python_version);
        print_config_value("Min Python Version", &config.min_python_version);

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

        print_config_value("Project Manager", &config.project_manager);
        print_config_value("Async Project", &config.is_async_project);
        print_config_value("Max Line Length", &config.max_line_length);
        print_config_value("Use Dependabot", &config.use_dependabot);
        print_config_value("Dependabot Schedule", &config.dependabot_schedule);
        print_config_value("Dependabot Day", &config.dependabot_day);
        print_config_value(
            "Use Continuous Deployment",
            &config.use_continuous_deployment,
        );
        print_config_value("Use Release Drafter", &config.use_release_drafter);
        print_config_value("Use Multi OS CI", &config.use_multi_os_ci);
        print_config_value("Include Docs", &config.include_docs);
        print_config_value("Download Latest Packages", &config.download_latest_packages);
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

fn print_config_value<T: Display>(label: &str, value: &Option<T>) {
    if let Some(v) = value {
        println!("{}: {}", label.blue(), v);
    } else {
        println!("{}: null", label.blue());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_dir() {
        let config_dir = config_dir();
        assert_ne!(config_dir, None);
        let config = config_dir.unwrap();

        let last = config.file_name();
        assert_ne!(last, None);
        assert_eq!(last.unwrap(), "python-project-generator");
    }

    #[test]
    fn test_config_file_path() {
        let config_file_path = config_file_path();
        assert_ne!(config_file_path, None);
        let mut config = config_file_path.unwrap();

        let last = config.file_name();
        assert_ne!(last, None);
        assert_eq!(last.unwrap(), "config.json");

        config.pop();
        let dir = config.file_name();
        assert_ne!(dir, None);
        assert_eq!(dir.unwrap(), "python-project-generator");
    }
}
