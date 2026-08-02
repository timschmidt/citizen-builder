//! Developer utility for materializing an all-node generated Citizen fixture.

use std::path::PathBuf;

use citizen_builder::generator::generate;
use citizen_builder::model::{CitizenProject, PaletteItem};

fn main() {
    let parent = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("usage: cargo run --example generate_fixture -- <parent-directory>");

    let mut project = CitizenProject::default();
    let root = project.root.id;

    let row = project.add_palette_item(Some(root), PaletteItem::Row);
    project.add_palette_item(Some(row), PaletteItem::Button);
    project.add_palette_item(Some(row), PaletteItem::Spacer);
    project.add_palette_item(Some(row), PaletteItem::Separator);

    let grid = project.add_palette_item(Some(root), PaletteItem::Grid);
    project.add_palette_item(Some(grid), PaletteItem::Checkbox);
    project.add_palette_item(Some(grid), PaletteItem::TextEdit);

    let group = project.add_palette_item(Some(root), PaletteItem::Group);
    let column = project.add_palette_item(Some(group), PaletteItem::Column);
    project.add_palette_item(Some(column), PaletteItem::Heading);
    project.add_palette_item(Some(column), PaletteItem::Label);

    let scroll = project.add_palette_item(Some(root), PaletteItem::Scroll);
    project.add_palette_item(Some(scroll), PaletteItem::Slider);
    project.add_palette_item(Some(scroll), PaletteItem::ProgressBar);

    let generated = generate(&project).expect("showcase fixture project is valid");
    let destination = generated
        .write_new(&parent)
        .expect("failed to export showcase Citizen fixture");
    println!("{}", destination.display());
}
