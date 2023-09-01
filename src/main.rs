mod cli;
mod file_manager;
mod github_actions;
mod licenses;
mod project_generator;
mod project_info;
mod python_files;
mod python_package_version;

use std::process::Command;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::Args;
use crate::project_generator::generate_project;
use crate::project_info::{get_project_info, ProjectInfo};

fn create(project_info: &ProjectInfo) -> Result<()> {
    generate_project(project_info);
    Command::new("git")
        .args(["init", &project_info.project_slug])
        .output()
        .expect("Failed to initialize git");

    Ok(())
}

fn main() {
    let args = Args::parse();
    let mut project_info = get_project_info();
    project_info.download_latest_packages = !args.skip_download_latest_packages;

    let create_result: Result<()>;
    if let Ok(progress_style) = ProgressStyle::with_template("{spinner:.green} {msg}") {
        let pb = ProgressBar::new_spinner();
        pb.enable_steady_tick(Duration::from_millis(80));
        pb.set_style(progress_style);
        pb.set_message("Generataing Project...");
        create_result = create(&project_info);
        pb.finish_and_clear();
    } else {
        create_result = create(&project_info);
    }

    match create_result {
        Ok(_) => {
            let success_message = format!(
                "\nProject created in the {} directory",
                project_info.project_slug
            );
            println!("{}", success_message.green());
        }
        Err(_) => {
            let error_message =
                "\nAn Error occurred creating the project. Please try again.".to_string();
            println!("{}", error_message.red());
        }
    };
}
