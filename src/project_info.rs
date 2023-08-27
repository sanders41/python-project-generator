use std::io::Write;

use colored::*;

#[derive(Debug)]
pub enum LicenseType {
    MIT,
    Apache2,
    NoLicense,
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
    pub copywright_year: Option<String>,
    pub python_version: String,
    pub min_python_version: String,
    pub is_application: bool,
    pub github_action_python_test_versions: String,
    pub max_line_length: u8,
    pub use_dependabot: bool,
    pub use_continuous_deployment: bool,
    pub use_release_drafter: bool,
    pub use_multi_os_ci: bool,
}

fn boolean_prompt(prompt_message: &str) -> bool {
    let mut line = String::new();

    println!("{prompt_message}");
    println!("  1 - Yes");
    println!("  2 - No");
    print!("  Choose from [1, 2] (1): ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "1" || line.trim() == "" {
        return true;
    } else if line.trim() == "2" {
        return false;
    } else {
        let error_message = format!("Invalid selection");
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}

fn is_application_prompt() -> bool {
    let mut line = String::new();

    println!("Application or Library");
    println!("  1 - Application");
    println!("  2 - Library");
    print!("  Choose from [1, 2] (1): ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "1" || line.trim() == "" {
        return true;
    } else if line.trim() == "2" {
        return false;
    } else {
        let error_message = format!("Invalid license type");
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }
}

fn is_valid_python_version(version: &str) -> bool {
    let split_version = version.split(".");
    let split_length = split_version.clone().count();

    if split_length < 2 || split_length > 3 {
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

fn copywright_year_prompt(license: &LicenseType) -> String {
    let mut line = String::new();

    print!("Copywright Year: ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "" {
        let error_message = format!("A copywright year is required for {:?} license", license);
        println!("\n{}", error_message.red());
        std::process::exit(1);
    } else {
        match line.trim().parse::<i32>() {
            Ok(y) => {
                if y < 1000 || y > 9999 {
                    let error_message = format!("{y} is not a valid year");
                    println!("\n{}", error_message.red());
                    std::process::exit(1);
                }
            }
            _ => {
                let error_message = format!("{} is not a valid year", line.trim());
                println!("\n{}", error_message.red());
                std::process::exit(1);
            }
        };
    }

    line
}

pub fn get_project_info() -> ProjectInfo {
    let project_name = prompt("Project Name", None);
    let project_slug_default = &project_name.replace(" ", "-").to_lowercase();
    let project_slug = prompt("Project Slug", Some(project_slug_default));
    let source_dir_default = &project_name.replace(" ", "_").to_lowercase();
    let source_dir = prompt("Source Directory", Some(source_dir_default));
    let project_description = prompt("Project Description", None);
    let creator = prompt("Creator", None);
    let creator_email = prompt("Creator Email", None);
    let license = license_prompt();

    let copywright_year: Option<String>;
    if let LicenseType::MIT = license {
        copywright_year = Some(copywright_year_prompt(&license));
    } else {
        copywright_year = None;
    }

    let python_version = python_version_prompt("3.11");
    let min_python_version = python_version_prompt("3.8");
    let github_action_python_test_versions =
        github_action_python_test_versions_prompt("3.8, 3.9, 3.10, 3.11".to_string());
    let is_application = is_application_prompt();
    let max_line_length = max_line_length_prompt();
    let use_dependabot = boolean_prompt("Use Dependabot");
    let use_continuous_deployment = boolean_prompt("Use Continuous Deployment");
    let use_release_drafter = boolean_prompt("Use Release Drafter");
    let use_multi_os_ci = boolean_prompt("Use Multi OS CI");

    ProjectInfo {
        project_name,
        project_slug,
        source_dir,
        project_description,
        creator,
        creator_email,
        license,
        copywright_year,
        python_version,
        min_python_version,
        is_application,
        github_action_python_test_versions,
        max_line_length,
        use_dependabot,
        use_continuous_deployment,
        use_release_drafter,
        use_multi_os_ci,
    }
}

fn github_action_python_test_versions_prompt(default: String) -> String {
    let mut line = String::new();

    print!("Python Versions for Github Actions Testing ({default}): ");
    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "" {
        return default;
    }

    let version_check = line.replace(" ", "");
    println!("{version_check}");

    for version in version_check.split(",") {
        if !is_valid_python_version(version) {
            let error_message = format!("{} is not a valid Python Version", version);
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    line.trim().to_string()
}

fn license_prompt() -> LicenseType {
    let mut line = String::new();
    let license: LicenseType;

    println!("Select License");
    println!("  1 - MIT");
    println!("  2 - Apache 2");
    println!("  3 - No License");
    print!("  Choose from [1, 2, 3] (1): ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "1" || line.trim() == "" {
        license = LicenseType::MIT;
    } else if line.trim() == "2" {
        license = LicenseType::Apache2;
    } else if line.trim() == "3" {
        license = LicenseType::NoLicense;
    } else {
        let error_message = format!("Invalid license type");
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    license
}

fn max_line_length_prompt() -> u8 {
    let mut line = String::new();
    let default: u8 = 100;

    print!("Max Line Length ({default}): ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "" {
        return default;
    }

    let max_line_length: u8 = match line.trim().parse::<u8>() {
        Ok(m) => m,
        _ => {
            let error_message = format!("{} is not a valid line length", line.trim());
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    };

    max_line_length
}

fn prompt(prompt_text: &str, value_default: Option<&str>) -> String {
    let mut line = String::new();

    if let Some(default) = value_default {
        print!("{prompt_text} ({default}): ");
    } else {
        print!("{prompt_text}: ");
    }

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "" {
        if let Some(default) = value_default {
            return default.to_string();
        } else {
            let error_message = format!(r#"A "{prompt_text}" value is required"#);
            println!("\n{}", error_message.red());
            std::process::exit(1);
        }
    }

    line.trim().to_string()
}

fn python_version_prompt(default: &str) -> String {
    let mut line = String::new();

    print!("Python Version ({default}): ");

    std::io::stdout().flush().unwrap();
    std::io::stdin()
        .read_line(&mut line)
        .expect("Error: Could not read a line");

    if line.trim() == "" {
        line = default.to_string();
    }

    if !is_valid_python_version(line.trim()) {
        let error_message = format!("{} is not a valid Python Version", line.trim());
        println!("\n{}", error_message.red());
        std::process::exit(1);
    }

    line.trim().to_string()
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
