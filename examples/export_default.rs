//! Export the default generated Citizen fixture for manual quality gates.

use std::path::Path;

use citizen_builder::generator::generate;
use citizen_builder::model::{
    AssetDefinition, AssetKind, CitizenProject, CitizenTemplate, DockPlacement, HostCitizen,
    PaletteItem,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let parent = arguments
        .next()
        .ok_or("usage: cargo run --example export_default -- <empty-parent-directory>")?;
    let mut project = match arguments.next().as_deref() {
        None => CitizenProject::default(),
        Some("--async") => {
            let mut project = CitizenProject::default();
            project.async_behavior.enabled = true;
            project.async_behavior.mappings[0].delay_ms = 25;
            project
        }
        Some("--template") => {
            let name = arguments.next().ok_or("--template requires a name")?;
            CitizenProject::from_template(parse_template(&name)?)
        }
        Some("--kitchen-sink") => kitchen_sink_project(),
        Some(option) => return Err(format!("unknown export option `{option}`").into()),
    };
    if arguments.next().is_some() {
        return Err("too many export arguments".into());
    }
    if project.async_behavior.enabled {
        for mapping in &mut project.async_behavior.mappings {
            mapping.delay_ms = 25;
        }
    }
    let generated = generate(&project).map_err(|diagnostics| {
        std::io::Error::other(format!("default fixture is invalid: {diagnostics:#?}"))
    })?;
    let destination = generated.write_new(Path::new(&parent))?;
    println!("{}", destination.display());
    Ok(())
}

fn parse_template(name: &str) -> Result<CitizenTemplate, Box<dyn std::error::Error>> {
    let template = match name {
        "settings" => CitizenTemplate::Settings,
        "logger" => CitizenTemplate::Logger,
        "editor" => CitizenTemplate::Editor,
        "plot" => CitizenTemplate::Plot,
        "file-browser" => CitizenTemplate::FileBrowser,
        "backend-control" => CitizenTemplate::BackendControl,
        _ => return Err(format!("unknown Citizen template `{name}`").into()),
    };
    Ok(template)
}

fn kitchen_sink_project() -> CitizenProject {
    let mut project = CitizenProject::default();
    let root = project.root.id;
    project.add_palette_item(Some(root), PaletteItem::StyledButton);
    project.add_palette_item(Some(root), PaletteItem::ReactiveLogger);
    project.add_palette_item(Some(root), PaletteItem::ReactiveEditor);
    project.add_palette_item(Some(root), PaletteItem::LinePlot);
    project.async_behavior.enabled = true;
    project.assets.push(AssetDefinition {
        file_name: "citizen-mark.svg".to_owned(),
        kind: AssetKind::Svg,
        contents: "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><circle cx=\"4\" cy=\"4\" r=\"3\"/></svg>\n".to_owned(),
    });
    project.composition.enabled = true;
    project.composition.external_citizens.push(HostCitizen {
        crate_name: "logger-citizen".to_owned(),
        citizen_type: "LoggerCitizen".to_owned(),
        citizen_id: "logger".to_owned(),
        title: "Logger".to_owned(),
        placement: DockPlacement::Right,
        fraction: 0.3,
    });
    project
}
