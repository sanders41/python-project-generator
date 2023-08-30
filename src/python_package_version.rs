use anyhow::Result;
use serde::Deserialize;

pub trait PypiPackage {
    fn get_latest_version(&self) -> Result<PythonPackageVersion>;
}

#[derive(Debug, Deserialize)]
pub struct PythonPackageVersion {
    pub name: String,
    pub version: String,
}

impl PypiPackage for PythonPackageVersion {
    fn get_latest_version(&self) -> Result<Self> {
        let url = format!("https://pypi.org/pypi/{}/json", self.name);
        let result = reqwest::blocking::get(url)?.json::<Self>()?;

        Ok(result)
    }
}
