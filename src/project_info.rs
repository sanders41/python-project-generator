use std::io::Write;
use std::path::Path;

use clap::ValueEnum;
use colored::*;
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Clone, Debug, Deserialize, Serialize, ValueEnum)]
pub enum LicenseType {
    Mit,
    Apache2,
    NoLicense,
}

struct Prompt {
    prompt_text: String,
    default: Option<String>,
}

trait PromptInput {
    fn show_prompt(&self) -> String;
}

impl PromptInput for Prompt {
    fn show_prompt(&self) -> String {
        let mut input = String::new();

        if let Some(d) = &self.default {
            print!("{} ({d}): ", self.prompt_text);
        } else {
            print!("{}: ", self.prompt_text);
        }

        std::io::stdout().flush().unwrap();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Error: Could not read a line");

        if input.trim() == "" {
            if let Some(d) = &self.default {
                return d.to_string();
            } else {
                let error_message = format!(r#"A "{}" value is required"#, self.prompt_text);
                println!("\n{}", error_message.red());
                std::process::exit(1);
            }
        }

        input.trim().to_string()
    }
}

#[derive(Debug)]
pub struct ProjectInfo {
    pub project_name: String,
    pub project_slug: String,
    pub source_dir: String,
    pub project_description: String,
    pub creator: String,
    pub creator_email: String,
    pub license: LicenseType,
    pub copyright_year: Option<String>,
    pub version: String,
    pub python_version: String,
    pub min_python_version: String,
    pub is_application: bool,
    pub github_actions_python_test_versions: Vec<String>,
    pub max_line_length: u8,
    pub use_dependabot: bool,
    pub use_continuous_deployment: bool,
    pub use_release_drafter: bool,
    pub use_multi_os_ci: bool,
    pub download_latest_packages: bool,
}

fn boolean_prompt(prompt_text: String, default: Option<bool>) -> bool {
    let default_str = match default {
        Some(d) => match d {
            true => "1".to_string(),
            false => "2".to_string(),
        },
        None => "1".to_string(),
    };
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt();

    if input == "1" || input.is_empty() {
        true
    } else if input == "2" {
        false
    } else {
        let error_message = "Invalid selection";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}

fn is_application_prompt(default: Option<bool>) -> bool {
    let prompt_text =
        "Application or Library\n  1 - Application\n  2 - Library\n  Choose from [1, 2]"
            .to_string();
    boolean_prompt(prompt_text, default)
}

pub fn is_valid_python_version(version: &str) -> bool {
    let split_version = version.split('.');
    let split_length = split_version.clone().count();

    if !(2..=3).contains(&split_length) {
        return false;
    }

    for (i, split) in split_version.into_iter().enumerate() {
        match split.parse::<i32>() {
            Ok(s) => {
                if i == 0 && s < 3 || s < 0 {
                    return false;
                }
            }
            _ => return false,
        };
    }

    true
}

fn copyright_year_prompt(license: &LicenseType) -> String {
    let prompt_text = "Copyright Year".to_string();
    let prompt = Prompt {
        prompt_text,
        default: None,
    };
    let input = prompt.show_prompt();

    if input.is_empty() {
        let error_message = format!("A copyright year is required for {:?} license", license);
        println!("\n{}", error_message.red());
        std::process::exit(1);
    } else {
        match input.parse::<i32>() {
            Ok(y) => {
                if !(1000..=9999).contains(&y) {
                    let error_message = format!("{y} is not a valid year");
                    println!("\n{}", error_message.red());
                    std::process::exit(1);
                }
            }
            _ => {
                let error_message = format!("{input} is not a valid year");
                println!("\n{}", error_message.red());
                std::process::exit(1);
            }
        };
    }

    input
}

pub fn get_project_info() -> ProjectInfo {
    let config = match Config::load_config() {
        Ok(c) => c,
        Err(_) => Config::new(),
    };
    let project_name_prompt = Prompt {
        prompt_text: "Project Name".to_string(),
        default: None,
    };
    let project_name = project_name_prompt.show_prompt();
    let project_slug_default = project_name.replace(' ', "-").to_lowercase();
    let project_slug_prompt = Prompt {
        prompt_text: "Project Slug".to_string(),
        default: Some(project_slug_default),
    };
    let project_slug = project_slug_prompt.show_prompt();

    if Path::new(&project_slug).exists() {
        let error_message = format!("The {project_slug} directory already exists");
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    let source_dir_default = project_name.replace(' ', "_").to_lowercase();
    let source_dir_prompt = Prompt {
        prompt_text: "Source Directory".to_string(),
        default: Some(source_dir_default),
    };
    let source_dir = source_dir_prompt.show_prompt();
    let project_description_prompt = Prompt {
        prompt_text: "Project Description".to_string(),
        default: None,
    };
    let project_description = project_description_prompt.show_prompt();
    let creator_prompt = Prompt {
        prompt_text: "Creator".to_string(),
        default: config.creator,
    };
    let creator = creator_prompt.show_prompt();
    let email_prompt = Prompt {
        prompt_text: "Creator Email".to_string(),
        default: config.creator_email,
    };
    let creator_email = email_prompt.show_prompt();
    let license = license_prompt(config.license);

    let copyright_year: Option<String>;
    if let LicenseType::Mit = license {
        copyright_year = Some(copyright_year_prompt(&license));
    } else {
        copyright_year = None;
    }

    let version_prompt = Prompt {
        prompt_text: "Version".to_string(),
        default: Some("0.1.0".to_string()),
    };
    let version = version_prompt.show_prompt();

    let python_version_default = match config.python_version {
        Some(python) => python,
        None => "3.11".to_string(),
    };
    let python_version = python_version_prompt(python_version_default);

    let min_python_version_default = match config.min_python_version {
        Some(python) => python,
        None => "3.8".to_string(),
    };
    let min_python_version = python_min_version_prompt(min_python_version_default);

    let github_actions_python_test_version_default =
        match config.github_actions_python_test_versions {
            Some(versions) => versions.join(", "),
            None => "3.8, 3.9, 3,10, 3.11".to_string(),
        };
    let github_actions_python_test_versions =
        github_actions_python_test_versions_prompt(github_actions_python_test_version_default);

    let is_application = is_application_prompt(config.is_application);
    let max_line_length = max_line_length_prompt(config.max_line_length);
    let use_dependabot = boolean_prompt("Use Dependabot".to_string(), config.use_dependabot);
    let use_continuous_deployment = boolean_prompt(
        "Use Continuous Deployment".to_string(),
        config.use_continuous_deployment,
    );
    let use_release_drafter = boolean_prompt(
        "Use Release Drafter".to_string(),
        config.use_release_drafter,
    );
    let use_multi_os_ci = boolean_prompt("Use Multi OS CI".to_string(), config.use_multi_os_ci);

    ProjectInfo {
        project_name,
        project_slug,
        source_dir,
        project_description,
        creator,
        creator_email,
        license,
        copyright_year,
        version,
        python_version,
        min_python_version,
        is_application,
        github_actions_python_test_versions,
        max_line_length,
        use_dependabot,
        use_continuous_deployment,
        use_release_drafter,
        use_multi_os_ci,
        download_latest_packages: false,
    }
}

fn github_actions_python_test_versions_prompt(default: String) -> Vec<String> {
    let prompt = Prompt {
        prompt_text: "Python Versions for Github Actions Testing".to_string(),
        default: Some(default),
    };
    let input = prompt.show_prompt();
    let mut versions: Vec<String> = Vec::new();

    let version_check = input.replace(' ', "");

    for version in version_check.split(',') {
        if !is_valid_python_version(version) {
            println!("{version}");
            let error_message = format!("{} is not a valid Python Version", version);
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }

        versions.push(version.to_string());
    }

    versions
}

fn license_prompt(default: Option<LicenseType>) -> LicenseType {
    let default_license: Option<String> = match default {
        Some(d) => match d {
            LicenseType::Mit => Some("1".to_string()),
            LicenseType::Apache2 => Some("2".to_string()),
            LicenseType::NoLicense => Some("3".to_string()),
        },
        None => Some("1".to_string()),
    };
    let prompt = Prompt {
        prompt_text:
            "Select License\n  1 - Mit\n  2 - Apache 2\n  3 - No License\n  Choose from [1, 2, 3]"
                .to_string(),
        default: default_license,
    };
    let input = prompt.show_prompt();
    let license: LicenseType;

    if input == "1" || input.is_empty() {
        license = LicenseType::Mit;
    } else if input == "2" {
        license = LicenseType::Apache2;
    } else if input == "3" {
        license = LicenseType::NoLicense;
    } else {
        let error_message = "Invalid license type";
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    license
}

fn max_line_length_prompt(default: Option<u8>) -> u8 {
    let default_val = default.unwrap_or(100);
    let prompt = Prompt {
        prompt_text: "Max Line Length".to_string(),
        default: Some(default_val.to_string()),
    };
    let input = prompt.show_prompt();

    let max_line_length: u8 = match input.parse::<u8>() {
        Ok(m) => m,
        _ => {
            let error_message = format!("{} is not a valid line length", input);
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    };

    max_line_length
}

fn python_min_version_prompt(default: String) -> String {
    let prompt = Prompt {
        prompt_text: "Minimum Python Version".to_string(),
        default: Some(default),
    };
    let input = prompt.show_prompt();

    if !is_valid_python_version(&input) {
        let error_message = format!("{} is not a valid Python Version", input.trim());
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    input.to_string()
}

fn python_version_prompt(default: String) -> String {
    let prompt = Prompt {
        prompt_text: "Python Version".to_string(),
        default: Some(default),
    };
    let input = prompt.show_prompt();

    if !is_valid_python_version(&input) {
        let error_message = format!("{} is not a valid Python Version", input.trim());
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    input.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_two_digit_python_version() {
        assert!(is_valid_python_version("3.9"));
    }

    #[test]
    fn test_valid_three_digit_python_version() {
        assert!(is_valid_python_version("3.11.0"));
    }

    #[test]
    fn test_invalid_python_version_major_less_than_three() {
        assert!(!is_valid_python_version("2.7"));
    }

    #[test]
    fn test_invalid_python_version_too_short() {
        assert!(!is_valid_python_version("3"));
    }

    #[test]
    fn test_invalid_python_version_too_long() {
        assert!(!is_valid_python_version("3.11.0.1"));
    }

    #[test]
    fn test_invalid_python_version_non_numeric_major() {
        assert!(!is_valid_python_version("a.11.0"));
    }

    #[test]
    fn test_invalid_python_version_non_numeric_minor() {
        assert!(!is_valid_python_version("3.a.0"));
    }

    #[test]
    fn test_invalid_python_version_non_numeric_patch() {
        assert!(!is_valid_python_version("3.9.a"));
    }
}
