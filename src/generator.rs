//! Deterministic standalone Citizen crate generation.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use crate::model::{
    CitizenProject, DesignNode, Diagnostic, DiagnosticSeverity, DockPlacement, FrameworkSource,
    InteractionEvent, MessageKey, MessageRole, NodeKind, StateValue, ThemePreset,
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
        "src/theme.rs".to_owned(),
        format_rust(&generate_theme(project)),
    );
    if !project.assets.is_empty() {
        files.insert(
            "src/assets.rs".to_owned(),
            format_rust(&generate_assets_module(project)),
        );
        for asset in &project.assets {
            files.insert(
                format!("assets/{}", asset.file_name),
                asset.contents.clone(),
            );
        }
    }
    if project.composition.enabled {
        files.insert(
            "host-composition.md".to_owned(),
            generate_host_composition(project),
        );
    }
    files.insert(
        "src/lib.rs".to_owned(),
        format_rust(&generate_library(project)),
    );
    files.insert(
        "src/messages.rs".to_owned(),
        format_rust(&generate_messages(project)),
    );
    files.insert(
        "src/backend.rs".to_owned(),
        format_rust(&generate_backend(project)),
    );
    if project.async_behavior.enabled {
        files.insert(
            "src/async_backend.rs".to_owned(),
            format_rust(&generate_async_backend(project)),
        );
    }
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
    let lens_dependency = framework_dependency(&project.framework, "egui_lens");
    let inferred_features = project.inferred_features();
    let component_features = [
        "component-lens",
        "component-plot",
        "component-quill",
        "component-widgets",
    ]
    .into_iter()
    .filter(|feature| inferred_features.contains(*feature))
    .collect::<Vec<_>>();
    let default_features = if component_features.is_empty() {
        "default = []".to_owned()
    } else {
        format!(
            "default = [{}]",
            component_features
                .iter()
                .map(|feature| toml_string(feature))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let mut component_feature_definitions = String::new();
    let mut component_dependencies = String::new();
    let mut native_component_dependencies = String::new();
    if inferred_features.contains("component-lens") {
        component_feature_definitions.push_str("component-lens = [\"dep:egui_lens\"]\n");
    }
    if inferred_features.contains("component-plot") {
        component_feature_definitions.push_str("component-plot = [\"dep:egui_plot\"]\n");
        component_dependencies.push_str("egui_plot = { version = \"=0.36.0\", optional = true }\n");
    }
    if inferred_features.contains("component-quill") {
        component_feature_definitions.push_str("component-quill = [\"dep:egui_quill\"]\n");
        let dependency = framework_dependency(&project.framework, "egui_quill");
        component_dependencies.push_str(&format!(
            "egui_quill = {}\n",
            optional_dependency(&dependency)
        ));
    }
    if inferred_features.contains("component-widgets") {
        component_feature_definitions
            .push_str("component-widgets = [\"dep:egui_mobius_widgets\"]\n");
        let dependency = framework_dependency(&project.framework, "egui_mobius_widgets");
        native_component_dependencies.push_str(&format!(
            "egui_mobius_widgets = {}\n",
            optional_dependency(&dependency)
        ));
    }
    let async_feature = if project.async_behavior.enabled {
        "async-backend = [\n    \"dep:egui_mobius\",\n    \"dep:tokio\",\n    \"dep:futures\",\n    \"dep:gloo-timers\",\n    \"dep:wasm-bindgen-futures\",\n]\n"
    } else {
        ""
    };
    let preview_async = if project.async_behavior.enabled {
        "    \"async-backend\",\n"
    } else {
        ""
    };
    let native_async_dependencies = if project.async_behavior.enabled {
        let dependency = framework_dependency(&project.framework, "egui_mobius");
        format!(
            "egui_mobius = {}\ntokio = {{ version = \"1.52.1\", features = [\"time\"], optional = true }}\n",
            optional_dependency(&dependency)
        )
    } else {
        String::new()
    };
    let wasm_async_dependencies = if project.async_behavior.enabled {
        "futures = { version = \"0.3\", optional = true }\ngloo-timers = { version = \"0.3\", features = [\"futures\"], optional = true }\n"
    } else {
        ""
    };

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
{default_features}
{async_feature}{component_feature_definitions}preview = [
    "dep:eframe",
    "dep:egui_dock",
    "dep:env_logger",
    "dep:log",
    "dep:wasm-bindgen-futures",
    "dep:web-sys",
{preview_async}]
lens = ["preview", "dep:egui_lens"]

[[bin]]
name = {preview_bin}
path = "src/bin/preview.rs"
required-features = ["preview"]

[dependencies]
egui = "=0.35.0"
egui_citizen = {citizen_dependency}
egui_mobius_reactive = {reactive_dependency}
egui_lens = {lens_dependency_optional}
{component_dependencies}eframe = {{ version = "=0.35.0", optional = true }}
egui_dock = {{ version = "=0.20.1", optional = true }}

[target.'cfg(not(target_arch = "wasm32"))'.dependencies]
env_logger = {{ version = "0.11", optional = true }}
{native_component_dependencies}{native_async_dependencies}

[target.'cfg(target_arch = "wasm32")'.dependencies]
{wasm_async_dependencies}log = {{ version = "0.4", optional = true }}
wasm-bindgen-futures = {{ version = "0.4", optional = true }}
web-sys = {{ version = "0.3", features = ["Document", "Element", "HtmlCanvasElement", "Window"], optional = true }}
"#,
        crate_name = toml_string(&project.crate_name),
        preview_bin = toml_string(preview_bin),
        lens_dependency_optional = optional_dependency(&lens_dependency),
        default_features = default_features,
        component_feature_definitions = component_feature_definitions,
        component_dependencies = component_dependencies,
        native_component_dependencies = native_component_dependencies,
        async_feature = async_feature,
        preview_async = preview_async,
        native_async_dependencies = native_async_dependencies,
        wasm_async_dependencies = wasm_async_dependencies,
    )
}

fn optional_dependency(dependency: &str) -> String {
    let body = dependency
        .strip_prefix("{ ")
        .and_then(|value| value.strip_suffix(" }"))
        .expect("framework dependency is an inline table");
    format!("{{ {body}, optional = true }}")
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

fn generate_theme(project: &CitizenProject) -> String {
    let mut writer = RustWriter::default();
    writer.line("//! Generated egui theme shared by preview and host applications.");
    writer.blank();
    writer.line("/// Apply this Citizen project's visual theme to an egui context.");
    writer.open("pub fn apply(context: &egui::Context)");
    let preset = match project.theme.preset {
        ThemePreset::Dark => "Dark",
        ThemePreset::Light => "Light",
    };
    writer.line(&format!("let theme = egui::Theme::{preset};"));
    writer.line("context.set_theme(theme);");
    writer.line(&format!(
        "let mut visuals = egui::Visuals::{}();",
        preset.to_ascii_lowercase()
    ));
    writer.line(&format!(
        "visuals.selection.bg_fill = egui::Color32::from_rgb({}, {}, {});",
        project.theme.accent_rgb[0], project.theme.accent_rgb[1], project.theme.accent_rgb[2]
    ));
    writer.line(&format!(
        "visuals.panel_fill = egui::Color32::from_rgb({}, {}, {});",
        project.theme.panel_rgb[0], project.theme.panel_rgb[1], project.theme.panel_rgb[2]
    ));
    writer.line("context.set_visuals(visuals);");
    writer.open("context.style_mut_of(theme, |style|");
    writer.line(&format!(
        "style.spacing.item_spacing = egui::vec2({0}_f32, {0}_f32);",
        format_f32(project.theme.item_spacing)
    ));
    writer.close_call();
    writer.close();
    writer.finish()
}

fn generate_assets_module(project: &CitizenProject) -> String {
    let mut writer = RustWriter::default();
    writer.line("//! UTF-8 assets embedded from the Citizen project.");
    for asset in &project.assets {
        writer.blank();
        writer.line(&format!("/// Embedded `{}` asset.", asset.file_name));
        writer.line(&format!(
            "pub const {}: &str = {{",
            asset_identifier(&asset.file_name)
        ));
        writer.indent += 1;
        writer.line("// Keep the source asset visible to Cargo's dependency tracker.");
        writer.line(&format!(
            "include_str!({})",
            rust_string(&format!("../assets/{}", asset.file_name))
        ));
        writer.indent -= 1;
        writer.line("};");
    }
    writer.finish()
}

fn asset_identifier(file_name: &str) -> String {
    file_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn generate_messages(project: &CitizenProject) -> String {
    let mut domains = BTreeMap::<String, Vec<_>>::new();
    for message in &project.messages {
        domains
            .entry(message.key.domain.clone())
            .or_default()
            .push(message);
    }

    let mut writer = RustWriter::default();
    writer.line("//! Domain-grouped application intents and outcomes.");
    writer.blank();
    writer.line("/// Every discrete message crossing the Citizen/host boundary.");
    writer.line("#[derive(Clone, Debug, PartialEq, Eq)]");
    writer.open("pub enum AppMessage");
    for domain in domains.keys() {
        let variant = pascal_case(domain);
        writer.line(&format!("{variant}({variant}Message),"));
    }
    writer.close();
    writer.blank();
    writer.open("impl AppMessage");
    writer.line("/// Stable diagnostic name used by reference-host logging.");
    writer.open("pub const fn name(&self) -> &'static str");
    writer.open("match self");
    for (domain, messages) in &domains {
        let domain_variant = pascal_case(domain);
        for message in messages {
            writer.line(&format!(
                "Self::{domain_variant}({domain_variant}Message::{variant}) => {name},",
                variant = message.key.variant,
                name = rust_string(&message.key.display_name()),
            ));
        }
    }
    writer.close();
    writer.close();
    writer.close();

    for (domain, messages) in domains {
        let domain_type = format!("{}Message", pascal_case(&domain));
        writer.blank();
        writer.line(&format!(
            "/// Messages in the `{domain}` application domain."
        ));
        writer.line("#[derive(Clone, Debug, PartialEq, Eq)]");
        writer.open(&format!("pub enum {domain_type}"));
        for message in messages {
            for line in message.description.lines() {
                writer.line(&format!("/// {}", line.trim()));
            }
            writer.line(&format!("{},", message.key.variant));
        }
        writer.close();
    }
    writer.finish()
}

fn generate_backend(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let mut writer = RustWriter::default();
    writer.line("//! Synchronous reference backend and UI-thread outcome application.");
    writer.blank();
    writer.line(&format!("use crate::{state_type};"));
    writer.line("use crate::messages::AppMessage;");
    writer.blank();
    writer.line("/// Minimal host-owned backend used by the generated preview.");
    writer.line("#[derive(Default)]");
    writer.line("pub struct ReferenceBackend;");
    writer.blank();
    writer.open("impl ReferenceBackend");
    writer.line("/// Route one intent and return the corresponding outcome.");
    writer.open(&format!(
        "pub fn handle(\n    &mut self,\n    message: &AppMessage,\n    _state: &{state_type},\n) -> Option<AppMessage>"
    ));
    writer.open("match message");
    for message in project
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Intent)
    {
        if let Some(outcome) = &message.paired_outcome {
            writer.open(&format!("{} =>", message_pattern(&message.key)));
            emit_let_message(&mut writer, "outcome", outcome);
            writer.line("Some(outcome)");
            writer.close();
        }
    }
    writer.line("_ => None,");
    writer.close();
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Apply an outcome to reactive state on the UI thread.");
    writer.open(&format!(
        "pub fn apply_outcome(message: &AppMessage, state: &{state_type})"
    ));
    let mut emitted_update = false;
    for message in project
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Outcome && !message.state_updates.is_empty())
    {
        emitted_update = true;
        writer.open(&format!(
            "if let {} = message",
            message_pattern(&message.key)
        ));
        for update in &message.state_updates {
            let value = match &update.value {
                StateValue::Bool(value) => value.to_string(),
                StateValue::Text(value) => format!("{}.to_owned()", rust_string(value)),
                StateValue::Number(value) => format!("{}_f32", format_f32(*value)),
            };
            writer.line(&format!("state.{}.set({value});", update.field));
        }
        writer.close();
    }
    if !emitted_update {
        writer.line("let _ = (message, state);");
    }
    writer.close();

    let paired = project
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Intent)
        .filter_map(|message| {
            message
                .paired_outcome
                .as_ref()
                .map(|outcome| (&message.key, outcome))
        })
        .collect::<Vec<_>>();
    if !paired.is_empty() {
        writer.blank();
        writer.line("#[cfg(test)]");
        writer.open("mod tests");
        writer.line("use super::*;");
        writer.blank();
        writer.line("#[test]");
        writer.open("fn reference_backend_routes_intents_and_applies_outcomes()");
        writer.line(&format!("let state = {state_type}::default();"));
        writer.line("let mut backend = ReferenceBackend;");
        for (index, (intent, outcome)) in paired.iter().enumerate() {
            let intent_name = format!("intent_{index}");
            let outcome_name = format!("outcome_{index}");
            emit_let_message(&mut writer, &intent_name, intent);
            writer.line(&format!("let {outcome_name} = backend"));
            writer.indent += 1;
            writer.line(&format!(".handle(&{intent_name}, &state)"));
            writer.line(".expect(\"paired intent must produce an outcome\");");
            writer.indent -= 1;
            emit_let_message(&mut writer, "expected", outcome);
            writer.line(&format!("assert_eq!({outcome_name}, expected);"));
            writer.line(&format!("apply_outcome(&{outcome_name}, &state);"));
            if let Some(definition) = project.message(outcome) {
                for update in &definition.state_updates {
                    let value = match &update.value {
                        StateValue::Bool(value) => {
                            let negation = if *value { "" } else { "!" };
                            writer
                                .line(&format!("assert!({negation}state.{}.get());", update.field));
                            continue;
                        }
                        StateValue::Text(value) => {
                            format!("{}.to_owned()", rust_string(value))
                        }
                        StateValue::Number(value) => {
                            format!("{}_f32", format_f32(*value))
                        }
                    };
                    writer.line(&format!(
                        "assert_eq!(state.{}.get(), {value});",
                        update.field
                    ));
                }
            }
        }
        writer.close();
        writer.close();
    }
    writer.finish()
}

fn pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            match characters.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + characters.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

fn emit_let_message(writer: &mut RustWriter, variable: &str, key: &MessageKey) {
    let domain = pascal_case(&key.domain);
    writer.line(&format!(
        "let {variable}_domain = crate::messages::{domain}Message::{};",
        key.variant
    ));
    writer.line(&format!(
        "let {variable} = crate::messages::AppMessage::{domain}({variable}_domain);"
    ));
}

fn message_pattern(key: &MessageKey) -> String {
    let domain = pascal_case(&key.domain);
    format!(
        "AppMessage::{domain}(crate::messages::{domain}Message::{})",
        key.variant
    )
}

fn interaction_message<'a>(
    project: &'a CitizenProject,
    node: &DesignNode,
    event: InteractionEvent,
) -> Option<&'a MessageKey> {
    project
        .interactions
        .iter()
        .find(|binding| binding.node == node.id && binding.event == event)
        .map(|binding| &binding.message)
}

fn emit_outbox_push(writer: &mut RustWriter, key: &MessageKey) {
    let domain = pascal_case(&key.domain);
    writer.line(&format!(
        "self.outbox.push(crate::messages::AppMessage::{domain}("
    ));
    writer.indent += 1;
    writer.line(&format!(
        "crate::messages::{domain}Message::{},",
        key.variant
    ));
    writer.indent -= 1;
    writer.line("));");
}

fn generate_async_backend(project: &CitizenProject) -> String {
    let mut writer = RustWriter::default();
    writer.line("//! Cancellable Level 3 async routing with native and browser backends.");
    writer.blank();
    writer.line("use crate::messages::AppMessage;");
    writer.blank();
    writer.open("fn route_async_intent(message: &AppMessage) -> Option<(AppMessage, u32)>");
    writer.open("match message");
    for mapping in &project.async_behavior.mappings {
        writer.open(&format!("{} =>", message_pattern(&mapping.intent)));
        let domain = pascal_case(&mapping.outcome.domain);
        writer.line(&format!(
            "let outcome = crate::messages::{domain}Message::{};",
            mapping.outcome.variant
        ));
        writer.line(&format!(
            "Some((AppMessage::{domain}(outcome), {}))",
            mapping.delay_ms
        ));
        writer.close();
    }
    writer.line("_ => None,");
    writer.close();
    writer.close();
    writer.blank();

    writer.line("#[cfg(not(target_arch = \"wasm32\"))]");
    writer.open("mod platform");
    writer.line("use std::sync::Arc;");
    writer.line("use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};");
    writer.line("use std::time::Duration;");
    writer.blank();
    writer.line("use egui_mobius::{AsyncDispatcher, Signal, Slot, factory};");
    writer.blank();
    writer.line("use super::{AppMessage, route_async_intent};");
    writer.blank();
    writer.line("#[derive(Clone)]");
    writer.open("struct AsyncRequest");
    writer.line("generation: u64,");
    writer.line("outcome: AppMessage,");
    writer.line("delay_ms: u32,");
    writer.close();
    writer.blank();
    writer.line("#[derive(Clone)]");
    writer.open("struct AsyncResult");
    writer.line("generation: u64,");
    writer.line("outcome: Option<AppMessage>,");
    writer.close();
    writer.blank();
    writer.line("/// Native signal/slot backend owned by the host application.");
    writer.open("pub struct AsyncBackend");
    writer.line("requests: Signal<AsyncRequest>,");
    writer.line("results: Slot<AsyncResult>,");
    writer.line("generation: Arc<AtomicU64>,");
    writer.line("pending: Arc<AtomicUsize>,");
    writer.line("_dispatcher: AsyncDispatcher<AsyncRequest, AsyncResult>,");
    writer.close();
    writer.blank();
    writer.open("impl AsyncBackend");
    writer.line("/// Construct the native Tokio-backed signal/slot pipeline.");
    writer.open("pub fn new() -> Self");
    writer.line("let (requests, request_slot) = factory::create_signal_slot::<AsyncRequest>();");
    writer.line("let (result_signal, results) = factory::create_signal_slot::<AsyncResult>();");
    writer.line("let dispatcher = AsyncDispatcher::<AsyncRequest, AsyncResult>::new();");
    writer.line("let generation = Arc::new(AtomicU64::new(0));");
    writer.line("let pending = Arc::new(AtomicUsize::new(0));");
    writer.line("let worker_generation = Arc::clone(&generation);");
    writer.line("let worker_pending = Arc::clone(&pending);");
    writer.open("dispatcher.attach_async(request_slot, result_signal, move |request|");
    writer.line("let generation = Arc::clone(&worker_generation);");
    writer.line("let pending = Arc::clone(&worker_pending);");
    writer.open("async move");
    writer.line("let mut elapsed_ms = 0;");
    writer.open("while elapsed_ms < request.delay_ms");
    writer.line("let slice_ms = (request.delay_ms - elapsed_ms).min(25);");
    writer.line("tokio::time::sleep(Duration::from_millis(u64::from(slice_ms))).await;");
    writer.line("elapsed_ms += slice_ms;");
    writer.open("if generation.load(Ordering::Acquire) != request.generation");
    writer.line("pending.fetch_sub(1, Ordering::AcqRel);");
    writer.open("return AsyncResult");
    writer.line("generation: request.generation,");
    writer.line("outcome: None,");
    writer.close_semicolon();
    writer.close();
    writer.close();
    writer.line("pending.fetch_sub(1, Ordering::AcqRel);");
    writer.open("AsyncResult");
    writer.line("generation: request.generation,");
    writer.line("outcome: Some(request.outcome),");
    writer.close();
    writer.close();
    writer.close_call();
    writer.open("Self");
    writer.line("requests,");
    writer.line("results,");
    writer.line("generation,");
    writer.line("pending,");
    writer.line("_dispatcher: dispatcher,");
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Submit a mapped intent. Returns false when the intent is synchronous.");
    writer.open("pub fn submit(&self, intent: &AppMessage) -> bool");
    writer.open("let Some((outcome, delay_ms)) = route_async_intent(intent) else");
    writer.line("return false;");
    writer.close_semicolon();
    writer.line("self.submit_outcome(outcome, delay_ms)");
    writer.close();
    writer.blank();
    writer.open("fn submit_outcome(&self, outcome: AppMessage, delay_ms: u32) -> bool");
    writer.line("let generation = self.generation.load(Ordering::Acquire);");
    writer.open("let request = AsyncRequest");
    writer.line("generation,");
    writer.line("outcome,");
    writer.line("delay_ms,");
    writer.close_semicolon();
    writer.line("self.pending.fetch_add(1, Ordering::AcqRel);");
    writer.open("if self.requests.send(request).is_err()");
    writer.line("self.pending.fetch_sub(1, Ordering::AcqRel);");
    writer.line("return false;");
    writer.close();
    writer.line("true");
    writer.close();
    writer.blank();
    writer.line("/// Drain completed outcomes on the UI thread.");
    writer.open("pub fn drain(&self) -> Vec<AppMessage>");
    writer.line("let generation = self.generation.load(Ordering::Acquire);");
    writer.line("let receiver = self");
    writer.indent += 1;
    writer.line(".results");
    writer.line(".receiver");
    writer.line(".lock()");
    writer.line(".expect(\"async result slot poisoned\");");
    writer.indent -= 1;
    writer.line("receiver");
    writer.indent += 1;
    writer.line(".try_iter()");
    writer.line(".filter(|result| result.generation == generation)");
    writer.line(".filter_map(|result| result.outcome)");
    writer.line(".collect()");
    writer.indent -= 1;
    writer.close();
    writer.blank();
    writer.line("/// Cooperatively cancel all pending native work.");
    writer.open("pub fn cancel_all(&self)");
    writer.line("self.generation.fetch_add(1, Ordering::AcqRel);");
    writer.close();
    writer.blank();
    writer.line("/// Whether no async request remains in flight.");
    writer.open("pub fn is_idle(&self) -> bool");
    writer.line("self.pending.load(Ordering::Acquire) == 0");
    writer.close();
    writer.close();
    writer.blank();
    writer.open("impl Default for AsyncBackend");
    writer.open("fn default() -> Self");
    writer.line("Self::new()");
    writer.close();
    writer.close();
    writer.blank();
    writer.line("#[cfg(test)]");
    writer.open("mod tests");
    writer.line("use super::*;");
    writer.blank();
    writer.line("#[test]");
    writer.open("fn native_work_completes_and_cancels_cooperatively()");
    emit_let_message(
        &mut writer,
        "intent",
        &project.async_behavior.mappings[0].intent,
    );
    writer.line(
        "let (expected, _) = route_async_intent(&intent).expect(\"fixture intent is mapped\");",
    );
    writer.line("let backend = AsyncBackend::new();");
    writer.line("assert!(backend.submit_outcome(expected.clone(), 5));");
    writer.line("let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);");
    writer.open("let received = loop");
    writer.open("if let Some(outcome) = backend.drain().into_iter().next()");
    writer.line("break outcome;");
    writer.close();
    writer.line("assert!(");
    writer.indent += 1;
    writer.line("std::time::Instant::now() < deadline,");
    writer.line("\"async result timed out\"");
    writer.indent -= 1;
    writer.line(");");
    writer.line("std::thread::sleep(std::time::Duration::from_millis(5));");
    writer.close_semicolon();
    writer.line("assert_eq!(received, expected);");
    writer.blank();
    writer.line("assert!(backend.submit_outcome(received, 200));");
    writer.line("backend.cancel_all();");
    writer.line("let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);");
    writer.open("while !backend.is_idle() && std::time::Instant::now() < deadline");
    writer.line("std::thread::sleep(std::time::Duration::from_millis(5));");
    writer.close();
    writer.line("assert!(backend.is_idle(), \"cancelled work did not stop\");");
    writer.line("assert!(backend.drain().is_empty());");
    writer.close();
    writer.close();
    writer.close();
    writer.blank();

    writer.line("#[cfg(target_arch = \"wasm32\")]");
    writer.open("mod platform");
    writer.line("use std::cell::{Cell, RefCell};");
    writer.line("use std::rc::Rc;");
    writer.blank();
    writer.line("use futures::future::{AbortHandle, Abortable};");
    writer.line("use gloo_timers::future::TimeoutFuture;");
    writer.blank();
    writer.line("use super::{AppMessage, route_async_intent};");
    writer.blank();
    writer.line("type PendingTasks = Rc<RefCell<Vec<(u64, AbortHandle)>>>;");
    writer.blank();
    writer.line("/// Browser backend using abortable local futures on the UI event loop.");
    writer.open("pub struct AsyncBackend");
    writer.line("generation: Rc<Cell<u64>>,");
    writer.line("next_task: Rc<Cell<u64>>,");
    writer.line("pending: PendingTasks,");
    writer.line("completed: Rc<RefCell<Vec<AppMessage>>>,");
    writer.close();
    writer.blank();
    writer.open("impl AsyncBackend");
    writer.line("/// Construct the WASM-compatible local async pipeline.");
    writer.open("pub fn new() -> Self");
    writer.open("Self");
    writer.line("generation: Rc::new(Cell::new(0)),");
    writer.line("next_task: Rc::new(Cell::new(0)),");
    writer.line("pending: Rc::new(RefCell::new(Vec::new())),");
    writer.line("completed: Rc::new(RefCell::new(Vec::new())),");
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Submit a mapped intent. Returns false when the intent is synchronous.");
    writer.open("pub fn submit(&self, intent: &AppMessage) -> bool");
    writer.open("let Some((outcome, delay_ms)) = route_async_intent(intent) else");
    writer.line("return false;");
    writer.close_semicolon();
    writer.line("let generation_value = self.generation.get();");
    writer.line("let generation = Rc::clone(&self.generation);");
    writer.line("let completed = Rc::clone(&self.completed);");
    writer.line("let pending = Rc::clone(&self.pending);");
    writer.line("let task_id = self.next_task.get().wrapping_add(1);");
    writer.line("self.next_task.set(task_id);");
    writer.line("let (abort_handle, registration) = AbortHandle::new_pair();");
    writer.line("self.pending.borrow_mut().push((task_id, abort_handle));");
    writer.open("wasm_bindgen_futures::spawn_local(async move");
    writer.open("let task = async move");
    writer.line("TimeoutFuture::new(delay_ms).await;");
    writer.open("if generation.get() == generation_value");
    writer.line("completed.borrow_mut().push(outcome);");
    writer.close();
    writer.close_semicolon();
    writer.line("let _ = Abortable::new(task, registration).await;");
    writer.line("pending.borrow_mut().retain(|(id, _)| *id != task_id);");
    writer.close_call();
    writer.line("true");
    writer.close();
    writer.blank();
    writer.line("/// Drain completed outcomes on the UI thread.");
    writer.open("pub fn drain(&self) -> Vec<AppMessage>");
    writer.line("std::mem::take(&mut *self.completed.borrow_mut())");
    writer.close();
    writer.blank();
    writer.line("/// Abort every pending browser future.");
    writer.open("pub fn cancel_all(&self)");
    writer.line("self.generation.set(self.generation.get().wrapping_add(1));");
    writer.line("self.completed.borrow_mut().clear();");
    writer.open("for (_, handle) in self.pending.borrow_mut().drain(..)");
    writer.line("handle.abort();");
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Whether no async request remains in flight.");
    writer.open("pub fn is_idle(&self) -> bool");
    writer.line("self.pending.borrow().is_empty()");
    writer.close();
    writer.close();
    writer.blank();
    writer.open("impl Default for AsyncBackend");
    writer.open("fn default() -> Self");
    writer.line("Self::new()");
    writer.close();
    writer.close();
    writer.close();
    writer.blank();
    writer.line("pub use platform::AsyncBackend;");
    writer.finish()
}

fn generate_library(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let component_nodes = project
        .node_ids_depth_first()
        .into_iter()
        .filter_map(|id| project.find_node(id))
        .filter(|node| {
            matches!(
                &node.kind,
                NodeKind::ReactiveLogger | NodeKind::ReactiveEditor { .. }
            )
        })
        .collect::<Vec<_>>();
    let mut writer = RustWriter::default();
    writer.line("//! Generated Citizen library.");
    for line in project.description.lines() {
        writer.line(&format!("//! {}", line.trim()));
    }
    writer.blank();
    if !project.assets.is_empty() {
        writer.line("pub mod assets;");
    }
    if project.async_behavior.enabled {
        writer.line("#[cfg(feature = \"async-backend\")]");
        writer.line("pub mod async_backend;");
    }
    writer.line("pub mod backend;");
    writer.line("pub mod messages;");
    writer.line("pub mod theme;");
    writer.blank();
    writer.line("use crate::messages::AppMessage;");
    writer.line("use egui_citizen::{Citizen, CitizenId, CitizenState};");
    if project.inferred_features().contains("component-lens") {
        writer.line("#[cfg(feature = \"component-lens\")]");
        writer.line("use egui_lens::{LogColors, ReactiveEventLogger, ReactiveEventLoggerState};");
    }
    writer.line("use egui_mobius_reactive::Dynamic;");
    if project.inferred_features().contains("component-quill") {
        writer.line("#[cfg(feature = \"component-quill\")]");
        writer.line("use egui_quill::{ReactiveEditor, ReactiveEditorState};");
    }
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
    writer.line("outbox: Vec<AppMessage>,");
    for node in &component_nodes {
        match &node.kind {
            NodeKind::ReactiveLogger => {
                writer.line("#[cfg(feature = \"component-lens\")]");
                writer.line(&format!(
                    "{}_state: Dynamic<ReactiveEventLoggerState>,",
                    component_field_name(node)
                ));
                writer.line("#[cfg(feature = \"component-lens\")]");
                writer.line(&format!(
                    "{}_colors: Dynamic<LogColors>,",
                    component_field_name(node)
                ));
            }
            NodeKind::ReactiveEditor { .. } => {
                writer.line("#[cfg(feature = \"component-quill\")]");
                writer.line(&format!(
                    "{}_state: Dynamic<ReactiveEditorState>,",
                    component_field_name(node)
                ));
            }
            _ => {}
        }
    }
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
    writer.line("outbox: Vec::new(),");
    for node in &component_nodes {
        match &node.kind {
            NodeKind::ReactiveLogger => {
                writer.line("#[cfg(feature = \"component-lens\")]");
                writer.line(&format!(
                    "{}_state: Dynamic::new(ReactiveEventLoggerState::new()),",
                    component_field_name(node)
                ));
                writer.line("#[cfg(feature = \"component-lens\")]");
                writer.line(&format!(
                    "{}_colors: Dynamic::new(LogColors::default()),",
                    component_field_name(node)
                ));
            }
            NodeKind::ReactiveEditor { content, language } => {
                writer.line("#[cfg(feature = \"component-quill\")]");
                writer.line(&format!(
                    "{}_state: Dynamic::new(",
                    component_field_name(node)
                ));
                writer.indent += 1;
                writer.line("ReactiveEditorState::new()");
                writer.indent += 1;
                writer.line(&format!(".with_content({})", rust_string(content)));
                writer.line(&format!(".with_language({}),", rust_string(language)));
                writer.indent -= 2;
                writer.line("),");
            }
            _ => {}
        }
    }
    writer.close();
    writer.close();
    writer.blank();
    writer.line("/// Drain discrete intents for routing by the host application.");
    writer.open("pub fn drain_outbox(&mut self) -> Vec<AppMessage>");
    writer.line("std::mem::take(&mut self.outbox)");
    writer.close();
    writer.blank();
    writer.line("/// Render this Citizen inside its host-provided panel UI.");
    writer.open("pub fn show(&mut self, ui: &mut egui::Ui)");
    emit_node(project, &project.root, &mut writer);
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

fn component_field_name(node: &DesignNode) -> String {
    format!("component_{}", node.id.0)
}

fn emit_node(project: &CitizenProject, node: &DesignNode, writer: &mut RustWriter) {
    writer.line(&format!("// {} ({})", node.name, node.kind.display_name()));
    match &node.kind {
        NodeKind::Column => {
            writer.open("ui.vertical(|ui|");
            emit_children(project, node, writer);
            writer.close_call();
        }
        NodeKind::Row { wrap } => {
            let method = if *wrap {
                "horizontal_wrapped"
            } else {
                "horizontal"
            };
            writer.open(&format!("ui.{method}(|ui|"));
            emit_children(project, node, writer);
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
                emit_node(project, child, writer);
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
            emit_children(project, node, writer);
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
            emit_children(project, node, writer);
            writer.close_call();
        }
        NodeKind::Label { text } => writer.line(&format!("ui.label({});", rust_string(text))),
        NodeKind::Heading { text } => {
            writer.line(&format!("ui.heading({});", rust_string(text)));
        }
        NodeKind::Button { text } => {
            if let Some(message) = interaction_message(project, node, InteractionEvent::Click) {
                writer.line(&format!("let response = ui.button({});", rust_string(text)));
                writer.open("if response.clicked()");
                emit_outbox_push(writer, message);
                writer.close();
            } else {
                writer.line(&format!(
                    "let _response = ui.button({});",
                    rust_string(text)
                ));
            }
        }
        NodeKind::StyledButton { text } => {
            writer.line(
                "#[cfg(all(feature = \"component-widgets\", not(target_arch = \"wasm32\")))]",
            );
            writer.open("");
            writer.line(&format!(
                "let response = egui_mobius_widgets::StyledButton::new({}).show(ui);",
                rust_string(text)
            ));
            if let Some(message) = interaction_message(project, node, InteractionEvent::Click) {
                writer.open("if response.clicked()");
                emit_outbox_push(writer, message);
                writer.close();
            } else {
                writer.line("let _ = response;");
            }
            writer.close();
            writer.line(
                "#[cfg(not(all(feature = \"component-widgets\", not(target_arch = \"wasm32\"))))]",
            );
            writer.open("");
            writer.line(&format!("let response = ui.button({});", rust_string(text)));
            if let Some(message) = interaction_message(project, node, InteractionEvent::Click) {
                writer.open("if response.clicked()");
                emit_outbox_push(writer, message);
                writer.close();
            } else {
                writer.line("let _ = response;");
            }
            writer.close();
        }
        NodeKind::ReactiveLogger => {
            let field = component_field_name(node);
            writer.line("#[cfg(feature = \"component-lens\")]");
            writer.line(&format!(
                "ReactiveEventLogger::with_colors(&self.{field}_state, &self.{field}_colors)"
            ));
            writer.indent += 1;
            writer.line(".show(ui);");
            writer.indent -= 1;
            writer.line("#[cfg(not(feature = \"component-lens\"))]");
            writer.line("ui.weak(\"Enable `component-lens` to render this logger.\");");
        }
        NodeKind::ReactiveEditor { .. } => {
            let field = component_field_name(node);
            writer.line("#[cfg(feature = \"component-quill\")]");
            writer.line(&format!(
                "ReactiveEditor::new(&self.{field}_state).show(ui);"
            ));
            writer.line("#[cfg(not(feature = \"component-quill\"))]");
            writer.line("ui.weak(\"Enable `component-quill` to render this editor.\");");
        }
        NodeKind::LinePlot { binding } => {
            let binding = binding.as_deref().expect("validated binding");
            writer.line("#[cfg(feature = \"component-plot\")]");
            writer.open("");
            writer.line(&format!(
                "let amplitude = f64::from(self.state.{binding}.get());"
            ));
            writer.line("let points = (0..=128)");
            writer.indent += 1;
            writer.open(".map(|index|");
            writer.line("let x = f64::from(index) / 128.0_f64 * std::f64::consts::TAU;");
            writer.line("[x, amplitude * x.sin()]");
            writer.close_chain();
            writer.line(".collect::<Vec<_>>();");
            writer.indent -= 1;
            writer.line(&format!(
                "egui_plot::Plot::new({})",
                rust_string(&node.name)
            ));
            writer.indent += 1;
            writer.line(".height(220.0_f32)");
            writer.line(".show(ui, |plot_ui| {");
            writer.indent += 1;
            writer.line("plot_ui.line(egui_plot::Line::new(\"signal\", points));");
            writer.indent -= 1;
            writer.line("});");
            writer.indent -= 1;
            writer.close();
            writer.line("#[cfg(not(feature = \"component-plot\"))]");
            writer.line("ui.weak(\"Enable `component-plot` to render this plot.\");");
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
            if let Some(message) = interaction_message(project, node, InteractionEvent::Change) {
                emit_outbox_push(writer, message);
            }
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
            if let Some(message) = interaction_message(project, node, InteractionEvent::Change) {
                emit_outbox_push(writer, message);
            }
            writer.indent -= 1;
            writer.line("}");
            if let Some(message) = interaction_message(project, node, InteractionEvent::Submit) {
                writer.line(
                    "if response.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)) {",
                );
                writer.indent += 1;
                emit_outbox_push(writer, message);
                writer.indent -= 1;
                writer.line("}");
            }
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
            if let Some(message) = interaction_message(project, node, InteractionEvent::Change) {
                emit_outbox_push(writer, message);
            }
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

fn emit_children(project: &CitizenProject, node: &DesignNode, writer: &mut RustWriter) {
    for child in &node.children {
        emit_node(project, child, writer);
    }
}

fn generate_preview(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let mut fixture_initializers = String::new();
    for field in &project.state_fields {
        let value = project
            .preview
            .values
            .get(&field.name)
            .expect("validated fixture contains every state field");
        let expression = match value {
            StateValue::Bool(value) => value.to_string(),
            StateValue::Text(value) => format!("{}.to_owned()", rust_string(value)),
            StateValue::Number(value) => format!("{}_f32", format_f32(*value)),
        };
        let _ = writeln!(
            fixture_initializers,
            "        state.{}.set({expression});",
            field.name
        );
    }
    let citizen_imports = if project.async_behavior.enabled {
        "CitizenId, CitizenMessage, Dispatcher"
    } else {
        "CitizenId, Dispatcher"
    };
    let async_import = if project.async_behavior.enabled {
        format!(
            "use {}::async_backend::AsyncBackend;\n",
            project.crate_ident()
        )
    } else {
        String::new()
    };
    let async_field = if project.async_behavior.enabled {
        "    async_backend: AsyncBackend,\n"
    } else {
        ""
    };
    let async_initializer = if project.async_behavior.enabled {
        "            async_backend: AsyncBackend::new(),\n"
    } else {
        ""
    };
    let dock_imports = if project.composition.enabled {
        "DockArea, DockState, NodeIndex"
    } else {
        "DockArea, DockState"
    };
    let composition_variant = if project.composition.enabled {
        "    External(&'static str, &'static str, &'static str),\n"
    } else {
        ""
    };
    let composition_title_arm = if project.composition.enabled {
        "            PreviewTab::External(_, title, _) => (*title).into(),\n"
    } else {
        ""
    };
    let composition_ui_arm = if project.composition.enabled {
        r#"            PreviewTab::External(citizen_id, title, crate_name) => {
                ui.heading(*title);
                ui.monospace(format!("CitizenId: {citizen_id}"));
                ui.label(format!("Compile-time Citizen crate: {crate_name}"));
                ui.weak("Add the external crate to the host and replace this preview placeholder with its concrete Citizen::show call.");
            }
"#
    } else {
        ""
    };
    let composition_id_arm = if project.composition.enabled {
        "                PreviewTab::External(citizen_id, _, _) => CitizenId::new(*citizen_id),\n"
    } else {
        ""
    };
    let mut composition_registrations = String::new();
    let mut composition_dock = String::new();
    if project.composition.enabled {
        for (index, external) in project.composition.external_citizens.iter().enumerate() {
            writeln!(
                composition_registrations,
                "        let _external_{index}_lifecycle = dispatcher.register(CitizenId::new({}));",
                rust_string(&external.citizen_id)
            )
            .expect("String writes do not fail");
        }
        composition_dock.push_str(
            "        let mut dock_state = DockState::new(vec![PreviewTab::Citizen, PreviewTab::Messages]);\n",
        );
        let has_split = project
            .composition
            .external_citizens
            .iter()
            .any(|external| external.placement != DockPlacement::Tab);
        if has_split {
            composition_dock.push_str("        let mut primary_node = NodeIndex::root();\n");
        } else {
            composition_dock.push_str("        let primary_node = NodeIndex::root();\n");
        }
        for (index, external) in project.composition.external_citizens.iter().enumerate() {
            writeln!(
                composition_dock,
                "        let external_tab_{index} = PreviewTab::External({}, {}, {});",
                rust_string(&external.citizen_id),
                rust_string(&external.title),
                rust_string(&external.crate_name)
            )
            .expect("String writes do not fail");
        }
        for (index, external) in project.composition.external_citizens.iter().enumerate() {
            if external.placement == DockPlacement::Tab {
                continue;
            }
            let method = match external.placement {
                DockPlacement::Left => "split_left",
                DockPlacement::Right => "split_right",
                DockPlacement::Above => "split_above",
                DockPlacement::Below => "split_below",
                DockPlacement::Tab => unreachable!(),
            };
            writeln!(
                composition_dock,
                "        let [next_primary, _external_{index}_node] =\n            dock_state\n                .main_surface_mut()\n                .{method}(primary_node, {}, vec![external_tab_{index}]);\n        primary_node = next_primary;",
                format_f32(1.0 - external.fraction)
            )
            .expect("String writes do not fail");
        }
        for (index, external) in project.composition.external_citizens.iter().enumerate() {
            if external.placement != DockPlacement::Tab {
                continue;
            }
            writeln!(
                composition_dock,
                "        dock_state.main_surface_mut()[primary_node].append_tab(external_tab_{index});"
            )
            .expect("String writes do not fail");
        }
        composition_dock.push_str("        let _ = primary_node;\n");
    } else {
        composition_dock.push_str(
            "        let dock_state = DockState::new(vec![PreviewTab::Citizen, PreviewTab::Messages]);\n",
        );
    }
    let message_routing = if project.async_behavior.enabled {
        format!(
            r#"        let mut citizen_deactivated = false;
        for lifecycle in self.dispatcher.drain_messages() {{
            let cancel_async = matches!(
                &lifecycle,
                CitizenMessage::Deactivated {{ id }} if id.0.as_str() == {citizen_type}::ID
            );
            self.log_event("lifecycle", &format!("{{lifecycle:?}}"));
            if cancel_async {{
                citizen_deactivated = true;
                self.async_backend.cancel_all();
                self.log_event("async-cancel", "Citizen deactivated");
            }}
        }}
        for intent in self.citizen.drain_outbox() {{
            self.log_event("intent", intent.name());
            if citizen_deactivated {{
                self.log_event("async-cancel", "Discarded intent from inactive Citizen");
                continue;
            }}
            if self.async_backend.submit(&intent) {{
                self.log_event("async-start", intent.name());
            }} else if let Some(outcome) = self.backend.handle(&intent, &self.citizen.state) {{
                apply_outcome(&outcome, &self.citizen.state);
                self.log_event("outcome", outcome.name());
            }}
        }}
        for outcome in self.async_backend.drain() {{
            apply_outcome(&outcome, &self.citizen.state);
            self.log_event("async-outcome", outcome.name());
        }}
        if !self.async_backend.is_idle() {{
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }}
"#,
            citizen_type = project.citizen_type,
        )
    } else {
        r#"        for lifecycle in self.dispatcher.drain_messages() {
            self.log_event("lifecycle", &format!("{lifecycle:?}"));
        }
        for intent in self.citizen.drain_outbox() {
            self.log_event("intent", intent.name());
            if let Some(outcome) = self.backend.handle(&intent, &self.citizen.state) {
                apply_outcome(&outcome, &self.citizen.state);
                self.log_event("outcome", outcome.name());
            }
        }
"#
        .to_owned()
    };
    let mut source = format!(
        r#"//! Native and WASM preview host generated by citizen-builder.

{async_import}use {crate_ident}::backend::{{ReferenceBackend, apply_outcome}};
use {crate_ident}::{{{citizen_type}, {state_type}}};

use eframe::egui;
use egui_citizen::{{{citizen_imports}}};
use egui_dock::{{{dock_imports}}};
#[cfg(feature = "lens")]
use egui_lens::{{LogColors, ReactiveEventLogger, ReactiveEventLoggerState}};
#[cfg(feature = "lens")]
use egui_mobius_reactive::Dynamic;

#[derive(Clone)]
enum PreviewTab {{
    Citizen,
    Messages,
{composition_variant}}}

struct PreviewViewer<'a> {{
    dispatcher: &'a mut Dispatcher,
    citizen: &'a mut {citizen_type},
    #[cfg(not(feature = "lens"))]
    event_log: &'a [String],
    #[cfg(feature = "lens")]
    lens_state: &'a Dynamic<ReactiveEventLoggerState>,
    #[cfg(feature = "lens")]
    lens_colors: &'a Dynamic<LogColors>,
}}

impl egui_dock::TabViewer for PreviewViewer<'_> {{
    type Tab = PreviewTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {{
        match tab {{
            PreviewTab::Citizen => {citizen_type}::TITLE.into(),
            PreviewTab::Messages => "Message path".into(),
{composition_title_arm}        }}
    }}

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {{
        match tab {{
            PreviewTab::Citizen => self.citizen.show(ui),
            PreviewTab::Messages => {{
                #[cfg(feature = "lens")]
                ReactiveEventLogger::with_colors(self.lens_state, self.lens_colors).show(ui);

                #[cfg(not(feature = "lens"))]
                egui::ScrollArea::vertical().show(ui, |ui| {{
                    ui.weak("Enable the `lens` feature for filtering and colored logs.");
                    for event in self.event_log {{
                        ui.monospace(event);
                    }}
                }});
            }}
{composition_ui_arm}        }}
    }}

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {{
        if response.clicked() {{
            let citizen_id = match tab {{
                PreviewTab::Citizen => CitizenId::new({citizen_type}::ID),
                PreviewTab::Messages => CitizenId::new("reference_message_log"),
{composition_id_arm}            }};
            self.dispatcher.activate(&citizen_id);
        }}
    }}
}}

struct PreviewApp {{
    dispatcher: Dispatcher,
    dock_state: DockState<PreviewTab>,
    citizen: {citizen_type},
    backend: ReferenceBackend,
{async_field}    event_log: Vec<String>,
    #[cfg(feature = "lens")]
    lens_state: Dynamic<ReactiveEventLoggerState>,
    #[cfg(feature = "lens")]
    lens_colors: Dynamic<LogColors>,
}}

impl PreviewApp {{
    fn new(creation_context: &eframe::CreationContext<'_>) -> Self {{
        {crate_ident}::theme::apply(&creation_context.egui_ctx);
        let mut dispatcher = Dispatcher::new();
        let lifecycle = dispatcher.register(CitizenId::new({citizen_type}::ID));
        let _message_log_lifecycle = dispatcher.register(CitizenId::new("reference_message_log"));
{composition_registrations}        dispatcher.activate(&CitizenId::new({citizen_type}::ID));
        let _ = dispatcher.drain_messages();
        let state = {state_type}::default();
{fixture_initializers}{composition_dock}        Self {{
            dispatcher,
            dock_state,
            citizen: {citizen_type}::new(lifecycle, state),
            backend: ReferenceBackend,
{async_initializer}            event_log: Vec::new(),
            #[cfg(feature = "lens")]
            lens_state: Dynamic::new(ReactiveEventLoggerState::new()),
            #[cfg(feature = "lens")]
            lens_colors: Dynamic::new(LogColors::default()),
        }}
    }}

    fn log_event(&mut self, kind: &str, message: &str) {{
        self.event_log.push(format!("{{kind}}: {{message}}"));
        #[cfg(feature = "lens")]
        ReactiveEventLogger::with_colors(&self.lens_state, &self.lens_colors)
            .log_custom(kind, message);
    }}
}}

impl eframe::App for PreviewApp {{
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {{
        {{
            let mut viewer = PreviewViewer {{
                dispatcher: &mut self.dispatcher,
                citizen: &mut self.citizen,
                #[cfg(not(feature = "lens"))]
                event_log: &self.event_log,
                #[cfg(feature = "lens")]
                lens_state: &self.lens_state,
                #[cfg(feature = "lens")]
                lens_colors: &self.lens_colors,
            }};
            DockArea::new(&mut self.dock_state).show_inside(ui, &mut viewer);
        }}

{message_routing}    }}
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
        citizen_imports = citizen_imports,
        async_import = async_import,
        async_field = async_field,
        async_initializer = async_initializer,
        dock_imports = dock_imports,
        composition_variant = composition_variant,
        composition_title_arm = composition_title_arm,
        composition_ui_arm = composition_ui_arm,
        composition_id_arm = composition_id_arm,
        composition_registrations = composition_registrations,
        composition_dock = composition_dock,
        message_routing = message_routing,
        fixture_initializers = fixture_initializers,
        title = rust_string(&format!("{} Preview", project.title)),
    );
    if !source.ends_with('\n') {
        source.push('\n');
    }
    source
}

fn generate_readme(project: &CitizenProject, preview_bin: &str) -> String {
    let async_note = if project.async_behavior.enabled {
        " This project also enables a cancellable Level 3 backend: native builds use egui_mobius `Signal`/`Slot` and `AsyncDispatcher`, while WASM uses abortable local futures."
    } else {
        ""
    };
    format!(
        "# {}\n\n{}\n\nThis crate contains one reusable egui_mobius Citizen generated by [citizen-builder](https://github.com/timschmidt/citizen-builder). Continuous UI data uses typed `Dynamic<T>` fields; discrete work crosses the host boundary as domain-grouped `AppMessage` intents and outcomes.{}\n\n## Native preview\n\n```shell\ncargo run --features preview,lens --bin {}\n```\n\nOmit `lens` for the lightweight built-in message log.\n\n## WASM preview\n\n```shell\nrustup target add wasm32-unknown-unknown\ncargo install trunk\ntrunk serve --open\n```\n\nSee [`host-integration.md`](host-integration.md) for lifecycle, docking, outbox routing, cancellation, and UI-thread outcome integration points.\n",
        project.title, project.description, async_note, preview_bin
    )
}

fn generate_host_integration(project: &CitizenProject) -> String {
    let state_type = format!("{}State", project.citizen_type);
    let state_field = format!("{}_state", project.citizen_id);
    let citizen_type = &project.citizen_type;
    let citizen_id = &project.citizen_id;
    let async_field = if project.async_behavior.enabled {
        "async_backend: AsyncBackend,\n"
    } else {
        ""
    };
    let async_initializer = if project.async_behavior.enabled {
        "let async_backend = AsyncBackend::new();\n"
    } else {
        ""
    };
    let routing = if project.async_behavior.enabled {
        format!(
            "for intent in self.{citizen_id}.drain_outbox() {{\n    if self.async_backend.submit(&intent) {{\n        continue;\n    }}\n    if let Some(outcome) = self.backend.handle(&intent, &self.{citizen_id}.state) {{\n        apply_outcome(&outcome, &self.{citizen_id}.state);\n    }}\n}}\nfor outcome in self.async_backend.drain() {{\n    apply_outcome(&outcome, &self.{citizen_id}.state);\n}}"
        )
    } else {
        format!(
            "for intent in self.{citizen_id}.drain_outbox() {{\n    if let Some(outcome) = self.backend.handle(&intent, &self.{citizen_id}.state) {{\n        apply_outcome(&outcome, &self.{citizen_id}.state);\n    }}\n}}"
        )
    };
    let cancellation = if project.async_behavior.enabled {
        format!(
            "\n\nWhen draining lifecycle messages, call `async_backend.cancel_all()` for `CitizenMessage::Deactivated {{ id }}` when `id` is `{citizen_id}`. Native tasks stop cooperatively; browser futures are aborted. In both cases, only `drain()` returns outcomes, so reactive state remains on the UI thread."
        )
    } else {
        String::new()
    };
    format!(
        "# Host integration\n\n`{citizen_type}` is a compile-time Citizen plug-in. Add this crate to the host, then make these five integration edits.\n\n1. Add fields to the host:\n\n```rust\n{citizen_id}: {citizen_type},\n{state_field}: {state_type},\nbackend: ReferenceBackend,\n{async_field}```\n\n2. Register and construct it during startup:\n\n```rust\nlet lifecycle = dispatcher.register(egui_citizen::CitizenId::new({citizen_type}::ID));\nlet {state_field} = {state_type}::default();\nlet {citizen_id} = {citizen_type}::new(lifecycle, {state_field}.clone());\n{async_initializer}```\n\n3. Add a `TabKind` variant and render arm:\n\n```rust\nTabKind::{citizen_type} => self.{citizen_id}.show(ui),\n```\n\n4. In `TabViewer::on_tab_button`, call `dispatcher.activate(...)`, then drain the lifecycle Dispatcher once after the dock renders.\n\n5. Drain and route discrete intents after UI rendering. Apply outcomes on the UI thread:\n\n```rust\n{routing}\n```{cancellation}\n\nThe generated `src/bin/preview.rs` is a complete executable reference with a visible intent/outcome path and optional `egui_lens` logging.\n",
    )
}

fn generate_host_composition(project: &CitizenProject) -> String {
    let mut dependencies = String::new();
    let mut fields = String::new();
    let mut registrations = String::new();
    let mut variants = String::new();
    let mut render_arms = String::new();
    let mut activation_arms = String::new();
    let mut split_placements = String::new();
    let mut tab_placements = String::new();
    for (index, external) in project.composition.external_citizens.iter().enumerate() {
        let crate_ident = external.crate_name.replace('-', "_");
        let tab_variant = pascal_case(&external.citizen_id);
        let state_type = format!("{}State", external.citizen_type);
        writeln!(
            dependencies,
            "{} = {{ path = \"../{}\" }}",
            external.crate_name, external.crate_name
        )
        .expect("String writes do not fail");
        writeln!(
            fields,
            "    {}: {crate_ident}::{},",
            external.citizen_id, external.citizen_type
        )
        .expect("String writes do not fail");
        writeln!(
            registrations,
            "let {id}_state = {crate_ident}::{state_type}::default();\nlet {id}_lifecycle = dispatcher.register(CitizenId::new({crate_ident}::{citizen_type}::ID));\nlet {id} = {crate_ident}::{citizen_type}::new({id}_lifecycle, {id}_state);",
            id = external.citizen_id,
            citizen_type = external.citizen_type,
        )
        .expect("String writes do not fail");
        writeln!(variants, "    {tab_variant},").expect("String writes do not fail");
        writeln!(
            render_arms,
            "    TabKind::{tab_variant} => self.{}.show(ui),",
            external.citizen_id
        )
        .expect("String writes do not fail");
        writeln!(
            activation_arms,
            "    TabKind::{tab_variant} => CitizenId::new({crate_ident}::{}::ID),",
            external.citizen_type
        )
        .expect("String writes do not fail");

        let tab = format!("TabKind::{tab_variant}");
        match external.placement {
            DockPlacement::Tab => {
                writeln!(
                    tab_placements,
                    "dock_state.main_surface_mut()[primary_node].append_tab({tab});"
                )
                .expect("String writes do not fail");
            }
            placement => {
                let method = match placement {
                    DockPlacement::Left => "split_left",
                    DockPlacement::Right => "split_right",
                    DockPlacement::Above => "split_above",
                    DockPlacement::Below => "split_below",
                    DockPlacement::Tab => unreachable!(),
                };
                writeln!(
                    split_placements,
                    "let [next_primary, _external_{index}] =\n    dock_state\n        .main_surface_mut()\n        .{method}(primary_node, {}, vec![{tab}]);\nprimary_node = next_primary;",
                    format_f32(1.0 - external.fraction)
                )
                .expect("String writes do not fail");
            }
        }
    }

    format!(
        "# Multi-Citizen host composition\n\nThis project still exports exactly one reusable Citizen: `{primary}`. The declarations below are host scaffolding for composing it with external Citizens through `egui_dock`; they do not couple those Citizens into the library crate.\n\n## Cargo dependencies\n\n```toml\n{dependencies}```\n\nReplace the sibling paths with the versions, Git revisions, or workspace paths used by your host.\n\n## Host fields and registration\n\n```rust\nuse egui_citizen::{{Citizen as _, CitizenId, Dispatcher}};\n\nstruct Host {{\n    {primary}: {crate_ident}::{primary_type},\n{fields}}}\n\nlet mut dispatcher = Dispatcher::new();\n{registrations}```\n\nRegister the primary `{primary_type}` as shown in [`host-integration.md`](host-integration.md), then initialize all fields in `Host`.\n\n## Render and activate tabs\n\n```rust\nenum TabKind {{\n    {primary_type},\n{variants}}}\n\nmatch tab {{\n    TabKind::{primary_type} => self.{primary}.show(ui),\n{render_arms}}}\n\nlet citizen_id = match tab {{\n    TabKind::{primary_type} => CitizenId::new({crate_ident}::{primary_type}::ID),\n{activation_arms}}};\ndispatcher.activate(&citizen_id);\n```\n\n## Author the initial dock\n\nThe saved fraction is the external Citizen's share; `egui_dock` receives the remaining primary share.\n\n```rust\nuse egui_dock::{{DockState, NodeIndex}};\n\nlet mut dock_state = DockState::new(vec![TabKind::{primary_type}]);\nlet mut primary_node = NodeIndex::root();\n{split_placements}{tab_placements}let _ = primary_node;\n```\n\nKeep every Citizen's lifecycle and intent/outcome routing host-owned and apply outcomes on the UI thread. The generated preview shows labeled external placeholders because this standalone crate intentionally does not link arbitrary neighboring Citizens.\n",
        primary = project.citizen_id,
        primary_type = project.citizen_type,
        crate_ident = project.crate_ident(),
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
  <link data-trunk rel="rust" data-bin="{preview_bin}" data-cargo-features="preview,lens">
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

    fn close_chain(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.line("})");
    }

    fn close_semicolon(&mut self) {
        self.indent = self.indent.saturating_sub(1);
        self.line("};");
    }

    fn finish(self) -> String {
        self.output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::FrameworkSource;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    fn fnv1a(contents: &str) -> u64 {
        contents
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }

    fn file_snapshot(generated: &GeneratedCrate) -> Vec<(String, usize, u64)> {
        generated
            .files
            .iter()
            .map(|(path, contents)| (path.clone(), contents.len(), fnv1a(contents)))
            .collect()
    }

    fn all_node_project() -> CitizenProject {
        use crate::model::PaletteItem;

        let mut project = CitizenProject::default();
        let root = project.root.id;
        let row = project.add_palette_item(Some(root), PaletteItem::Row);
        project.add_palette_item(Some(row), PaletteItem::Button);
        project.add_palette_item(Some(row), PaletteItem::StyledButton);
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
        let scroll_column = project.add_palette_item(Some(scroll), PaletteItem::Column);
        project.add_palette_item(Some(scroll_column), PaletteItem::Slider);
        project.add_palette_item(Some(scroll_column), PaletteItem::ProgressBar);
        project.add_palette_item(Some(root), PaletteItem::ReactiveLogger);
        project.add_palette_item(Some(root), PaletteItem::ReactiveEditor);
        project.add_palette_item(Some(root), PaletteItem::LinePlot);
        project
    }

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
            "src/backend.rs",
            "src/lib.rs",
            "src/messages.rs",
            "src/theme.rs",
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
        assert!(manifest.contains("../egui_mobius/crates/egui_lens"));
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
        assert!(library.contains("outbox: Vec<AppMessage>"));
        assert!(library.contains("pub fn drain_outbox"));
        assert!(library.contains("SettingsMessage::ApplyRequested"));
    }

    #[test]
    fn level_two_generation_separates_and_routes_discrete_messages() {
        let generated = generate(&CitizenProject::default()).unwrap();
        let messages = generated.file("src/messages.rs").unwrap();
        let backend = generated.file("src/backend.rs").unwrap();
        let preview = generated.file("src/bin/preview.rs").unwrap();

        assert!(messages.contains("pub enum AppMessage"));
        assert!(messages.contains("pub enum SettingsMessage"));
        assert!(messages.contains("ApplyRequested"));
        assert!(messages.contains("ApplyCompleted"));
        assert!(backend.contains("pub struct ReferenceBackend"));
        assert!(backend.contains("pub fn apply_outcome"));
        assert!(backend.contains("state.display_name.set(\"Applied\".to_owned())"));
        assert!(backend.contains("reference_backend_routes_intents_and_applies_outcomes"));
        assert!(preview.contains("self.citizen.drain_outbox()"));
        assert!(preview.contains("self.log_event(\"intent\", intent.name())"));
        assert!(preview.contains("self.log_event(\"outcome\", outcome.name())"));
        assert!(preview.contains("ReactiveEventLogger::with_colors"));
    }

    #[test]
    fn level_three_generation_is_opt_in_and_target_gated() {
        let default = generate(&CitizenProject::default()).unwrap();
        assert!(default.file("src/async_backend.rs").is_none());
        assert!(
            !default
                .file("Cargo.toml")
                .unwrap()
                .contains("async-backend")
        );

        let mut project = CitizenProject::default();
        project.async_behavior.enabled = true;
        project.async_behavior.mappings[0].delay_ms = 10;
        let async_source = generate_async_backend(&project);
        if let Err(error) = syn::parse_file(&async_source) {
            panic!("async backend syntax error: {error}\n{async_source}");
        }
        let generated = generate(&project).unwrap();
        let manifest = generated.file("Cargo.toml").unwrap();
        let backend = generated.file("src/async_backend.rs").unwrap();
        let preview = generated.file("src/bin/preview.rs").unwrap();

        assert!(manifest.contains("async-backend = ["));
        assert!(manifest.contains("egui_mobius = { git ="));
        assert!(manifest.contains("tokio = { version = \"1.52.1\""));
        assert!(manifest.contains("gloo-timers"));
        assert!(backend.contains("AsyncDispatcher::<AsyncRequest, AsyncResult>::new()"));
        assert!(backend.contains("factory::create_signal_slot"));
        assert!(backend.contains("pub fn cancel_all"));
        assert!(backend.contains("AbortHandle::new_pair"));
        assert!(preview.contains("self.async_backend.submit(&intent)"));
        assert!(preview.contains("CitizenMessage::Deactivated"));
        assert!(preview.contains("self.async_backend.drain()"));
    }

    #[test]
    fn curated_templates_generate_their_inferred_ecosystem_features() {
        use crate::model::CitizenTemplate;

        for template in CitizenTemplate::ALL {
            let project = CitizenProject::from_template(*template);
            let generated = generate(&project).unwrap_or_else(|diagnostics| {
                panic!(
                    "{} template did not generate: {diagnostics:#?}",
                    template.display_name()
                )
            });
            let manifest = generated.file("Cargo.toml").unwrap();
            for feature in project.inferred_features() {
                match feature.as_str() {
                    "component-lens" | "component-plot" | "component-quill"
                    | "component-widgets" | "async-backend" => {
                        assert!(
                            manifest.contains(&feature),
                            "{} did not emit {feature}",
                            template.display_name()
                        );
                    }
                    "embedded-assets" => {
                        assert!(generated.file("src/assets.rs").is_some());
                    }
                    "host-composition" => {
                        assert!(generated.file("host-composition.md").is_some());
                    }
                    unexpected => panic!("unhandled inferred feature {unexpected}"),
                }
            }
        }
    }

    #[test]
    fn theme_assets_and_host_composition_are_emitted() {
        use crate::model::{AssetDefinition, AssetKind, DockPlacement, HostCitizen};

        let mut project = CitizenProject::default();
        project.assets.push(AssetDefinition {
            file_name: "help.md".to_owned(),
            kind: AssetKind::Text,
            contents: "# Help\n".to_owned(),
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
        let generated = generate(&project).unwrap();
        assert_eq!(generated.file("assets/help.md"), Some("# Help\n"));
        assert!(
            generated
                .file("src/assets.rs")
                .unwrap()
                .contains("pub const HELP_MD")
        );
        assert!(
            generated
                .file("src/theme.rs")
                .unwrap()
                .contains("style.spacing.item_spacing")
        );
        assert!(
            generated
                .file("src/bin/preview.rs")
                .unwrap()
                .contains("PreviewTab::External")
        );
        assert!(
            generated
                .file("host-composition.md")
                .unwrap()
                .contains("split_right")
        );
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
        assert!(preview.contains("state.enabled.set(false)"));
        assert!(preview.contains("state.display_name.set(\"Preview Citizen\".to_owned())"));
    }

    #[test]
    fn generation_is_deterministic() {
        let project = CitizenProject::default();
        assert_eq!(generate(&project).unwrap(), generate(&project).unwrap());
    }

    #[test]
    fn every_default_file_matches_its_golden_snapshot() {
        const GOLDEN: &[(&str, usize, u64)] = &[
            (".gitignore", 14, 14_995_896_359_876_899_134),
            ("Cargo.toml", 1_382, 1_094_513_618_164_468_579),
            ("README.md", 754, 7_196_891_225_137_058_697),
            ("Trunk.toml", 145, 14_939_467_107_560_880_908),
            ("citizen.json", 4_289, 7_719_420_408_759_829_591),
            ("host-integration.md", 1_204, 15_042_447_790_753_805_948),
            ("index.html", 723, 7_977_982_574_669_601_589),
            ("src/backend.rs", 1_956, 14_526_239_575_183_017_818),
            ("src/bin/preview.rs", 6_558, 2_293_129_792_260_615_330),
            ("src/lib.rs", 4_031, 7_456_848_881_883_145_320),
            ("src/messages.rs", 822, 195_968_056_571_156_936),
            ("src/theme.rs", 569, 15_720_035_924_432_644_901),
        ];
        let actual = file_snapshot(&generate(&CitizenProject::default()).unwrap());
        let expected = GOLDEN
            .iter()
            .map(|(path, len, hash)| ((*path).to_owned(), *len, *hash))
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    #[test]
    fn representative_all_node_library_matches_golden_snapshot() {
        const GOLDEN_LENGTH: usize = 10_254;
        const GOLDEN_HASH: u64 = 9_597_425_857_174_742_145;
        let generated = generate(&all_node_project()).unwrap();
        let library = generated.file("src/lib.rs").unwrap();
        assert_eq!(
            (library.len(), fnv1a(library)),
            (GOLDEN_LENGTH, GOLDEN_HASH)
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn export_writes_complete_crate_and_refuses_overwrite() {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let parent = std::env::temp_dir().join(format!(
            "citizen-builder-export-test-{}-{sequence}",
            std::process::id()
        ));
        std::fs::create_dir(&parent).unwrap();
        let generated = generate(&CitizenProject::default()).unwrap();
        let destination = generated.write_new(&parent).unwrap();
        for path in generated.files.keys() {
            assert!(destination.join(path).is_file(), "missing {path}");
        }
        assert_eq!(
            generated.write_new(&parent).unwrap_err().kind(),
            std::io::ErrorKind::AlreadyExists
        );
        std::fs::remove_dir_all(parent).unwrap();
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
        project.state_fields[2].value = StateValue::Number(2.0);
        let library = generate(&project)
            .unwrap()
            .file("src/lib.rs")
            .unwrap()
            .to_owned();
        assert!(library.contains("Dynamic::new(2.0_f32)"));
    }
}
