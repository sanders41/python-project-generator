mod cli;
mod file_manager;
mod github_actions;
mod licenses;
mod project_generator;
mod project_info;
mod python_files;
mod python_package_version;

use std::process::Command;

use clap::Parser;

use crate::cli::Args;
use crate::project_generator::generate_project;
use crate::project_info::get_project_info;

fn main() {
    let args = Args::parse();
    let mut project_info = get_project_info();
    project_info.download_latest_packages = !args.skip_download_latest_packages;

    generate_project(&project_info);
    Command::new("git")
        .args(["init", &project_info.project_slug])
        .output()
        .expect("Failed to initialize git");
}
