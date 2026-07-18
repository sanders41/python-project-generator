use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum PythonPackage {
    Maturin,
    Mkdocs,
    MkdocsMaterial,
    Mkdocstrings,
    MyPy,
    Prek,
    Pyrefly,
    Pytest,
    PytestAsyncio,
    PytestCov,
    Ruff,
}

impl fmt::Display for PythonPackage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PythonPackage::Maturin => write!(f, "maturin"),
            PythonPackage::Mkdocs => write!(f, "mkdocs"),
            PythonPackage::MkdocsMaterial => write!(f, "mkdocs-material"),
            PythonPackage::Mkdocstrings => write!(f, "mkdocstrings"),
            PythonPackage::MyPy => write!(f, "mypy"),
            PythonPackage::Prek => write!(f, "prek"),
            PythonPackage::Pyrefly => write!(f, "pyrefly"),
            PythonPackage::Pytest => write!(f, "pytest"),
            PythonPackage::PytestAsyncio => write!(f, "pytest-asyncio"),
            PythonPackage::PytestCov => write!(f, "pytest-cov"),
            PythonPackage::Ruff => write!(f, "ruff"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum PrekHook {
    Builtin,
    PreCommit,
    MyPy,
    Pyrefly,
    Ruff,
}

impl fmt::Display for PrekHook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PrekHook::Builtin => write!(f, "builtin"),
            PrekHook::MyPy => write!(f, "mypy"),
            PrekHook::PreCommit => write!(f, "pre-commit"),
            PrekHook::Pyrefly => write!(f, "pyrefly"),
            PrekHook::Ruff => write!(f, "ruff"),
        }
    }
}

pub fn default_pre_commit_rev(hook: &PrekHook) -> Option<String> {
    match hook {
        PrekHook::Builtin => None,
        PrekHook::MyPy => Some("v1.18.2".to_string()),
        PrekHook::PreCommit => Some("v6.0.0".to_string()),
        PrekHook::Pyrefly => Some("1.1.1".to_string()),
        PrekHook::Ruff => Some("v0.14.4".to_string()),
    }
}

pub fn pre_commit_repo(hook: &PrekHook) -> String {
    match hook {
        PrekHook::Builtin => "builtin".to_string(),
        PrekHook::MyPy => "https://github.com/pre-commit/mirrors-mypy".to_string(),
        PrekHook::PreCommit => "https://github.com/pre-commit/pre-commit-hooks".to_string(),
        PrekHook::Pyrefly => "https://github.com/facebook/pyrefly-pre-commit".to_string(),
        PrekHook::Ruff => "https://github.com/astral-sh/ruff-pre-commit".to_string(),
    }
}
