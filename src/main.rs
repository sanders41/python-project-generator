mod project_info;

use crate::project_info::get_project_info;

fn main() {
    let project_info = get_project_info();
    println!("{:?}", &project_info);
}
