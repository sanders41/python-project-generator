use std::{
    fmt,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::config::Config;

#[cfg(feature = "fastapi")]
use crate::utils::is_allowed_fastapi_python_version;

#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum DependabotSchedule {
    #[default]
    Daily,
    Weekly,
    Monthly,
}

impl fmt::Display for DependabotSchedule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Daily => write!(f, "Daily"),
            Self::Weekly => write!(f, "Weekly"),
            Self::Monthly => write!(f, "Montly"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum Day {
    #[default]
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Monday => write!(f, "Monday"),
            Self::Tuesday => write!(f, "Tuesday"),
            Self::Wednesday => write!(f, "Wednesday"),
            Self::Thursday => write!(f, "Thursday"),
            Self::Friday => write!(f, "Friday"),
            Self::Saturday => write!(f, "Saturday"),
            Self::Sunday => write!(f, "Sunday"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum LicenseType {
    #[default]
    Mit,
    Apache2,
    NoLicense,
}

impl fmt::Display for LicenseType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Mit => write!(f, "MIT"),
            Self::Apache2 => write!(f, "Apache 2.0"),
            Self::NoLicense => write!(f, "No License"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum Pyo3PythonManager {
    #[default]
    Uv,
    Setuptools,
}

impl fmt::Display for Pyo3PythonManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Uv => write!(f, "uv"),
            Self::Setuptools => write!(f, "Setuptools"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum ProjectManager {
    Maturin,
    Poetry,
    Setuptools,
    #[default]
    Uv,
    Pixi,
}

impl fmt::Display for ProjectManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Maturin => write!(f, "Maturin"),
            Self::Poetry => write!(f, "Poetry"),
            Self::Setuptools => write!(f, "Setuptools"),
            Self::Uv => write!(f, "uv"),
            Self::Pixi => write!(f, "Pixi"),
        }
    }
}

#[cfg(feature = "fastapi")]
#[derive(Clone, Debug, Default, Deserialize, Serialize, ValueEnum, PartialEq, Eq)]
pub enum DatabaseManager {
    #[default]
    AsyncPg,
    SqlAlchemy,
}

#[cfg(feature = "fastapi")]
impl fmt::Display for DatabaseManager {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::AsyncPg => write!(f, "asyncpg"),
            Self::SqlAlchemy => write!(f, "SQLAlchemy"),
        }
    }
}

struct Prompt {
    prompt_text: String,
    default: Option<String>,
}

impl Prompt {
    fn show_prompt(&self) -> Result<String> {
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
                return Ok(d.to_string());
            } else {
                bail!(format!(r#"A "{}" value is required"#, self.prompt_text));
            }
        }

        Ok(input.trim().to_string())
    }
}

#[derive(Debug)]
pub struct DocsInfo {
    pub site_name: String,
    pub site_description: String,
    pub site_url: String,
    pub locale: String,
    pub repo_name: String,
    pub repo_url: String,
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
    pub project_manager: ProjectManager,
    pub pyo3_python_manager: Option<Pyo3PythonManager>,
    pub is_async_project: bool,
    pub is_application: bool,
    pub github_actions_python_test_versions: Vec<String>,
    pub max_line_length: u8,
    pub use_dependabot: bool,
    pub dependabot_schedule: Option<DependabotSchedule>,
    pub dependabot_day: Option<Day>,
    pub use_continuous_deployment: bool,
    pub use_release_drafter: bool,
    pub use_multi_os_ci: bool,
    pub include_docs: bool,
    pub docs_info: Option<DocsInfo>,
    pub download_latest_packages: bool,
    pub project_root_dir: Option<PathBuf>,

    #[cfg(feature = "fastapi")]
    pub is_fastapi_project: bool,

    #[cfg(feature = "fastapi")]
    pub database_manager: Option<DatabaseManager>,
}

impl ProjectInfo {
    pub fn base_dir(&self) -> PathBuf {
        match &self.project_root_dir {
            Some(root) => PathBuf::from(&format!("{}/{}", root.display(), self.project_slug)),
            None => PathBuf::from(&self.project_slug),
        }
    }

    pub fn module_name(&self) -> String {
        self.source_dir.replace([' ', '-'], "_")
    }

    pub fn source_dir_path(&self) -> PathBuf {
        let base = self.base_dir();
        base.join(&self.source_dir)
    }
}

/// `selected_default` is the value passed from the saved `default` values. default is used if
/// `selected_default` is None.
fn boolean_prompt(
    prompt_text: String,
    selected_default: Option<bool>,
    default: bool,
) -> Result<bool> {
    let default_str = match selected_default {
        Some(d) => match d {
            true => "1".to_string(),
            false => "2".to_string(),
        },
        None => {
            if default {
                "1".to_string()
            } else {
                "2".to_string()
            }
        }
    };

    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" || input.is_empty() {
        Ok(true)
    } else if input == "2" {
        Ok(false)
    } else {
        bail!("Invalid selection");
    }
}

fn default_or_prompt_bool(
    prompt_text: String,
    selected_default: Option<bool>,
    default: bool,
    use_defaults: bool,
) -> Result<bool> {
    if use_defaults {
        return Ok(selected_default.unwrap_or(default));
    }

    let result = boolean_prompt(prompt_text, selected_default, default)?;

    Ok(result)
}

fn string_prompt(prompt_text: String, default: Option<String>) -> Result<String> {
    let prompt = Prompt {
        prompt_text,
        default,
    };
    let value = prompt.show_prompt()?;

    Ok(value)
}

fn default_or_prompt_string(
    prompt_text: String,
    default: Option<String>,
    use_defaults: bool,
) -> Result<String> {
    if use_defaults {
        if let Some(d) = default {
            return Ok(d);
        }
    }

    let result = string_prompt(prompt_text, default)?;

    Ok(result)
}

fn dependabot_day_prompt(default: Option<Day>) -> Result<Option<Day>> {
    let default_str = match default {
        Some(s) => match s {
            Day::Monday => "1".to_string(),
            Day::Tuesday => "2".to_string(),
            Day::Wednesday => "3".to_string(),
            Day::Thursday => "4".to_string(),
            Day::Friday => "5".to_string(),
            Day::Saturday => "6".to_string(),
            Day::Sunday => "6".to_string(),
        },
        None => "1".to_string(),
    };
    let prompt_text =
        "Dependabot Day\n  1 - Monday\n  2 - Tuesday\n  3 - Wednesday\n  4 - Thursday\n  5 - Friday\n  6 - Saturday\n  7 - Sunday\n  Choose from [1, 2, 3, 4, 5, 6, 7]"
            .to_string();
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" || input.is_empty() {
        Ok(Some(Day::Monday))
    } else if input == "2" {
        Ok(Some(Day::Tuesday))
    } else if input == "3" {
        Ok(Some(Day::Wednesday))
    } else if input == "4" {
        Ok(Some(Day::Thursday))
    } else if input == "5" {
        Ok(Some(Day::Friday))
    } else if input == "6" {
        Ok(Some(Day::Saturday))
    } else if input == "7" {
        Ok(Some(Day::Sunday))
    } else {
        bail!("Invalid selection");
    }
}

fn dependabot_schedule_prompt(
    default: Option<DependabotSchedule>,
) -> Result<Option<DependabotSchedule>> {
    let default_str = match default {
        Some(s) => match s {
            DependabotSchedule::Daily => "1".to_string(),
            DependabotSchedule::Weekly => "2".to_string(),
            DependabotSchedule::Monthly => "3".to_string(),
        },
        None => "1".to_string(),
    };
    let prompt_text =
        "Dependabot Schedule\n  1 - Daily\n  2 - Weekly\n  3 - Monthly\n  Choose from [1, 2, 3]"
            .to_string();
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" || input.is_empty() {
        Ok(Some(DependabotSchedule::Daily))
    } else if input == "2" {
        Ok(Some(DependabotSchedule::Weekly))
    } else if input == "3" {
        Ok(Some(DependabotSchedule::Monthly))
    } else {
        bail!("Invalid selection");
    }
}

fn project_manager_prompt(default: Option<ProjectManager>) -> Result<ProjectManager> {
    let default_str = match default {
        Some(d) => match d {
            ProjectManager::Uv => "1".to_string(),
            ProjectManager::Poetry => "2".to_string(),
            ProjectManager::Maturin => "3".to_string(),
            ProjectManager::Setuptools => "4".to_string(),
            ProjectManager::Pixi => "5".to_string(),
        },
        None => "poetry".to_string(),
    };
    let prompt_text =
        "Project Manager\n  1 - uv\n  2 - Poetry\n  3 - Maturin\n  4 - setuptools\n  5 - Pixi\n  Choose from [1, 2, 3, 4, 5]"
            .to_string();
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" {
        Ok(ProjectManager::Uv)
    } else if input == "2" || input.is_empty() {
        Ok(ProjectManager::Poetry)
    } else if input == "3" {
        Ok(ProjectManager::Maturin)
    } else if input == "4" {
        Ok(ProjectManager::Setuptools)
    } else if input == "5" {
        Ok(ProjectManager::Pixi)
    } else {
        bail!("Invalid selection");
    }
}

fn pyo3_python_manager_prompt(default: Option<Pyo3PythonManager>) -> Result<Pyo3PythonManager> {
    let default_str = match default {
        Some(d) => match d {
            Pyo3PythonManager::Uv => "1".to_string(),
            Pyo3PythonManager::Setuptools => "2".to_string(),
        },
        None => "Uv".to_string(),
    };
    let prompt_text =
        "PyO3 Python Manager\n  1 - uv\n  2 - setuptools\n  Choose from [1, 2]".to_string();
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" {
        Ok(Pyo3PythonManager::Uv)
    } else if input == "2" {
        Ok(Pyo3PythonManager::Setuptools)
    } else {
        bail!("Invalid selection");
    }
}

pub fn is_valid_python_version(version: &str) -> bool {
    let split_version: Vec<&str> = version.split('.').collect();
    let split_length = split_version.len();

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

fn copyright_year_prompt(license: &LicenseType, default: Option<String>) -> Result<String> {
    let prompt_text = "Copyright Year".to_string();
    let prompt = Prompt {
        prompt_text,
        default,
    };
    let input = prompt.show_prompt()?;

    if input.is_empty() {
        bail!(format!(
            "A copyright year is required for {} license",
            license
        ));
    } else {
        match input.parse::<i32>() {
            Ok(y) => {
                if !(1000..=9999).contains(&y) {
                    bail!(format!("{y} is not a valid year"));
                }
            }
            _ => {
                bail!(format!("{input} is not a valid year"));
            }
        };
    }

    Ok(input)
}

#[cfg(feature = "fastapi")]
fn database_manager_prompt(default: Option<DatabaseManager>) -> Result<DatabaseManager> {
    let default_str = match default {
        Some(d) => match d {
            DatabaseManager::AsyncPg => "1".to_string(),
            DatabaseManager::SqlAlchemy => "2".to_string(),
        },
        None => "AsyncPg".to_string(),
    };
    let prompt_text =
        "Database Manager\n  1 - asyncpg\n  2 - SQLAlchemy Choose from [1, 2]".to_string();
    let prompt = Prompt {
        prompt_text,
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;

    if input == "1" {
        Ok(DatabaseManager::AsyncPg)
    } else if input == "2" || input.is_empty() {
        Ok(DatabaseManager::SqlAlchemy)
    } else {
        bail!("Invalid selection");
    }
}

pub fn get_project_info(use_defaults: bool) -> Result<ProjectInfo> {
    let config = Config::default().load_config();
    let project_name = string_prompt("Project Name".to_string(), None)?;
    let project_slug_default = project_name.replace(' ', "-").to_lowercase();
    let project_slug = default_or_prompt_string(
        "Project Slug".to_string(),
        Some(project_slug_default),
        use_defaults,
    )?;

    if Path::new(&project_slug).exists() {
        bail!(format!("The {project_slug} directory already exists"));
    }

    let source_dir_default = project_name.replace([' ', '-'], "_").to_lowercase();
    let source_dir = default_or_prompt_string(
        "Source Directory".to_string(),
        Some(source_dir_default),
        use_defaults,
    )?;
    let project_description = string_prompt("Project Description".to_string(), None)?;
    let creator = default_or_prompt_string("Creator".to_string(), config.creator, use_defaults)?;
    let creator_email = default_or_prompt_string(
        "Creator Email".to_string(),
        config.creator_email,
        use_defaults,
    )?;
    let license = if use_defaults {
        config.license.unwrap_or_default()
    } else {
        license_prompt(config.license)?
    };
    let copyright_year = if let LicenseType::Mit = license {
        if let Ok(now) = OffsetDateTime::now_local() {
            if use_defaults {
                Some(now.year().to_string())
            } else {
                let result = copyright_year_prompt(&license, Some(now.year().to_string()))?;
                Some(result)
            }
        } else {
            None
        }
    } else {
        None
    };

    let default_version = "0.1.0".to_string();
    let version =
        default_or_prompt_string("Version".to_string(), Some(default_version), use_defaults)?;

    #[cfg(feature = "fastapi")]
    let is_fastapi_project = default_or_prompt_bool(
        "FastAPI Project\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
        config.is_fastapi_project,
        false,
        use_defaults,
    )?;

    let python_version_default = match config.python_version {
        Some(python) => python,
        None => "3.13".to_string(),
    };
    let python_version = if use_defaults {
        python_version_default
    } else {
        python_version_prompt(python_version_default)?
    };

    #[cfg(feature = "fastapi")]
    if is_fastapi_project && !is_allowed_fastapi_python_version(&python_version)? {
        bail!("The minimum supported Python version for FastAPI projects is 3.11");
    }

    let min_python_version_default = {
        #[cfg(feature = "fastapi")]
        {
            if is_fastapi_project {
                match config.min_python_version {
                    Some(python) => python,
                    None => "3.11".to_string(),
                }
            } else {
                match config.min_python_version {
                    Some(python) => python,
                    None => "3.9".to_string(),
                }
            }
        }
        #[cfg(not(feature = "fastapi"))]
        {
            match config.min_python_version {
                Some(python) => python,
                None => "3.9".to_string(),
            }
        }
    };

    let min_python_version = if use_defaults {
        min_python_version_default
    } else {
        python_min_version_prompt(min_python_version_default)?
    };

    #[cfg(feature = "fastapi")]
    if is_fastapi_project && !is_allowed_fastapi_python_version(&min_python_version)? {
        bail!("The minimum supported Python version for FastAPI projects is 3.11");
    }

    let github_actions_python_test_version_default =
        match config.github_actions_python_test_versions {
            Some(versions) => versions,
            None => {
                let mut split_version = min_python_version.split('.');
                if let Some(v) = split_version.nth(1) {
                    let min = v.parse::<i32>()?;
                    if min >= 12 {
                        vec![format!("3.{min}")]
                    } else {
                        let mut versions: Vec<String> = Vec::new();

                        // Up to 3.13
                        for i in min..14 {
                            versions.push(format!("3.{i}"));
                        }

                        versions
                    }
                } else {
                    #[cfg(feature = "fastapi")]
                    if is_fastapi_project {
                        vec!["3.11".to_string(), "3.12".to_string(), "3.13".to_string()]
                    } else {
                        vec![
                            "3.9".to_string(),
                            "3.10".to_string(),
                            "3.11".to_string(),
                            "3.12".to_string(),
                            "3.13".to_string(),
                        ]
                    }

                    #[cfg(not(feature = "fastapi"))]
                    {
                        vec![
                            "3.9".to_string(),
                            "3.10".to_string(),
                            "3.11".to_string(),
                            "3.12".to_string(),
                            "3.13".to_string(),
                        ]
                    }
                }
            }
        };
    let github_actions_python_test_versions = if use_defaults {
        github_actions_python_test_version_default
    } else {
        github_actions_python_test_versions_prompt(github_actions_python_test_version_default)?
    };

    let project_manager = if use_defaults {
        config.project_manager.unwrap_or_default()
    } else {
        let default = config.project_manager.unwrap_or_default();
        project_manager_prompt(Some(default))?
    };

    #[cfg(feature = "fastapi")]
    if is_fastapi_project && project_manager == ProjectManager::Pixi {
        bail!("Pixi is not currently supported for FastAPI projects");
    }

    let pyo3_python_manager = if project_manager == ProjectManager::Maturin {
        if use_defaults {
            if let Some(default) = config.pyo3_python_manager {
                Some(default)
            } else {
                let default = config.pyo3_python_manager.unwrap_or_default();
                Some(pyo3_python_manager_prompt(Some(default))?)
            }
        } else {
            let default = config.pyo3_python_manager.unwrap_or_default();
            Some(pyo3_python_manager_prompt(Some(default))?)
        }
    } else {
        None
    };

    let is_application = default_or_prompt_bool(
        "Application or Library\n  1 - Application\n  2 - Library\n  Choose from [1, 2]"
            .to_string(),
        config.is_application,
        true,
        use_defaults,
    )?;

    #[cfg(feature = "fastapi")]
    let database_manager = if is_fastapi_project {
        if use_defaults {
            Some(config.database_manager.unwrap_or_default())
        } else {
            let default = config.database_manager.unwrap_or_default();
            Some(database_manager_prompt(Some(default))?)
        }
    } else {
        None
    };

    let is_async_project = {
        #[cfg(feature = "fastapi")]
        {
            if is_fastapi_project {
                true
            } else {
                default_or_prompt_bool(
                    "Async Project\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
                    config.is_async_project,
                    false,
                    use_defaults,
                )?
            }
        }
        #[cfg(not(feature = "fastapi"))]
        {
            default_or_prompt_bool(
                "Async Project\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
                config.is_async_project,
                false,
                use_defaults,
            )?
        }
    };

    let max_line_length = if use_defaults {
        config.max_line_length.unwrap_or(100)
    } else {
        max_line_length_prompt(config.max_line_length)?
    };

    let use_dependabot = if use_defaults {
        config.use_dependabot.unwrap_or(true)
    } else {
        boolean_prompt(
            "Use Dependabot\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
            config.use_dependabot,
            true,
        )?
    };

    let dependabot_schedule = if use_dependabot {
        if use_defaults {
            Some(config.dependabot_schedule.unwrap_or_default())
        } else {
            dependabot_schedule_prompt(Some(DependabotSchedule::default()))?
        }
    } else {
        None
    };

    let dependabot_day = if use_dependabot && use_defaults {
        Some(config.dependabot_day.unwrap_or_default())
    } else if let Some(DependabotSchedule::Weekly) = &dependabot_schedule {
        dependabot_day_prompt(Some(Day::default()))?
    } else {
        None
    };
    let use_continuous_deployment = default_or_prompt_bool(
        "Use Continuous Deployment\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
        config.use_continuous_deployment,
        true,
        use_defaults,
    )?;
    let use_release_drafter = default_or_prompt_bool(
        "Use Release Drafter\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
        config.use_release_drafter,
        true,
        use_defaults,
    )?;
    let use_multi_os_ci = default_or_prompt_bool(
        "Use Multi OS CI\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
        config.use_multi_os_ci,
        true,
        use_defaults,
    )?;
    let include_docs = default_or_prompt_bool(
        "Include Docs\n  1 - Yes\n  2 - No\n  Choose from [1, 2]".to_string(),
        config.include_docs,
        false,
        use_defaults,
    )?;

    let docs_info = if include_docs {
        let site_name = string_prompt("Docs Site Name".to_string(), None)?;
        let site_description = string_prompt("Docs Site Description".to_string(), None)?;
        let site_url = string_prompt("Docs Site Url".to_string(), None)?;
        let locale = string_prompt("Docs Locale".to_string(), Some("en".to_string()))?;
        let repo_name = string_prompt("Docs Repo Name".to_string(), None)?;
        let repo_url = string_prompt("Docs Repo Url".to_string(), None)?;

        Some(DocsInfo {
            site_name,
            site_description,
            site_url,
            locale,
            repo_name,
            repo_url,
        })
    } else {
        None
    };

    Ok(ProjectInfo {
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
        project_manager,
        pyo3_python_manager,
        is_application,
        is_async_project,
        github_actions_python_test_versions,
        max_line_length,
        use_dependabot,
        dependabot_schedule,
        dependabot_day,
        use_continuous_deployment,
        use_release_drafter,
        use_multi_os_ci,
        include_docs,
        docs_info,
        download_latest_packages: false,
        project_root_dir: None,

        #[cfg(feature = "fastapi")]
        is_fastapi_project,

        #[cfg(feature = "fastapi")]
        database_manager,
    })
}

fn github_actions_python_test_versions_prompt(default: Vec<String>) -> Result<Vec<String>> {
    let default_str = default.join(", ");
    let prompt = Prompt {
        prompt_text: "Python Versions for Github Actions Testing".to_string(),
        default: Some(default_str),
    };
    let input = prompt.show_prompt()?;
    let mut versions: Vec<String> = Vec::new();

    let version_check = input.replace(' ', "");

    for version in version_check.split(',') {
        if !is_valid_python_version(version) {
            bail!(format!("{} is not a valid Python Version", version));
        }

        versions.push(version.to_string());
    }

    Ok(versions)
}

fn license_prompt(default: Option<LicenseType>) -> Result<LicenseType> {
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
    let input = prompt.show_prompt()?;
    let license: LicenseType;

    if input == "1" || input.is_empty() {
        license = LicenseType::Mit;
    } else if input == "2" {
        license = LicenseType::Apache2;
    } else if input == "3" {
        license = LicenseType::NoLicense;
    } else {
        bail!("Invalid license type");
    }

    Ok(license)
}

fn max_line_length_prompt(default: Option<u8>) -> Result<u8> {
    let default_val = default.unwrap_or(100);
    let prompt = Prompt {
        prompt_text: "Max Line Length".to_string(),
        default: Some(default_val.to_string()),
    };
    let input = prompt.show_prompt()?;

    let max_line_length: u8 = match input.parse::<u8>() {
        Ok(m) => m,
        _ => {
            bail!(format!("{} is not a valid line length", input));
        }
    };

    Ok(max_line_length)
}

fn python_min_version_prompt(default: String) -> Result<String> {
    let prompt = Prompt {
        prompt_text: "Minimum Python Version".to_string(),
        default: Some(default),
    };
    let input = prompt.show_prompt()?;

    if !is_valid_python_version(&input) {
        bail!(format!("{} is not a valid Python Version", input.trim()));
    }

    Ok(input.to_string())
}

fn python_version_prompt(default: String) -> Result<String> {
    let prompt = Prompt {
        prompt_text: "Python Version".to_string(),
        default: Some(default),
    };
    let input = prompt.show_prompt()?;

    if !is_valid_python_version(&input) {
        bail!(format!("{} is not a valid Python Version", input.trim()));
    }

    Ok(input.to_string())
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
