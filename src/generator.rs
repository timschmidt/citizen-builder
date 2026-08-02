//! Deterministic standalone Citizen crate generation.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::model::{
    CitizenProject, DesignNode, Diagnostic, DiagnosticSeverity, FrameworkSource, NodeKind,
    StateValue,
};

/// A generated standalone crate represented as ordered relative paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedCrate {
    /// Directory name suggested for export.
    pub directory_name: String,
    /// Deterministically ordered UTF-8 files.
    pub files: BTreeMap<String, String>,
}

impl GeneratedCrate {
    /// Read one generated file by relative path.
    pub fn file(&self, path: &str) -> Option<&str> {
        self.files.get(path).map(String::as_str)
    }

    /// Export into a newly created child directory without overwriting data.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write_new(&self, parent: &Path) -> std::io::Result<PathBuf> {
        let destination = parent.join(&self.directory_name);
        std::fs::create_dir(&destination)?;
        for (relative, contents) in &self.files {
            let path = destination.join(relative);
            if let Some(directory) = path.parent() {
                std::fs::create_dir_all(directory)?;
            }
            std::fs::write(path, contents)?;
        }
        Ok(destination)
    }
}

/// Generate a complete Citizen crate when the design has no validation errors.
pub fn generate(project: &CitizenProject) -> Result<GeneratedCrate, Vec<Diagnostic>> {
    let diagnostics = project.validate();
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        return Err(diagnostics);
    }

    let preview_bin = format!("{}-preview", project.crate_name);
    let mut files = BTreeMap::new();
    files.insert(".gitignore".to_owned(), "/target\n/dist\n".to_owned());
    files.insert(
        "Cargo.toml".to_owned(),
        generate_manifest(project, &preview_bin),
    );
    files.insert(
        "README.md".to_owned(),
        generate_readme(project, &preview_bin),
    );
    files.insert("Trunk.toml".to_owned(), generate_trunk_config());
    files.insert(
        "citizen.json".to_owned(),
        project
            .to_json_pretty()
            .expect("validated model serializes"),
    );
    files.insert(
        "host-integration.md".to_owned(),
        generate_host_integration(project),
    );
    files.insert(
        "index.html".to_owned(),
        generate_index(project, &preview_bin),
    );
    files.insert(
        "src/lib.rs".to_owned(),
        format_rust(&generate_library(project)),
    );
    files.insert(
        "src/bin/preview.rs".to_owned(),
        format_rust(&generate_preview(project)),
    );

    Ok(GeneratedCrate {
        directory_name: project.crate_name.clone(),
        files,
    })
}

fn generate_manifest(project: &CitizenProject, preview_bin: &str) -> String {
    let description = toml_string(&project.description);
    let citizen_dependency = framework_dependency(&project.framework, "egui_citizen");
    let reactive_dependency = framework_dependency(&project.framework, "egui_mobius_reactive");

    format!(
        r#"[package]
name = {crate_name}
version = "0.1.0"
edition = "2024"
rust-version = "1.92"
description = {description}
license = "MIT"
publish = false

[features]
default = []
preview = [
    "dep:eframe",
    "dep:egui_dock",
    "dep:env_logger",
    "dep:log",
    "dep:wasm-bindgen-futures",
    "dep:web-sys",
]

[[bin]]
name = {preview_bin}
path = "src/bin/preview.rs"
required-features = ["preview"]

[dependencies]
egui = "=0.35.0"
egui_citizen = {citizen_dependency}
egui_mobius_reactive = {reactive_dependency}
eframe = {{ version = "=0.35.0", optional = true }}
egui_dock = {{ version = "=0.20.1", optional = true }}

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = {{ version = "0.11", optional = true }}

[target.'cfg(target_arch = "wasm32")'.dependencies]
log = {{ version = "0.4", optional = true }}
wasm-bindgen-futures = {{ version = "0.4", optional = true }}
web-sys = {{ version = "0.3", features = ["Document", "Element", "HtmlCanvasElement", "Window"], optional = true }}
"#,
        crate_name = toml_string(&project.crate_name),
        preview_bin = toml_string(preview_bin),
    )
}

fn framework_dependency(source: &FrameworkSource, package: &str) -> String {
    match source {
        FrameworkSource::Git {
            repository,
            revision,
        } => format!(
            "{{ git = {}, rev = {} }}",
            toml_string(repository),
            toml_string(revision)
        ),
        FrameworkSource::Path { workspace } => {
            let path = Path::new(workspace).join("crates").join(package);
            format!("{{ path = {} }}", toml_string(&path.to_string_lossy()))
        }
    }
}

fn generate_library(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let mut writer = RustWriter::default();
    writer.line("//! Generated Citizen library.");
    for line in project.description.lines() {
        writer.line(&format!("//! {}", line.trim()));
    }
    writer.blank();
    writer.line("use egui_citizen::{Citizen, CitizenId, CitizenState};");
    writer.line("use egui_mobius_reactive::Dynamic;");
    writer.blank();
    writer.line("/// Typed reactive state shared with this Citizen.");
    writer.line("#[derive(Clone)]");
    writer.open(&format!("pub struct {state_type}"));
    for field in &project.state_fields {
        let ty = match field.value {
            StateValue::Bool(_) => "bool",
            StateValue::Text(_) => "String",
            StateValue::Number(_) => "f32",
        };
        writer.line(&format!("pub {}: Dynamic<{ty}>,", field.name));
    }
    writer.close();
    writer.blank();
    writer.open(&format!("impl Default for {state_type}"));
    writer.open("fn default() -> Self");
    writer.open("Self");
    for field in &project.state_fields {
        let default = match &field.value {
            StateValue::Bool(value) => format!("Dynamic::new({value})"),
            StateValue::Text(value) => {
                format!("Dynamic::new({}.to_owned())", rust_string(value))
            }
            StateValue::Number(value) => {
                format!("Dynamic::new({}_f32)", format_f32(*value))
            }
        };
        writer.line(&format!("{}: {default},", field.name));
    }
    writer.close();
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Reusable egui_mobius Citizen panel.");
    writer.open(&format!("pub struct {}", project.citizen_type));
    writer.line("citizen_id: CitizenId,");
    writer.line("citizen_state: CitizenState,");
    writer.line(&format!(
        "/// Reactive state contract consumed by this Citizen.\npub state: {state_type},"
    ));
    writer.close();
    writer.blank();
    writer.open(&format!("impl {}", project.citizen_type));
    writer.line(&format!(
        "/// Stable Dispatcher identity.\npub const ID: &'static str = {};",
        rust_string(&project.citizen_id)
    ));
    writer.line(&format!(
        "/// Default dock-tab title.\npub const TITLE: &'static str = {};",
        rust_string(&project.title)
    ));
    writer.blank();
    writer.line("/// Construct the Citizen from registered lifecycle state and shared data.");
    writer.open(&format!(
        "pub fn new(citizen_state: CitizenState, state: {state_type}) -> Self"
    ));
    writer.open("Self");
    writer.line("citizen_id: CitizenId::new(Self::ID),");
    writer.line("citizen_state,");
    writer.line("state,");
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Render this Citizen inside its host-provided panel UI.");
    writer.open("pub fn show(&mut self, ui: &mut egui::Ui)");
    emit_node(&project.root, &mut writer);
    writer.close();
    writer.close();
    writer.blank();
    writer.open(&format!("impl Citizen for {}", project.citizen_type));
    writer.open("fn id(&self) -> &CitizenId");
    writer.line("&self.citizen_id");
    writer.close();
    writer.blank();
    writer.open("fn citizen_state(&self) -> &CitizenState");
    writer.line("&self.citizen_state");
    writer.close();
    writer.blank();
    writer.open("fn citizen_state_mut(&mut self) -> &mut CitizenState");
    writer.line("&mut self.citizen_state");
    writer.close();
    writer.close();
    writer.finish()
}

fn emit_node(node: &DesignNode, writer: &mut RustWriter) {
    writer.line(&format!("// {} ({})", node.name, node.kind.display_name()));
    match &node.kind {
        NodeKind::Column => {
            writer.open("ui.vertical(|ui|");
            emit_children(node, writer);
            writer.close_call();
        }
        NodeKind::Row { wrap } => {
            let method = if *wrap {
                "horizontal_wrapped"
            } else {
                "horizontal"
            };
            writer.open(&format!("ui.{method}(|ui|"));
            emit_children(node, writer);
            writer.close_call();
        }
        NodeKind::Grid { columns, striped } => {
            writer.line(&format!(
                "let layout = egui::Grid::new({});",
                rust_string(&node.name)
            ));
            writer.line(&format!("let layout = layout.num_columns({columns});"));
            writer.line(&format!("let layout = layout.striped({striped});"));
            writer.open("layout.show(ui, |ui|");
            for (index, child) in node.children.iter().enumerate() {
                emit_node(child, writer);
                if (index + 1) % columns == 0 {
                    writer.line("ui.end_row();");
                }
            }
            writer.close_call();
        }
        NodeKind::Group { title } => {
            writer.open("ui.group(|ui|");
            if !title.is_empty() {
                writer.line(&format!("ui.strong({});", rust_string(title)));
                writer.line("ui.separator();");
            }
            emit_children(node, writer);
            writer.close_call();
        }
        NodeKind::Scroll { max_height } => {
            writer.line("let layout = egui::ScrollArea::vertical();");
            writer.line(&format!(
                "let layout = layout.id_salt({});",
                rust_string(&node.name)
            ));
            if *max_height > 0.0 {
                writer.line(&format!(
                    "let layout = layout.max_height({}_f32);",
                    format_f32(*max_height)
                ));
            }
            writer.open("layout.show(ui, |ui|");
            emit_children(node, writer);
            writer.close_call();
        }
        NodeKind::Label { text } => writer.line(&format!("ui.label({});", rust_string(text))),
        NodeKind::Heading { text } => {
            writer.line(&format!("ui.heading({});", rust_string(text)));
        }
        NodeKind::Button { text } => {
            writer.line(&format!(
                "let _response = ui.button({});",
                rust_string(text)
            ));
        }
        NodeKind::Checkbox { text, binding } => {
            let binding = binding.as_deref().expect("validated binding");
            writer.open("");
            writer.line(&format!("let mut value = self.state.{binding}.get();"));
            writer.line(&format!(
                "if ui.checkbox(&mut value, {}).changed() {{",
                rust_string(text)
            ));
            writer.indent += 1;
            writer.line(&format!("self.state.{binding}.set(value);"));
            writer.indent -= 1;
            writer.line("}");
            writer.close();
        }
        NodeKind::TextEdit {
            label,
            hint,
            binding,
        } => {
            let binding = binding.as_deref().expect("validated binding");
            writer.open("");
            writer.line(&format!("let mut value = self.state.{binding}.get();"));
            writer.open("ui.horizontal(|ui|");
            if !label.is_empty() {
                writer.line(&format!("ui.label({});", rust_string(label)));
            }
            writer.line(&format!(
                "let editor = egui::TextEdit::singleline(&mut value);\nlet editor = editor.hint_text({});\nlet response = ui.add(editor);",
                rust_string(hint)
            ));
            writer.line("if response.changed() {");
            writer.indent += 1;
            writer.line(&format!("self.state.{binding}.set(value);"));
            writer.indent -= 1;
            writer.line("}");
            writer.close_call();
            writer.close();
        }
        NodeKind::Slider {
            label,
            min,
            max,
            binding,
        } => {
            let binding = binding.as_deref().expect("validated binding");
            writer.open("");
            writer.line(&format!("let mut value = self.state.{binding}.get();"));
            writer.line(&format!(
                "let range = {}_f32..={}_f32;\nlet slider = egui::Slider::new(&mut value, range);\nlet response = ui.add(slider.text({}));",
                format_f32(*min),
                format_f32(*max),
                rust_string(label)
            ));
            writer.line("if response.changed() {");
            writer.indent += 1;
            writer.line(&format!("self.state.{binding}.set(value);"));
            writer.indent -= 1;
            writer.line("}");
            writer.close();
        }
        NodeKind::ProgressBar {
            binding,
            show_percentage,
        } => {
            let binding = binding.as_deref().expect("validated binding");
            writer.open("");
            writer.line(&format!(
                "let value = self.state.{binding}.get().clamp(0.0_f32, 1.0_f32);"
            ));
            let suffix = if *show_percentage {
                ".show_percentage()"
            } else {
                ""
            };
            writer.line(&format!("ui.add(egui::ProgressBar::new(value){suffix});"));
            writer.close();
        }
        NodeKind::Separator => writer.line("ui.separator();"),
        NodeKind::Spacer { points } => {
            writer.line(&format!("ui.add_space({}_f32);", format_f32(*points)));
        }
    }
}

fn emit_children(node: &DesignNode, writer: &mut RustWriter) {
    for child in &node.children {
        emit_node(child, writer);
    }
}

fn generate_preview(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let mut source = format!(
        r#"//! Native and WASM preview host generated by citizen-builder.

use eframe::egui;
use egui_citizen::{{CitizenId, Dispatcher}};
use egui_dock::{{DockArea, DockState}};
use {crate_ident}::{{{citizen_type}, {state_type}}};

#[derive(Clone)]
struct PreviewTab;

struct PreviewViewer<'a> {{
    dispatcher: &'a mut Dispatcher,
    citizen: &'a mut {citizen_type},
}}

impl egui_dock::TabViewer for PreviewViewer<'_> {{
    type Tab = PreviewTab;

    fn title(&mut self, _tab: &mut Self::Tab) -> egui::WidgetText {{
        {citizen_type}::TITLE.into()
    }}

    fn ui(&mut self, ui: &mut egui::Ui, _tab: &mut Self::Tab) {{
        self.citizen.show(ui);
    }}

    fn on_tab_button(&mut self, _tab: &mut Self::Tab, response: &egui::Response) {{
        if response.clicked() {{
            let citizen_id = CitizenId::new({citizen_type}::ID);
            self.dispatcher.activate(&citizen_id);
        }}
    }}
}}

struct PreviewApp {{
    dispatcher: Dispatcher,
    dock_state: DockState<PreviewTab>,
    citizen: {citizen_type},
}}

impl PreviewApp {{
    fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {{
        let mut dispatcher = Dispatcher::new();
        let lifecycle = dispatcher.register(CitizenId::new({citizen_type}::ID));
        dispatcher.activate(&CitizenId::new({citizen_type}::ID));
        let _ = dispatcher.drain_messages();
        Self {{
            dispatcher,
            dock_state: DockState::new(vec![PreviewTab]),
            citizen: {citizen_type}::new(lifecycle, {state_type}::default()),
        }}
    }}
}}

impl eframe::App for PreviewApp {{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {{
        let mut viewer = PreviewViewer {{
            dispatcher: &mut self.dispatcher,
            citizen: &mut self.citizen,
        }};
        DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        let _messages = self.dispatcher.drain_messages();
    }}
}}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {{
    env_logger::init();
    eframe::run_native(
        {title},
        eframe::NativeOptions {{
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 640.0])
                .with_min_inner_size([560.0, 360.0]),
            ..Default::default()
        }},
        Box::new(|cc| Ok(Box::new(PreviewApp::new(cc)))),
    )
}}

#[cfg(target_arch = "wasm32")]
fn main() {{
    use eframe::wasm_bindgen::JsCast as _;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    wasm_bindgen_futures::spawn_local(async {{
        let document = web_sys::window()
            .expect("window is unavailable")
            .document()
            .expect("document is unavailable");
        let canvas = document
            .get_element_by_id("citizen_canvas")
            .expect("citizen_canvas is missing")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("citizen_canvas is not a canvas");
        eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|cc| Ok(Box::new(PreviewApp::new(cc)))),
            )
            .await
            .expect("failed to start Citizen preview");
        if let Some(loading) = document.get_element_by_id("loading") {{
            loading.remove();
        }}
    }});
}}
"#,
        crate_ident = project.crate_ident(),
        citizen_type = project.citizen_type,
        title = rust_string(&format!("{} Preview", project.title)),
    );
    if !source.ends_with('\n') {
        source.push('\n');
    }
    source
}

fn generate_readme(project: &CitizenProject, preview_bin: &str) -> String {
    format!(
        "# {}\n\n{}\n\nThis crate contains one reusable egui_mobius Citizen generated by [citizen-builder](https://github.com/timschmidt/citizen-builder).\n\n## Native preview\n\n```shell\ncargo run --features preview --bin {}\n```\n\n## WASM preview\n\n```shell\nrustup target add wasm32-unknown-unknown\ncargo install trunk\ntrunk serve --open\n```\n\nSee [`host-integration.md`](host-integration.md) for the Dispatcher, `TabKind`, and `TabViewer` integration points.\n",
        project.title, project.description, preview_bin
    )
}

fn generate_host_integration(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let state_field = format!("{}_state", project.citizen_id);
    let citizen_type = &project.citizen_type;
    let citizen_id = &project.citizen_id;
    format!(
        "# Host integration\n\n`{citizen_type}` is a compile-time Citizen plug-in. Add this crate to the host, then make these four integration edits.\n\n1. Add fields to the host:\n\n```rust\n{citizen_id}: {citizen_type},\n{state_field}: {state_type},\n```\n\n2. Register and construct it during startup:\n\n```rust\nlet lifecycle = dispatcher.register(egui_citizen::CitizenId::new({citizen_type}::ID));\nlet {state_field} = {state_type}::default();\nlet {citizen_id} = {citizen_type}::new(lifecycle, {state_field}.clone());\n```\n\n3. Add a `TabKind` variant and render arm:\n\n```rust\nTabKind::{citizen_type} => self.{citizen_id}.show(ui),\n```\n\n4. In `TabViewer::on_tab_button`, call `dispatcher.activate(...)`, then drain the Dispatcher once after the dock renders.\n\nThe generated `src/bin/preview.rs` is a complete executable reference.\n",
    )
}

fn generate_trunk_config() -> String {
    "[build]\ntarget = \"index.html\"\nrelease = false\nfilehash = true\n\n[serve]\naddresses = [\"127.0.0.1\"]\nport = 8080\nopen = true\n\n[tools]\nwasm_opt = \"z\"\n"
        .to_owned()
}

fn generate_index(project: &CitizenProject, preview_bin: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no">
  <title>{title} Preview</title>
  <link data-trunk rel="rust" data-bin="{preview_bin}" data-cargo-features="preview">
  <style>
    html, body {{ margin: 0; width: 100%; height: 100%; overflow: hidden; background: #16181d; }}
    #citizen_canvas {{ width: 100%; height: 100%; }}
    #loading {{ position: fixed; inset: 0; display: grid; place-items: center; color: #d7dce5; font: 16px system-ui; }}
  </style>
</head>
<body>
  <div id="loading">Loading {title}…</div>
  <canvas id="citizen_canvas"></canvas>
</body>
</html>
"#,
        title = html_escape(&project.title),
        preview_bin = html_escape(preview_bin),
    )
}

fn rust_string(value: &str) -> String {
    format!("{value:?}")
}

fn toml_string(value: &str) -> String {
    serde_json::to_string(value).expect("strings always serialize")
}

fn format_f32(value: f32) -> String {
    let mut formatted = value.to_string();
    if !formatted.contains(['.', 'e', 'E']) {
        formatted.push_str(".0");
    }
    formatted
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn format_rust(source: &str) -> String {
    syn::parse_file(source).expect("validated generation must produce Rust syntax");
    source.to_owned()
}

#[derive(Default)]
struct RustWriter {
    output: String,
    indent: usize,
}

impl RustWriter {
    fn line(&mut self, line: &str) {
        for part in line.lines() {
            let _ = writeln!(self.output, "{}{part}", "    ".repeat(self.indent));
        }
    }

    fn blank(&mut self) {
        self.output.push('\n');
    }

    fn open(&mut self, header: &str) {
        if header.is_empty() {
            self.line("{");
        } else {
            self.line(&format!("{header} {{"));
        }
        self.indent += 1;
    }

    fn close(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.line("}");
    }

    fn close_call(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.line("});");
    }

    fn finish(self) -> String {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{FrameworkSource, StateField};

    #[test]
    fn default_project_generates_complete_crate() {
        let generated = generate(&CitizenProject::default()).unwrap();
        for required in [
            "Cargo.toml",
            "README.md",
            "Trunk.toml",
            "citizen.json",
            "host-integration.md",
            "index.html",
            "src/lib.rs",
            "src/bin/preview.rs",
        ] {
            assert!(generated.file(required).is_some(), "missing {required}");
        }
    }

    #[test]
    fn git_dependencies_are_pinned_exactly() {
        let project = CitizenProject::default();
        let manifest = generate(&project)
            .unwrap()
            .file("Cargo.toml")
            .unwrap()
            .to_owned();
        assert!(manifest.contains(crate::model::DEFAULT_FRAMEWORK_REVISION));
        assert!(manifest.contains("egui_citizen = { git ="));
    }

    #[test]
    fn path_dependencies_point_into_framework_crates() {
        let project = CitizenProject {
            framework: FrameworkSource::Path {
                workspace: "../egui_mobius".to_owned(),
            },
            ..CitizenProject::default()
        };
        let manifest = generate(&project)
            .unwrap()
            .file("Cargo.toml")
            .unwrap()
            .to_owned();
        assert!(manifest.contains("../egui_mobius/crates/egui_citizen"));
        assert!(manifest.contains("../egui_mobius/crates/egui_mobius_reactive"));
    }

    #[test]
    fn library_implements_citizen_and_dynamic_state() {
        let library = generate(&CitizenProject::default())
            .unwrap()
            .file("src/lib.rs")
            .unwrap()
            .to_owned();
        assert!(library.contains("impl Citizen for ExampleCitizen"));
        assert!(library.contains("pub enabled: Dynamic<bool>"));
        assert!(library.contains("pub display_name: Dynamic<String>"));
        assert!(library.contains("pub level: Dynamic<f32>"));
    }

    #[test]
    fn preview_contains_native_and_wasm_entrypoints() {
        let preview = generate(&CitizenProject::default())
            .unwrap()
            .file("src/bin/preview.rs")
            .unwrap()
            .to_owned();
        assert!(preview.contains("cfg(not(target_arch = \"wasm32\"))"));
        assert!(preview.contains("cfg(target_arch = \"wasm32\")"));
        assert!(preview.contains("Dispatcher::new()"));
        assert!(preview.contains("DockArea::new"));
    }

    #[test]
    fn generation_is_deterministic() {
        let project = CitizenProject::default();
        assert_eq!(generate(&project).unwrap(), generate(&project).unwrap());
    }

    #[test]
    fn invalid_design_does_not_generate() {
        let project = CitizenProject {
            crate_name: "Not Valid".to_owned(),
            ..CitizenProject::default()
        };
        assert!(generate(&project).is_err());
    }

    #[test]
    fn all_state_defaults_emit_valid_dynamic_constructors() {
        let mut project = CitizenProject::default();
        project.state_fields.push(StateField {
            name: "count".to_owned(),
            value: StateValue::Number(2.0),
        });
        let library = generate(&project)
            .unwrap()
            .file("src/lib.rs")
            .unwrap()
            .to_owned();
        assert!(library.contains("Dynamic::new(2.0_f32)"));
    }
}
