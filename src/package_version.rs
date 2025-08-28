use std::{fmt, thread, time::Duration};

use anyhow::{bail, Result};
use exponential_backoff::Backoff;

#[derive(Debug, PartialEq, Eq)]
pub enum PythonPackage {
    Maturin,
    Mkdocs,
    MkdocsMaterial,
    Mkdocstrings,
    MyPy,
    PreCommit,
    Pytest,
    PytestAsyncio,
    PytestCov,
    Ruff,
    Tomli,
}

impl fmt::Display for PythonPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PythonPackage::Maturin => write!(f, "maturin"),
            PythonPackage::Mkdocs => write!(f, "mkdocs"),
            PythonPackage::MkdocsMaterial => write!(f, "mkdocs-material"),
            PythonPackage::Mkdocstrings => write!(f, "mkdocstrings"),
            PythonPackage::MyPy => write!(f, "mypy"),
            PythonPackage::PreCommit => write!(f, "pre-commit"),
            PythonPackage::Pytest => write!(f, "pytest"),
            PythonPackage::PytestAsyncio => write!(f, "pytest-asyncio"),
            PythonPackage::PytestCov => write!(f, "pytest-cov"),
            PythonPackage::Ruff => write!(f, "ruff"),
            PythonPackage::Tomli => write!(f, "tomli"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PreCommitHook {
    PreCommit,
    MyPy,
    Ruff,
}

impl fmt::Display for PreCommitHook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PreCommitHook::MyPy => write!(f, "mypy"),
            PreCommitHook::PreCommit => write!(f, "pre-commit"),
            PreCommitHook::Ruff => write!(f, "ruff"),
        }
    }
}

pub trait LatestVersion {
    fn get_latest_version(&mut self) -> Result<()>;
}

#[derive(Debug)]
pub struct PreCommitHookVersion {
    pub hook: PreCommitHook,
    pub repo: String,
    pub rev: String,
}

impl LatestVersion for PreCommitHookVersion {
    fn get_latest_version(&mut self) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let attempts = 3;
        let min = Duration::from_millis(100); // 10ms
        let max = Duration::from_secs(1);
        let backoff = Backoff::new(attempts, min, max);
        let api_url = format!(
            "{}/releases",
            self.repo
                .replace("https://github.com", "https://api.github.com/repos")
        );

        for duration in backoff {
            let response = client
                .get(&api_url)
                .header(reqwest::header::USER_AGENT, "python-project-generator")
                .timeout(Duration::new(5, 0))
                .send();

            match response {
                Ok(r) => {
                    let result = r.text()?;
                    let info: Vec<serde_json::Value> = serde_json::from_str(&result)?;
                    for i in info {
                        if i["draft"] == false && i["prerelease"] == false {
                            self.rev = i["tag_name"].to_string().replace('"', "");
                            break;
                        }
                    }

                    return Ok(());
                }
                Err(e) => match duration {
                    Some(duration) => thread::sleep(duration),
                    None => bail!("{e}"),
                },
            }
        }
        bail!("Error retrieving latest version");
    }
}

impl PreCommitHookVersion {
    pub fn new(hook: PreCommitHook) -> Self {
        let rev = default_pre_commit_rev(&hook);
        let repo = pre_commit_repo(&hook);
        PreCommitHookVersion { hook, repo, rev }
    }
}

#[derive(Debug)]
pub struct PythonPackageVersion {
    pub package: PythonPackage,
    pub version: String,
}

impl LatestVersion for PythonPackageVersion {
    fn get_latest_version(&mut self) -> Result<()> {
        self.version = get_latest_python_version(&self.package.to_string())?;

        Ok(())
    }
}

impl PythonPackageVersion {
    pub fn new(package: PythonPackage) -> Self {
        let version = default_version(&package);

        PythonPackageVersion { package, version }
    }
}

#[derive(Debug)]
pub struct RustPackageVersion {
    pub name: String,
    pub version: String,
    pub features: Option<Vec<String>>,
}

impl LatestVersion for RustPackageVersion {
    fn get_latest_version(&mut self) -> Result<()> {
        let url = format!("https://crates.io/api/v1/crates/{}", self.name);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(url)
            .header(reqwest::header::USER_AGENT, "python-project-generator")
            .timeout(Duration::new(5, 0))
            .send()?
            .text()?;
        let info: serde_json::Value = serde_json::from_str(&response)?;
        self.name = info["crate"]["id"].to_string().replace('"', "");
        self.version = info["crate"]["max_stable_version"]
            .to_string()
            .replace('"', "");

        Ok(())
    }
}

pub fn default_version(package: &PythonPackage) -> String {
    match package {
        PythonPackage::Maturin => "1.9.3".to_string(),
        PythonPackage::Mkdocs => "1.6.1".to_string(),
        PythonPackage::MkdocsMaterial => "9.6.18".to_string(),
        PythonPackage::Mkdocstrings => "0.30.0".to_string(),
        PythonPackage::MyPy => "1.17.1".to_string(),
        PythonPackage::PreCommit => "4.3.0".to_string(),
        PythonPackage::Pytest => "8.4.1".to_string(),
        PythonPackage::PytestAsyncio => "1.1.0".to_string(),
        PythonPackage::PytestCov => "6.2.1".to_string(),
        PythonPackage::Ruff => "0.12.11".to_string(),
        PythonPackage::Tomli => "2.2.1".to_string(),
    }
}

pub fn default_pre_commit_rev(hook: &PreCommitHook) -> String {
    match hook {
        PreCommitHook::MyPy => "v1.17.1".to_string(),
        PreCommitHook::PreCommit => "v6.0.0".to_string(),
        PreCommitHook::Ruff => "v0.12.11".to_string(),
    }
}

pub fn pre_commit_repo(hook: &PreCommitHook) -> String {
    match hook {
        PreCommitHook::MyPy => "https://github.com/pre-commit/mirrors-mypy".to_string(),
        PreCommitHook::PreCommit => "https://github.com/pre-commit/pre-commit-hooks".to_string(),
        PreCommitHook::Ruff => "https://github.com/astral-sh/ruff-pre-commit".to_string(),
    }
}

fn get_latest_python_version(name: &str) -> Result<String> {
    let url = format!("https://pypi.org/pypi/{name}/json");
    let client = reqwest::blocking::Client::new();
    let attempts = 3;
    let min = Duration::from_millis(100); // 10ms
    let max = Duration::from_secs(1);
    let backoff = Backoff::new(attempts, min, max);

    for duration in backoff {
        let response = client.get(&url).timeout(Duration::new(5, 0)).send();

        match response {
            Ok(r) => {
                let result = r.text()?;
                let info: serde_json::Value = serde_json::from_str(&result)?;
                return Ok(info["info"]["version"].to_string().replace('"', ""));
            }
            Err(e) => match duration {
                Some(duration) => thread::sleep(duration),
                None => bail!("{e}"),
            },
        }
    }
    bail!("Error retrieving latest version");
}
