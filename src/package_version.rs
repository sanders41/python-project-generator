use anyhow::Result;

#[derive(Debug)]
pub enum PreCommitHook {
    Black,
    PreCommit,
    MyPy,
    Ruff,
}

pub trait LatestVersion {
    fn get_latest_version(&mut self) -> Result<()>;
}

#[derive(Debug)]
pub struct PreCommitHookVersion {
    pub id: PreCommitHook,
    pub repo: String,
    pub rev: String,
}

impl LatestVersion for PreCommitHookVersion {
    fn get_latest_version(&mut self) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        let api_url = format!(
            "{}/releases",
            self.repo
                .replace("https://github.com", "https://api.github.com/repos")
        );
        let response = client
            .get(api_url)
            .header(reqwest::header::USER_AGENT, "python-project-generator")
            .send()?
            .text()?;
        let info: Vec<serde_json::Value> = serde_json::from_str(&response)?;
        for i in info {
            if i["draft"] == false && i["prerelease"] == false {
                self.rev = i["tag_name"].to_string().replace('"', "");
                break;
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct PythonPackageVersion {
    pub name: String,
    pub version: String,
}

impl LatestVersion for PythonPackageVersion {
    fn get_latest_version(&mut self) -> Result<()> {
        let url = format!("https://pypi.org/pypi/{}/json", self.name);
        let response = reqwest::blocking::get(url)?.text()?;
        let info: serde_json::Value = serde_json::from_str(&response)?;
        self.name = info["info"]["name"].to_string().replace('"', "");
        self.version = info["info"]["version"].to_string().replace('"', "");

        Ok(())
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
        let response = reqwest::blocking::get(url)?.text()?;
        let info: serde_json::Value = serde_json::from_str(&response)?;
        self.name = info["crate"]["id"].to_string().replace('"', "");
        self.version = info["crate"]["max_stable_version"]
            .to_string()
            .replace('"', "");

        Ok(())
    }
}
