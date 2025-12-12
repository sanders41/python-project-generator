use std::fmt;

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

pub fn default_pre_commit_rev(hook: &PreCommitHook) -> String {
    match hook {
        PreCommitHook::MyPy => "v1.18.2".to_string(),
        PreCommitHook::PreCommit => "v6.0.0".to_string(),
        PreCommitHook::Ruff => "v0.14.4".to_string(),
    }
}

pub fn pre_commit_repo(hook: &PreCommitHook) -> String {
    match hook {
        PreCommitHook::MyPy => "https://github.com/pre-commit/mirrors-mypy".to_string(),
        PreCommitHook::PreCommit => "https://github.com/pre-commit/pre-commit-hooks".to_string(),
        PreCommitHook::Ruff => "https://github.com/astral-sh/ruff-pre-commit".to_string(),
    }
}
