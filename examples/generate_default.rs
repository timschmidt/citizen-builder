//! Developer utility for materializing the default generated Citizen fixture.

use std::path::PathBuf;

use citizen_builder::generator::generate;
use citizen_builder::model::CitizenProject;

fn main() {
    let parent = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --example generate_default -- <parent-directory>");
    let generated = generate(&CitizenProject::default()).expect("default project is valid");
    let destination = generated
        .write_new(&parent)
        .expect("failed to export default Citizen fixture");
    println!("{}", destination.display());
}
