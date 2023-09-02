use std::fs::read_to_string;
use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

use crate::project_info::LicenseType;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub settings_dir: Option<PathBuf>,
    pub creator: Option<String>,
    pub creator_email: Option<String>,
    pub license: Option<LicenseType>,
    pub python_version: Option<String>,
    pub min_python_version: Option<String>,
    pub is_application: Option<bool>,
    pub github_action_python_test_versions: Option<Vec<String>>,
    pub max_line_length: Option<u8>,
    pub use_dependabot: Option<bool>,
    pub use_continuous_deployment: Option<bool>,
    pub use_release_drafter: Option<bool>,
    pub use_multi_os_ci: Option<bool>,
    pub download_latest_packages: Option<bool>,
}

impl Config {
    pub fn new() -> Self {
        let config_dir: Option<PathBuf> = match dirs::config_dir() {
            Some(mut c) => {
                c.push("python_project_generator/config.json");
                Some(c)
            }
            None => None,
        };

        Self {
            settings_dir: config_dir,
            creator: None,
            creator_email: None,
            license: None,
            python_version: None,
            min_python_version: None,
            is_application: None,
            github_action_python_test_versions: None,
            max_line_length: None,
            use_dependabot: None,
            use_continuous_deployment: None,
            use_release_drafter: None,
            use_multi_os_ci: None,
            download_latest_packages: None,
        }
    }

    pub fn load_config() -> Result<Self> {
        let settings_dir = dirs::config_dir();
        if let Some(mut config_file) = settings_dir {
            config_file.push("python_project_generator/config.json");
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

    // TODO: Need to implement this
    pub fn _save_config(&self) -> Result<()> {
        Ok(())
    }
}
