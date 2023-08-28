mod file_manager;
mod github_actions;
mod licenses;
mod project_generator;
mod project_info;
mod python_files;

use crate::project_generator::generate_project;
use crate::project_info::get_project_info;

fn main() {
    let project_info = get_project_info();
    generate_project(&project_info);
}
