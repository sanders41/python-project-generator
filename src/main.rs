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

use clap::Parser;
use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::Args;
use crate::project_generator::generate_project;
use crate::project_info::get_project_info;

fn main() {
    let args = Args::parse();
    let mut project_info = get_project_info();

    let pb = ProgressBar::new_spinner();
    pb.enable_steady_tick(Duration::from_millis(80));
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message("Generataing Project...");

    project_info.download_latest_packages = !args.skip_download_latest_packages;

    generate_project(&project_info);
    Command::new("git")
        .args(["init", &project_info.project_slug])
        .output()
        .expect("Failed to initialize git");

    pb.finish_and_clear();
    let success_message = format!(
        "\nProject created in the {} directory",
        project_info.project_slug
    );
    println!("{}", success_message.green());
}
