mod project_generator;
mod project_info;

use crate::project_generator::generate_project;
use crate::project_info::get_project_info;

fn main() {
    let project_info = get_project_info();
    generate_project(&project_info);
}
