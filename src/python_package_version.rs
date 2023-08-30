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
        let response = reqwest::blocking::get(url)?.text()?;
        let info: serde_json::Value = serde_json::from_str(&response)?;
        let name = info["info"]["name"].to_string().replace('"', "");
        let version = info["info"]["version"].to_string().replace('"', "");

        Ok(Self { name, version })
    }
}
