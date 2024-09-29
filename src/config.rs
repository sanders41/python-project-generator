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

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
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

    #[serde(skip)]
    config_dir: Option<PathBuf>,
    #[serde(skip)]
    config_file_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            creator: None,
            creator_email: None,
            license: None,
            python_version: None,
            min_python_version: None,
            project_manager: None,
            is_async_project: None,
            is_application: None,
            github_actions_python_test_versions: None,
            max_line_length: None,
            use_dependabot: None,
            dependabot_schedule: None,
            dependabot_day: None,
            use_continuous_deployment: None,
            use_release_drafter: None,
            use_multi_os_ci: None,
            include_docs: None,
            download_latest_packages: None,
            config_dir: config_dir(),
            config_file_path: config_file_path(),
        }
    }
}

impl Config {
    pub fn load_config(&self) -> Self {
        if let Some(config_file) = &self.config_file_path {
            if config_file.exists() {
                if let Ok(config_str) = read_to_string(config_file) {
                    if let Ok(config) = serde_json::from_str::<Self>(&config_str) {
                        return Self {
                            creator: config.creator,
                            creator_email: config.creator_email,
                            license: config.license,
                            python_version: config.python_version,
                            min_python_version: config.min_python_version,
                            project_manager: config.project_manager,
                            is_async_project: config.is_async_project,
                            is_application: config.is_application,
                            github_actions_python_test_versions: config
                                .github_actions_python_test_versions,
                            max_line_length: config.max_line_length,
                            use_dependabot: config.use_dependabot,
                            dependabot_schedule: config.dependabot_schedule,
                            dependabot_day: config.dependabot_day,
                            use_continuous_deployment: config.use_continuous_deployment,
                            use_release_drafter: config.use_release_drafter,
                            use_multi_os_ci: config.use_multi_os_ci,
                            include_docs: config.include_docs,
                            download_latest_packages: config.download_latest_packages,
                            config_dir: self.config_dir.clone(),
                            config_file_path: self.config_file_path.clone(),
                        };
                    }
                }
            }
        };

        Self::default()
    }

    pub fn reset() -> Result<()> {
        let config = Self::default();
        config.save()?;

        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        match &self.config_dir {
            Some(c) => {
                if !c.exists() {
                    create_dir_all(c)?;
                }

                match &self.config_file_path {
                    Some(c) => {
                        let config_file = File::create(c)?;
                        serde_json::to_writer_pretty(config_file, self)?;
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

    pub fn save_creator(&self, value: String) -> Result<()> {
        let mut config = self.load_config();
        config.creator = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_creator_email(&self, value: String) -> Result<()> {
        let mut config = self.load_config();
        config.creator_email = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_license(&self, value: LicenseType) -> Result<()> {
        let mut config = self.load_config();
        config.license = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_python_version(&self, value: String) -> Result<()> {
        let mut config = self.load_config();
        config.python_version = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_min_python_version(&self, value: String) -> Result<()> {
        let mut config = self.load_config();
        config.min_python_version = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_project_manager(&self, value: ProjectManager) -> Result<()> {
        let mut config = self.load_config();
        config.project_manager = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_is_async_project(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.is_async_project = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_is_application(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.is_application = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_github_actions_python_test_versions(&self, value: String) -> Result<()> {
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

        let mut config = self.load_config();
        config.github_actions_python_test_versions = Some(versions);
        config.save()?;

        Ok(())
    }

    pub fn save_max_line_length(&self, value: u8) -> Result<()> {
        let mut config = self.load_config();
        config.max_line_length = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_use_dependabot(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.use_dependabot = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_dependabot_schedule(&self, value: DependabotSchedule) -> Result<()> {
        let mut config = self.load_config();
        config.dependabot_schedule = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_dependabot_day(&self, value: Day) -> Result<()> {
        let mut config = self.load_config();
        config.dependabot_day = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_use_continuous_deployment(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.use_continuous_deployment = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_use_release_drafter(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.use_release_drafter = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_use_multi_os_ci(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.use_multi_os_ci = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_include_docs(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.include_docs = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn save_download_latest_packages(&self, value: bool) -> Result<()> {
        let mut config = self.load_config();
        config.download_latest_packages = Some(value);
        config.save()?;

        Ok(())
    }

    pub fn show(&self) {
        let config = self.load_config();
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
    use tempfile::tempdir;

    fn mock_config() -> Config {
        let mut base = tempdir().unwrap().path().to_path_buf();
        base.push("python-project-generator");
        let config_dir = base.clone();
        create_dir_all(&config_dir).unwrap();
        base.push("config.json");
        let config_file_path = base;

        let config = Config {
            config_dir: Some(config_dir),
            config_file_path: Some(config_file_path),
            ..Default::default()
        };

        config.save().unwrap();

        config
    }

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

    #[test]
    fn test_save_and_load_config() {
        let mut config = mock_config();
        config.creator = Some("Some Person".to_string());
        config.creator_email = Some("someone@email.com".to_string());
        config.save().unwrap();
        let result = config.load_config();

        assert_eq!(result, config);
    }

    #[test]
    fn test_save_creator() {
        let config = mock_config();
        let expected = "Some Person".to_string();
        config.save_creator(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.creator, Some(expected));
    }

    #[test]
    fn test_save_creator_email() {
        let config = mock_config();
        let expected = "someone@email.com".to_string();
        config.save_creator_email(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.creator_email, Some(expected));
    }

    #[test]
    fn test_save_license() {
        let config = mock_config();
        let expected = LicenseType::Apache2;
        config.save_license(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.license, Some(expected));
    }

    #[test]
    fn test_save_python_version() {
        let config = mock_config();
        let expected = "3.12".to_string();
        config.save_python_version(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.python_version, Some(expected));
    }

    #[test]
    fn test_save_min_python_version() {
        let config = mock_config();
        let expected = "3.12".to_string();
        config.save_min_python_version(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.min_python_version, Some(expected));
    }

    #[test]
    fn test_project_manager() {
        let config = mock_config();
        let expected = ProjectManager::Maturin;
        config.save_project_manager(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.project_manager, Some(expected));
    }

    #[test]
    fn test_is_async_project() {
        let config = mock_config();
        let expected = true;
        config.save_is_async_project(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.is_async_project, Some(expected));
    }

    #[test]
    fn test_is_application() {
        let config = mock_config();
        let expected = false;
        config.save_is_application(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.is_application, Some(expected));
    }

    #[test]
    fn test_github_actions_pythong_test_versions() {
        let config = mock_config();
        let expected = vec!["3.11".to_string(), "3.12".to_string()];
        config
            .save_github_actions_python_test_versions("3.11, 3.12".to_string())
            .unwrap();
        let result = config.load_config();

        assert_eq!(result.github_actions_python_test_versions, Some(expected));
    }

    #[test]
    fn test_max_line_length() {
        let config = mock_config();
        let expected = 42;
        config.save_max_line_length(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.max_line_length, Some(expected));
    }

    #[test]
    fn test_use_dependabot() {
        let config = mock_config();
        let expected = false;
        config.save_use_dependabot(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_dependabot, Some(expected));
    }

    #[test]
    fn test_dependabot_schedule() {
        let config = mock_config();
        let expected = DependabotSchedule::Weekly;
        config.save_dependabot_schedule(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_schedule, Some(expected));
    }

    #[test]
    fn test_dependabot_day() {
        let config = mock_config();
        let expected = Day::Monday;
        config.save_dependabot_day(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_day, Some(expected));
    }

    #[test]
    fn test_use_continuous_deployment() {
        let config = mock_config();
        let expected = false;
        config.save_use_continuous_deployment(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_continuous_deployment, Some(expected));
    }

    #[test]
    fn test_use_release_drafter() {
        let config = mock_config();
        let expected = false;
        config.save_use_release_drafter(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_release_drafter, Some(expected));
    }

    #[test]
    fn test_use_multi_os_ci() {
        let config = mock_config();
        let expected = false;
        config.save_use_multi_os_ci(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_multi_os_ci, Some(expected));
    }

    #[test]
    fn test_include_docs() {
        let config = mock_config();
        let expected = true;
        config.save_include_docs(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.include_docs, Some(expected));
    }

    #[test]
    fn test_download_latest_packages() {
        let config = mock_config();
        let expected = false;
        config.save_download_latest_packages(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.download_latest_packages, Some(expected));
    }
}
