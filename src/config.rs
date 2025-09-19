use std::{
    fmt::Display,
    fs::{create_dir_all, read_to_string, File},
    path::PathBuf,
    rc::Rc,
};

use anyhow::{bail, Result};
use colored::*;
use serde::{Deserialize, Serialize};

use crate::project_info::{
    is_valid_python_version, Day, DependabotSchedule, LicenseType, ProjectManager,
    Pyo3PythonManager,
};

#[cfg(feature = "fastapi")]
use crate::project_info::DatabaseManager;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Config {
    pub creator: Option<String>,
    pub creator_email: Option<String>,
    pub license: Option<LicenseType>,
    pub python_version: Option<String>,
    pub min_python_version: Option<String>,
    pub project_manager: Option<ProjectManager>,
    pub pyo3_python_manager: Option<Pyo3PythonManager>,
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

    #[cfg(feature = "fastapi")]
    pub is_fastapi_project: Option<bool>,

    #[cfg(feature = "fastapi")]
    pub database_manager: Option<DatabaseManager>,

    #[serde(skip)]
    config_dir: Rc<Option<PathBuf>>,
    #[serde(skip)]
    config_file_path: Rc<Option<PathBuf>>,
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
            pyo3_python_manager: None,
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

            #[cfg(feature = "fastapi")]
            is_fastapi_project: None,

            #[cfg(feature = "fastapi")]
            database_manager: None,
        }
    }
}

impl Config {
    pub fn load_config(&self) -> Self {
        if let Some(config_file) = &*self.config_file_path {
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
                            pyo3_python_manager: config.pyo3_python_manager,
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

                            #[cfg(feature = "fastapi")]
                            is_fastapi_project: config.is_fastapi_project,

                            #[cfg(feature = "fastapi")]
                            database_manager: config.database_manager,
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
        match &*self.config_dir {
            Some(c) => {
                if !c.exists() {
                    create_dir_all(c)?;
                }

                match &*self.config_file_path {
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
        self.handle_save_config(|config| &mut config.creator, Some(value))?;
        Ok(())
    }

    pub fn reset_creator(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.creator, None)?;
        Ok(())
    }

    pub fn save_creator_email(&self, value: String) -> Result<()> {
        self.handle_save_config(|config| &mut config.creator_email, Some(value))?;
        Ok(())
    }

    pub fn reset_creator_email(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.creator_email, None)?;
        Ok(())
    }

    pub fn save_license(&self, value: LicenseType) -> Result<()> {
        self.handle_save_config(|config| &mut config.license, Some(value))?;
        Ok(())
    }

    pub fn reset_license(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.license, None)?;
        Ok(())
    }

    pub fn save_python_version(&self, value: String) -> Result<()> {
        self.handle_save_config(|config| &mut config.python_version, Some(value))?;
        Ok(())
    }

    pub fn reset_python_version(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.python_version, None)?;
        Ok(())
    }

    pub fn save_min_python_version(&self, value: String) -> Result<()> {
        self.handle_save_config(|config| &mut config.min_python_version, Some(value))?;
        Ok(())
    }

    pub fn reset_min_python_version(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.min_python_version, None)?;
        Ok(())
    }

    pub fn save_project_manager(&self, value: ProjectManager) -> Result<()> {
        self.handle_save_config(|config| &mut config.project_manager, Some(value))?;
        Ok(())
    }

    pub fn reset_project_manager(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.project_manager, None)?;
        Ok(())
    }

    pub fn save_pyo3_python_manager(&self, value: Pyo3PythonManager) -> Result<()> {
        self.handle_save_config(|config| &mut config.pyo3_python_manager, Some(value))?;
        Ok(())
    }

    pub fn reset_pyo3_python_manager(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.pyo3_python_manager, None)?;
        Ok(())
    }

    pub fn save_is_async_project(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_async_project, Some(value))?;
        Ok(())
    }

    pub fn reset_is_async_project(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_async_project, None)?;
        Ok(())
    }

    pub fn save_is_application(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_application, Some(value))?;
        Ok(())
    }

    pub fn reset_is_application(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_application, None)?;
        Ok(())
    }

    pub fn save_github_actions_python_test_versions(&self, value: String) -> Result<()> {
        self.handle_save_github_actions_python_test_versions(Some(value))?;
        Ok(())
    }

    pub fn reset_github_actions_python_test_versions(&self) -> Result<()> {
        self.handle_save_github_actions_python_test_versions(None)?;
        Ok(())
    }

    fn handle_save_github_actions_python_test_versions(&self, value: Option<String>) -> Result<()> {
        let mut config = self.load_config();

        if let Some(v) = value {
            let versions = v
                .replace(' ', "")
                .split(',')
                .map(|s| s.to_string())
                .collect::<Vec<String>>();

            for version in &versions {
                if !is_valid_python_version(&version.replace('"', "")) {
                    bail!(format!("{} is not a valid Python Version", version));
                }
            }

            config.github_actions_python_test_versions = Some(versions);
        } else {
            config.github_actions_python_test_versions = None;
        }

        config.save()?;

        Ok(())
    }

    pub fn save_max_line_length(&self, value: u8) -> Result<()> {
        self.handle_save_config(|config| &mut config.max_line_length, Some(value))?;
        Ok(())
    }

    pub fn reset_max_line_length(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.max_line_length, None)?;
        Ok(())
    }

    pub fn save_use_dependabot(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_dependabot, Some(value))?;
        Ok(())
    }

    pub fn reset_use_dependabot(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_dependabot, None)?;
        Ok(())
    }

    pub fn save_dependabot_schedule(&self, value: DependabotSchedule) -> Result<()> {
        self.handle_save_config(|config| &mut config.dependabot_schedule, Some(value))?;
        Ok(())
    }

    pub fn reset_dependabot_schedule(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.dependabot_schedule, None)?;
        Ok(())
    }

    pub fn save_dependabot_day(&self, value: Day) -> Result<()> {
        self.handle_save_config(|config| &mut config.dependabot_day, Some(value))?;
        Ok(())
    }

    pub fn reset_dependabot_day(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.dependabot_day, None)?;
        Ok(())
    }

    pub fn save_use_continuous_deployment(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_continuous_deployment, Some(value))?;
        Ok(())
    }

    pub fn reset_use_continuous_deployment(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_continuous_deployment, None)?;
        Ok(())
    }

    pub fn save_use_release_drafter(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_release_drafter, Some(value))?;
        Ok(())
    }

    pub fn reset_use_release_drafter(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_release_drafter, None)?;
        Ok(())
    }

    pub fn save_use_multi_os_ci(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_multi_os_ci, Some(value))?;
        Ok(())
    }

    pub fn reset_use_multi_os_ci(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.use_multi_os_ci, None)?;
        Ok(())
    }

    pub fn save_include_docs(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.include_docs, Some(value))?;
        Ok(())
    }

    pub fn reset_include_docs(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.include_docs, None)?;
        Ok(())
    }

    pub fn save_download_latest_packages(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.download_latest_packages, Some(value))?;
        Ok(())
    }

    pub fn reset_download_latest_packages(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.download_latest_packages, None)?;
        Ok(())
    }

    fn handle_save_config<F, T>(&self, func: F, value: Option<T>) -> Result<()>
    where
        F: FnOnce(&mut Self) -> &mut Option<T>,
    {
        let mut config = self.load_config();
        let field = func(&mut config);
        *field = value;
        config.save()?;

        Ok(())
    }

    #[cfg(feature = "fastapi")]
    pub fn save_is_fastapi_project(&self, value: bool) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_fastapi_project, Some(value))?;
        Ok(())
    }

    #[cfg(feature = "fastapi")]
    pub fn reset_is_fastapi_project(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.is_fastapi_project, None)?;
        Ok(())
    }

    #[cfg(feature = "fastapi")]
    pub fn save_database_manager(&self, value: DatabaseManager) -> Result<()> {
        self.handle_save_config(|config| &mut config.database_manager, Some(value))?;
        Ok(())
    }

    #[cfg(feature = "fastapi")]
    pub fn reset_database_manager(&self) -> Result<()> {
        self.handle_save_config(|config| &mut config.database_manager, None)?;
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
        print_config_value("PyO3 Python Manager", &config.pyo3_python_manager);
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

        #[cfg(feature = "fastapi")]
        print_config_value("FastAPI Project", &config.is_fastapi_project);

        #[cfg(feature = "fastapi")]
        print_config_value("Database Manager", &config.is_fastapi_project);
    }
}

fn config_dir() -> Rc<Option<PathBuf>> {
    let config_dir: Option<PathBuf> = dirs::config_dir();

    if let Some(mut c) = config_dir {
        c.push("python-project-generator");
        return Rc::new(Some(c));
    }

    Rc::new(None)
}

fn config_file_path() -> Rc<Option<PathBuf>> {
    if let Some(c) = &config_dir().as_ref() {
        let mut c = c.clone();
        c.push("config.json");
        return Rc::new(Some(c));
    };

    Rc::new(None)
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
    use tmp_path::tmp_path;

    #[tmp_path]
    fn mock_config() -> Config {
        tmp_path.push("python-project-generator");
        let config_dir = tmp_path.clone();
        create_dir_all(&config_dir).unwrap();
        tmp_path.push("config.json");
        let config_file_path = tmp_path;

        let config = Config {
            config_dir: Some(config_dir).into(),
            config_file_path: Some(config_file_path).into(),
            ..Default::default()
        };

        config.save().unwrap();

        config
    }

    #[test]
    fn test_config_dir() {
        let config_dir = config_dir();
        assert_ne!(config_dir, Rc::new(None));
        let config = config_dir.as_ref().as_ref().unwrap();

        let last = config.file_name();
        assert_ne!(last, None);
        assert_eq!(last.unwrap(), "python-project-generator");
    }

    #[test]
    fn test_config_file_path() {
        let config_file_path = config_file_path();
        assert_ne!(config_file_path, Rc::new(None));
        let mut config = config_file_path.as_ref().as_ref().unwrap().clone();

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
    fn test_reset_creator() {
        let config = mock_config();
        config.save_creator("Some Person".to_string()).unwrap();
        config.reset_creator().unwrap();
        let result = config.load_config();

        assert!(result.creator.is_none());
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
    fn test_reset_creator_email() {
        let config = mock_config();
        config
            .save_creator_email("someone@email.com".to_string())
            .unwrap();
        config.reset_creator_email().unwrap();
        let result = config.load_config();

        assert_eq!(result.creator_email, None);
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
    fn test_reset_license() {
        let config = mock_config();
        config.save_license(LicenseType::Apache2).unwrap();
        config.reset_license().unwrap();
        let result = config.load_config();

        assert_eq!(result.license, None);
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
    fn test_reset_python_version() {
        let config = mock_config();
        config.save_python_version("3.12".to_string()).unwrap();
        config.reset_python_version().unwrap();
        let result = config.load_config();

        assert_eq!(result.python_version, None);
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
    fn test_reset_min_python_version() {
        let config = mock_config();
        config.save_min_python_version("3.12".to_string()).unwrap();
        config.reset_min_python_version().unwrap();
        let result = config.load_config();

        assert_eq!(result.min_python_version, None);
    }

    #[test]
    fn test_save_project_manager() {
        let config = mock_config();
        let expected = ProjectManager::Maturin;
        config.save_project_manager(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.project_manager, Some(expected));
    }

    #[test]
    fn test_reset_project_manager() {
        let config = mock_config();
        config
            .save_project_manager(ProjectManager::Maturin)
            .unwrap();
        config.reset_project_manager().unwrap();
        let result = config.load_config();

        assert_eq!(result.project_manager, None);
    }

    #[test]
    fn test_save_pyo3_python_manger() {
        let config = mock_config();
        let expected = Pyo3PythonManager::Uv;
        config.save_pyo3_python_manager(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.pyo3_python_manager, Some(expected));
    }

    #[test]
    fn test_reset_pyo3_project_manager() {
        let config = mock_config();
        config
            .save_pyo3_python_manager(Pyo3PythonManager::Uv)
            .unwrap();
        config.reset_pyo3_python_manager().unwrap();
        let result = config.load_config();

        assert_eq!(result.pyo3_python_manager, None);
    }

    #[test]
    fn test_save_is_async_project() {
        let config = mock_config();
        let expected = true;
        config.save_is_async_project(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.is_async_project, Some(expected));
    }

    #[test]
    fn test_reset_is_async_project() {
        let config = mock_config();
        config.save_is_async_project(true).unwrap();
        config.reset_is_async_project().unwrap();
        let result = config.load_config();

        assert_eq!(result.is_async_project, None);
    }

    #[test]
    fn test_save_is_application() {
        let config = mock_config();
        let expected = false;
        config.save_is_application(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.is_application, Some(expected));
    }

    #[test]
    fn test_reset_is_application() {
        let config = mock_config();
        config.save_is_application(false).unwrap();
        config.reset_is_application().unwrap();
        let result = config.load_config();

        assert_eq!(result.is_application, None);
    }

    #[test]
    fn test_save_github_actions_pythong_test_versions() {
        let config = mock_config();
        let expected = vec!["3.11".to_string(), "3.12".to_string()];
        config
            .save_github_actions_python_test_versions("3.11, 3.12".to_string())
            .unwrap();
        let result = config.load_config();

        assert_eq!(result.github_actions_python_test_versions, Some(expected));
    }

    #[test]
    fn test_reset_github_actions_pythong_test_versions() {
        let config = mock_config();
        config
            .save_github_actions_python_test_versions("3.11, 3.12".to_string())
            .unwrap();
        config.reset_github_actions_python_test_versions().unwrap();
        let result = config.load_config();

        assert_eq!(result.github_actions_python_test_versions, None);
    }

    #[test]
    fn test_save_max_line_length() {
        let config = mock_config();
        let expected = 42;
        config.save_max_line_length(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.max_line_length, Some(expected));
    }

    #[test]
    fn test_reset_max_line_length() {
        let config = mock_config();
        config.save_max_line_length(42).unwrap();
        config.reset_max_line_length().unwrap();
        let result = config.load_config();

        assert_eq!(result.max_line_length, None);
    }

    #[test]
    fn test_save_use_dependabot() {
        let config = mock_config();
        let expected = false;
        config.save_use_dependabot(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_dependabot, Some(expected));
    }

    #[test]
    fn test_reset_use_dependabot() {
        let config = mock_config();
        config.save_use_dependabot(false).unwrap();
        config.reset_use_dependabot().unwrap();
        let result = config.load_config();

        assert_eq!(result.use_dependabot, None);
    }

    #[test]
    fn test_save_dependabot_schedule() {
        let config = mock_config();
        let expected = DependabotSchedule::Weekly;
        config.save_dependabot_schedule(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_schedule, Some(expected));
    }

    #[test]
    fn test_reset_dependabot_schedule() {
        let config = mock_config();
        config
            .save_dependabot_schedule(DependabotSchedule::Weekly)
            .unwrap();
        config.reset_dependabot_schedule().unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_schedule, None);
    }

    #[test]
    fn test_save_dependabot_day() {
        let config = mock_config();
        let expected = Day::Monday;
        config.save_dependabot_day(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_day, Some(expected));
    }

    #[test]
    fn test_reset_dependabot_day() {
        let config = mock_config();
        config.save_dependabot_day(Day::Tuesday).unwrap();
        config.reset_dependabot_day().unwrap();
        let result = config.load_config();

        assert_eq!(result.dependabot_day, None);
    }

    #[test]
    fn test_save_use_continuous_deployment() {
        let config = mock_config();
        let expected = false;
        config.save_use_continuous_deployment(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_continuous_deployment, Some(expected));
    }

    #[test]
    fn test_reset_use_continuous_deployment() {
        let config = mock_config();
        config.save_use_continuous_deployment(false).unwrap();
        config.reset_use_continuous_deployment().unwrap();
        let result = config.load_config();

        assert_eq!(result.use_continuous_deployment, None);
    }

    #[test]
    fn test_save_use_release_drafter() {
        let config = mock_config();
        let expected = false;
        config.save_use_release_drafter(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_release_drafter, Some(expected));
    }

    #[test]
    fn test_reset_use_release_drafter() {
        let config = mock_config();
        config.save_use_release_drafter(false).unwrap();
        config.reset_use_release_drafter().unwrap();
        let result = config.load_config();

        assert_eq!(result.use_release_drafter, None);
    }

    #[test]
    fn test_save_use_multi_os_ci() {
        let config = mock_config();
        let expected = false;
        config.save_use_multi_os_ci(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.use_multi_os_ci, Some(expected));
    }

    #[test]
    fn test_reset_use_multi_os_ci() {
        let config = mock_config();
        config.save_use_multi_os_ci(false).unwrap();
        config.reset_use_multi_os_ci().unwrap();
        let result = config.load_config();

        assert_eq!(result.use_multi_os_ci, None);
    }

    #[test]
    fn test_save_include_docs() {
        let config = mock_config();
        let expected = true;
        config.save_include_docs(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.include_docs, Some(expected));
    }

    #[test]
    fn test_reset_include_docs() {
        let config = mock_config();
        config.save_include_docs(true).unwrap();
        config.reset_include_docs().unwrap();
        let result = config.load_config();

        assert_eq!(result.include_docs, None);
    }

    #[test]
    fn test_save_download_latest_packages() {
        let config = mock_config();
        let expected = false;
        config.save_download_latest_packages(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.download_latest_packages, Some(expected));
    }

    #[test]
    fn test_reset_download_latest_packages() {
        let config = mock_config();
        config.save_download_latest_packages(false).unwrap();
        config.reset_download_latest_packages().unwrap();
        let result = config.load_config();

        assert_eq!(result.download_latest_packages, None);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_is_fastapi_project() {
        let config = mock_config();
        let expected = true;
        config.save_is_fastapi_project(expected).unwrap();
        let result = config.load_config();

        assert_eq!(result.is_fastapi_project, Some(expected));
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_reset_is_fastapi_project() {
        let config = mock_config();
        config.save_is_fastapi_project(true).unwrap();
        config.reset_is_fastapi_project().unwrap();
        let result = config.load_config();

        assert_eq!(result.is_fastapi_project, None);
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_save_database_manager() {
        let config = mock_config();
        let expected = DatabaseManager::AsyncPg;
        config.save_database_manager(expected.clone()).unwrap();
        let result = config.load_config();

        assert_eq!(result.database_manager, Some(expected));
    }

    #[cfg(feature = "fastapi")]
    #[test]
    fn test_reset_database_manager() {
        let config = mock_config();
        config
            .save_database_manager(DatabaseManager::SqlAlchemy)
            .unwrap();
        config.reset_database_manager().unwrap();
        let result = config.load_config();

        assert_eq!(result.database_manager, None);
    }
}
