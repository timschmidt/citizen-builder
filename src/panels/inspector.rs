use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::{
    AssetDefinition, AssetKind, CitizenProject, DiagnosticSeverity, DockPlacement, FrameworkSource,
    HostCitizen, InteractionEvent, MessageKey, MessageRole, NodeId, NodeKind, StateAssignment,
    StateType, StateValue, ThemePreset,
};

/// Citizen metadata, state-contract, and node-property inspector.
pub(crate) struct InspectorPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl InspectorPanel {
    pub(crate) const ID: &'static str = "builder_inspector";

    pub(crate) fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(Self::ID),
            citizen_state,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, shared: &BuilderSharedState) {
        let mut project = shared.project.get();
        let selected = shared.selection.get();
        let mut changed = false;

        ui.heading("Citizen inspector");
        ui.separator();

        egui::ScrollArea::vertical()
            .id_salt("citizen_inspector_scroll")
            .show(ui, |ui| {
                egui::CollapsingHeader::new("Citizen crate")
                    .default_open(true)
                    .show(ui, |ui| {
                        changed |= text_property(ui, "Cargo package", &mut project.crate_name);
                        changed |= text_property(ui, "Citizen type", &mut project.citizen_type);
                        changed |= text_property(ui, "CitizenId", &mut project.citizen_id);
                        changed |= text_property(ui, "Dock title", &mut project.title);
                        ui.label("Description");
                        changed |= ui
                            .add(
                                egui::TextEdit::multiline(&mut project.description)
                                    .desired_rows(3)
                                    .desired_width(f32::INFINITY),
                            )
                            .changed();
                        ui.weak(format!(
                            "Generator {} · backend {} {}",
                            project.generator.generator_version,
                            project.generator.backend,
                            project.generator.backend_version
                        ));
                    });

                egui::CollapsingHeader::new("Framework source")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let is_git = matches!(project.framework, FrameworkSource::Git { .. });
                            if ui.selectable_label(is_git, "Pinned Git").clicked() && !is_git {
                                project.framework = FrameworkSource::default();
                                changed = true;
                            }
                            if ui.selectable_label(!is_git, "Local workspace").clicked() && is_git {
                                project.framework = FrameworkSource::Path {
                                    workspace: "../egui_mobius".to_owned(),
                                };
                                changed = true;
                            }
                        });
                        match &mut project.framework {
                            FrameworkSource::Git {
                                repository,
                                revision,
                            } => {
                                changed |= text_property(ui, "Repository", repository);
                                changed |= text_property(ui, "Exact revision", revision);
                            }
                            FrameworkSource::Path { workspace } => {
                                changed |= text_property(ui, "Workspace root", workspace);
                            }
                        }
                    });

                egui::CollapsingHeader::new("Theme & generated capabilities")
                    .default_open(false)
                    .show(ui, |ui| {
                        changed |= edit_theme(ui, &mut project);
                        ui.separator();
                        ui.strong("Inferred Cargo features");
                        let features = project.inferred_features();
                        if features.is_empty() {
                            ui.weak("Core egui + reactive state only");
                        } else {
                            ui.horizontal_wrapped(|ui| {
                                for feature in features {
                                    ui.monospace(feature);
                                }
                            });
                        }
                        ui.weak(
                            "Features are derived from the design and are not hand-maintained.",
                        );
                    });

                egui::CollapsingHeader::new("Embedded assets")
                    .default_open(false)
                    .show(ui, |ui| {
                        changed |= edit_assets(ui, &mut project);
                    });

                egui::CollapsingHeader::new("Multi-Citizen host composition")
                    .default_open(false)
                    .show(ui, |ui| {
                        changed |= edit_host_composition(ui, &mut project);
                    });

                egui::CollapsingHeader::new("Dynamic<T> state")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut pending_renames = Vec::new();
                        let mut pending_delete = None;
                        for (index, field) in project.state_fields.iter_mut().enumerate() {
                            ui.group(|ui| {
                                let old_name = field.name.clone();
                                let mut new_name = old_name.clone();
                                ui.horizontal(|ui| {
                                    ui.monospace(field.state_type().display_name());
                                    if ui.text_edit_singleline(&mut new_name).changed() {
                                        pending_renames.push((index, new_name));
                                    }
                                    if ui.small_button("Delete").clicked() {
                                        pending_delete = Some(index);
                                    }
                                });
                                match &mut field.value {
                                    StateValue::Bool(value) => {
                                        changed |= ui.checkbox(value, "Default").changed();
                                    }
                                    StateValue::Text(value) => {
                                        changed |= text_property(ui, "Default", value);
                                    }
                                    StateValue::Number(value) => {
                                        ui.horizontal(|ui| {
                                            ui.label("Default");
                                            changed |=
                                                ui.add(egui::DragValue::new(value)).changed();
                                        });
                                    }
                                }
                            });
                        }
                        for (index, new_name) in pending_renames {
                            changed |= project.rename_state_field(index, new_name);
                        }
                        if let Some(index) = pending_delete {
                            changed |= project.remove_state_field(index);
                        }
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Add:");
                            for state_type in [StateType::Bool, StateType::Text, StateType::Number]
                            {
                                if ui.button(state_type.display_name()).clicked() {
                                    project.add_state_field(state_type);
                                    changed = true;
                                }
                            }
                        });
                    });

                egui::CollapsingHeader::new("Preview fixture")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.weak("Preview-only values never change generated defaults.");
                        changed |= text_property(ui, "Fixture name", &mut project.preview.name);
                        let fields = project
                            .state_fields
                            .iter()
                            .map(|field| (field.name.clone(), field.value.clone()))
                            .collect::<Vec<_>>();
                        for (name, default) in fields {
                            ui.horizontal(|ui| {
                                ui.monospace(&name);
                                match project.preview.values.get_mut(&name) {
                                    Some(StateValue::Bool(value))
                                        if matches!(default, StateValue::Bool(_)) =>
                                    {
                                        changed |= ui.checkbox(value, "").changed();
                                    }
                                    Some(StateValue::Text(value))
                                        if matches!(default, StateValue::Text(_)) =>
                                    {
                                        changed |= ui.text_edit_singleline(value).changed();
                                    }
                                    Some(StateValue::Number(value))
                                        if matches!(default, StateValue::Number(_)) =>
                                    {
                                        changed |= ui.add(egui::DragValue::new(value)).changed();
                                    }
                                    _ => {
                                        ui.colored_label(
                                            egui::Color32::LIGHT_RED,
                                            "missing or wrong type",
                                        );
                                        if ui.small_button("Repair").clicked() {
                                            project
                                                .preview
                                                .values
                                                .insert(name.clone(), default.clone());
                                            changed = true;
                                        }
                                    }
                                }
                            });
                        }
                        if ui.button("Reset fixture from state defaults").clicked() {
                            project.preview.values = project
                                .state_fields
                                .iter()
                                .map(|field| (field.name.clone(), field.value.clone()))
                                .collect();
                            changed = true;
                        }
                    });

                egui::CollapsingHeader::new("Messages & interactions")
                    .default_open(true)
                    .show(ui, |ui| {
                        changed |= edit_messages(ui, &mut project);
                        ui.separator();
                        changed |= edit_async_behavior(ui, &mut project);
                    });

                ui.separator();
                ui.heading("Selected node");
                let choices = project
                    .state_fields
                    .iter()
                    .map(|field| (field.name.clone(), field.state_type()))
                    .collect::<Vec<_>>();
                let mut delete_node = false;
                let mut interaction_events = Vec::new();
                if let Some(id) = selected {
                    let root_id = project.root.id;
                    if let Some(node) = project.find_node_mut(id) {
                        ui.label(format!("{} · {}", node.name, node.kind.display_name()));
                        changed |= text_property(ui, "Semantic name", &mut node.name);
                        ui.separator();
                        changed |= edit_node_kind(ui, id, &mut node.kind, &choices);
                        interaction_events = node.kind.supported_interactions().to_vec();
                        if node.kind.allows_children() {
                            ui.weak(format!("{} child nodes", node.children.len()));
                        }
                        if id != root_id && ui.button("Delete this node").clicked() {
                            delete_node = true;
                        }
                    } else {
                        ui.weak("The selected node no longer exists.");
                    }
                    if delete_node && project.remove_node(id) {
                        shared.selection.set(Some(root_id));
                        changed = true;
                    } else if !interaction_events.is_empty() {
                        ui.separator();
                        ui.strong("Interaction intents");
                        changed |= edit_interactions(ui, &mut project, id, &interaction_events);
                    }
                } else {
                    ui.weak("Select a node in the outline.");
                }

                ui.separator();
                egui::CollapsingHeader::new("Validation")
                    .default_open(true)
                    .show(ui, |ui| {
                        let diagnostics = project.validate();
                        if diagnostics.is_empty() {
                            ui.colored_label(egui::Color32::LIGHT_GREEN, "No diagnostics");
                        }
                        for diagnostic in diagnostics {
                            let color = match diagnostic.severity {
                                DiagnosticSeverity::Error => egui::Color32::LIGHT_RED,
                                DiagnosticSeverity::Warning => egui::Color32::YELLOW,
                            };
                            ui.colored_label(
                                color,
                                format!("{}: {}", diagnostic.path, diagnostic.message),
                            );
                        }
                    });
            });

        if changed {
            shared.commit_project(project, "Citizen design updated");
        }
    }
}

impl Citizen for InspectorPanel {
    fn id(&self) -> &CitizenId {
        &self.citizen_id
    }

    fn citizen_state(&self) -> &CitizenState {
        &self.citizen_state
    }

    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}

fn edit_theme(ui: &mut egui::Ui, project: &mut CitizenProject) -> bool {
    let before = project.theme.clone();
    ui.horizontal(|ui| {
        ui.label("Visual preset");
        egui::ComboBox::from_id_salt("theme_preset")
            .selected_text(project.theme.preset.display_name())
            .show_ui(ui, |ui| {
                for preset in [ThemePreset::Dark, ThemePreset::Light] {
                    ui.selectable_value(&mut project.theme.preset, preset, preset.display_name());
                }
            });
    });
    ui.horizontal(|ui| {
        ui.label("Accent");
        ui.color_edit_button_srgb(&mut project.theme.accent_rgb);
        ui.label("Panel");
        ui.color_edit_button_srgb(&mut project.theme.panel_rgb);
    });
    ui.horizontal(|ui| {
        ui.label("Item spacing");
        ui.add(egui::DragValue::new(&mut project.theme.item_spacing).range(0.0..=64.0));
    });
    project.theme != before
}

fn edit_assets(ui: &mut egui::Ui, project: &mut CitizenProject) -> bool {
    let mut changed = false;
    let mut pending_delete = None;
    for (index, asset) in project.assets.iter_mut().enumerate() {
        let before = asset.clone();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("File");
                ui.add(
                    egui::TextEdit::singleline(&mut asset.file_name)
                        .desired_width(170.0)
                        .hint_text("help.md or icon.svg"),
                );
                egui::ComboBox::from_id_salt(("asset_kind", index))
                    .selected_text(asset.kind.display_name())
                    .show_ui(ui, |ui| {
                        for kind in [AssetKind::Text, AssetKind::Svg] {
                            ui.selectable_value(&mut asset.kind, kind, kind.display_name());
                        }
                    });
                if ui.small_button("Delete").clicked() {
                    pending_delete = Some(index);
                }
            });
            ui.add(
                egui::TextEdit::multiline(&mut asset.contents)
                    .code_editor()
                    .desired_rows(5)
                    .desired_width(f32::INFINITY)
                    .hint_text("UTF-8 asset contents"),
            );
        });
        changed |= *asset != before;
    }
    if let Some(index) = pending_delete {
        project.assets.remove(index);
        changed = true;
    }
    if ui.button("Add text asset").clicked() {
        let mut sequence = project.assets.len() + 1;
        let file_name = loop {
            let candidate = format!("asset-{sequence}.txt");
            if project
                .assets
                .iter()
                .all(|asset| asset.file_name != candidate)
            {
                break candidate;
            }
            sequence += 1;
        };
        project.assets.push(AssetDefinition {
            file_name,
            kind: AssetKind::Text,
            contents: String::new(),
        });
        changed = true;
    }
    ui.weak(
        "Assets are copied beneath assets/ and exposed through generated include_str! constants.",
    );
    changed
}

fn edit_host_composition(ui: &mut egui::Ui, project: &mut CitizenProject) -> bool {
    let mut changed = ui
        .checkbox(
            &mut project.composition.enabled,
            "Author an external Citizen dock layout",
        )
        .changed();
    ui.weak("The generated library remains one Citizen; composition is host-only scaffolding.");

    let mut pending_delete = None;
    for (index, external) in project.composition.external_citizens.iter_mut().enumerate() {
        let before = external.clone();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong(format!("External {}", index + 1));
                if ui.small_button("Delete").clicked() {
                    pending_delete = Some(index);
                }
            });
            text_property(ui, "Cargo package", &mut external.crate_name);
            text_property(ui, "Citizen type", &mut external.citizen_type);
            text_property(ui, "CitizenId", &mut external.citizen_id);
            text_property(ui, "Dock title", &mut external.title);
            ui.horizontal(|ui| {
                ui.label("Placement");
                egui::ComboBox::from_id_salt(("host_placement", index))
                    .selected_text(external.placement.display_name())
                    .show_ui(ui, |ui| {
                        for placement in [
                            DockPlacement::Tab,
                            DockPlacement::Left,
                            DockPlacement::Right,
                            DockPlacement::Above,
                            DockPlacement::Below,
                        ] {
                            ui.selectable_value(
                                &mut external.placement,
                                placement,
                                placement.display_name(),
                            );
                        }
                    });
                if external.placement == DockPlacement::Tab {
                    ui.weak("same leaf");
                } else {
                    ui.label("fraction");
                    ui.add(egui::DragValue::new(&mut external.fraction).range(0.1..=0.9));
                }
            });
        });
        changed |= *external != before;
    }
    if let Some(index) = pending_delete {
        project.composition.external_citizens.remove(index);
        changed = true;
    }
    if ui.button("Add external Citizen").clicked() {
        let mut sequence = project.composition.external_citizens.len() + 1;
        while project
            .composition
            .external_citizens
            .iter()
            .any(|external| external.citizen_id == format!("external_{sequence}"))
        {
            sequence += 1;
        }
        project.composition.external_citizens.push(HostCitizen {
            crate_name: format!("external-citizen-{sequence}"),
            citizen_type: format!("ExternalCitizen{sequence}"),
            citizen_id: format!("external_{sequence}"),
            title: format!("External Citizen {sequence}"),
            placement: DockPlacement::Tab,
            fraction: 0.3,
        });
        changed = true;
    }

    if project.composition.enabled {
        ui.separator();
        ui.strong("Dock sketch");
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Primary · {}", project.title));
            for external in &project.composition.external_citizens {
                ui.label(format!(
                    "{} · {}",
                    external.placement.display_name(),
                    external.title
                ));
            }
        });
    }
    changed
}

fn edit_messages(ui: &mut egui::Ui, project: &mut CitizenProject) -> bool {
    let mut changed = false;
    let outcomes = project
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Outcome)
        .map(|message| message.key.clone())
        .collect::<Vec<_>>();
    let state_fields = project
        .state_fields
        .iter()
        .map(|field| (field.name.clone(), field.value.clone()))
        .collect::<Vec<_>>();
    let mut pending_delete = None;

    for index in 0..project.messages.len() {
        let original = project.messages[index].clone();
        let mut draft = original.clone();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                let role = match draft.role {
                    MessageRole::Intent => "Intent",
                    MessageRole::Outcome => "Outcome",
                };
                ui.strong(role);
                ui.monospace(draft.key.display_name());
                if ui.small_button("Delete").clicked() {
                    pending_delete = Some(index);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Domain");
                ui.add(egui::TextEdit::singleline(&mut draft.key.domain).desired_width(100.0));
                ui.label("Variant");
                ui.add(egui::TextEdit::singleline(&mut draft.key.variant).desired_width(160.0));
            });
            text_property(ui, "Documentation", &mut draft.description);

            match draft.role {
                MessageRole::Intent => {
                    ui.horizontal(|ui| {
                        ui.label("Reference outcome");
                        egui::ComboBox::from_id_salt(("message_outcome", index))
                            .selected_text(
                                draft
                                    .paired_outcome
                                    .as_ref()
                                    .map_or("None".to_owned(), MessageKey::display_name),
                            )
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut draft.paired_outcome, None, "None");
                                for outcome in &outcomes {
                                    ui.selectable_value(
                                        &mut draft.paired_outcome,
                                        Some(outcome.clone()),
                                        outcome.display_name(),
                                    );
                                }
                            });
                    });
                    ui.weak("Intents request work; they never write Dynamic<T> state directly.");
                }
                MessageRole::Outcome => {
                    ui.label("UI-thread state updates");
                    let mut remove_update = None;
                    for (update_index, update) in draft.state_updates.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            let old_field = update.field.clone();
                            egui::ComboBox::from_id_salt(("outcome_field", index, update_index))
                                .selected_text(&update.field)
                                .show_ui(ui, |ui| {
                                    for (name, _) in &state_fields {
                                        ui.selectable_value(&mut update.field, name.clone(), name);
                                    }
                                });
                            if update.field != old_field
                                && let Some((_, default)) =
                                    state_fields.iter().find(|(name, _)| name == &update.field)
                            {
                                update.value = default.clone();
                            }
                            edit_state_value(ui, &mut update.value);
                            if ui.small_button("Remove").clicked() {
                                remove_update = Some(update_index);
                            }
                        });
                    }
                    if let Some(update_index) = remove_update {
                        draft.state_updates.remove(update_index);
                    }
                    if let Some((name, default)) = state_fields.first()
                        && ui.small_button("Add state update").clicked()
                    {
                        draft.state_updates.push(StateAssignment {
                            field: name.clone(),
                            value: default.clone(),
                        });
                    }
                }
            }
        });

        if draft.key != original.key {
            if project.rename_message(index, draft.key.clone()) {
                changed = true;
            } else {
                draft.key = original.key;
            }
        }
        let message = &mut project.messages[index];
        if message.description != draft.description {
            message.description = draft.description;
            changed = true;
        }
        if message.paired_outcome != draft.paired_outcome {
            message.paired_outcome = draft.paired_outcome;
            changed = true;
        }
        if message.state_updates != draft.state_updates {
            message.state_updates = draft.state_updates;
            changed = true;
        }
    }

    if let Some(index) = pending_delete {
        changed |= project.remove_message(index);
    }
    ui.horizontal_wrapped(|ui| {
        ui.label("Add:");
        if ui.button("Intent").clicked() {
            project.add_message(MessageRole::Intent);
            changed = true;
        }
        if ui.button("Outcome").clicked() {
            project.add_message(MessageRole::Outcome);
            changed = true;
        }
    });
    changed
}

fn edit_interactions(
    ui: &mut egui::Ui,
    project: &mut CitizenProject,
    node: NodeId,
    events: &[InteractionEvent],
) -> bool {
    let intents = project
        .messages
        .iter()
        .filter(|message| message.role == MessageRole::Intent)
        .map(|message| message.key.clone())
        .collect::<Vec<_>>();
    let mut changed = false;
    for event in events {
        let before = project
            .interactions
            .iter()
            .find(|binding| binding.node == node && binding.event == *event)
            .map(|binding| binding.message.clone());
        let mut selected = before.clone();
        ui.horizontal(|ui| {
            ui.label(event.display_name());
            egui::ComboBox::from_id_salt(("interaction", node.0, *event))
                .selected_text(
                    selected
                        .as_ref()
                        .map_or("Unbound".to_owned(), MessageKey::display_name),
                )
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected, None, "Unbound");
                    for intent in &intents {
                        ui.selectable_value(
                            &mut selected,
                            Some(intent.clone()),
                            intent.display_name(),
                        );
                    }
                });
        });
        if selected != before {
            project.set_interaction(node, *event, selected);
            changed = true;
        }
    }
    changed
}

fn edit_async_behavior(ui: &mut egui::Ui, project: &mut CitizenProject) -> bool {
    let choices = project
        .messages
        .iter()
        .filter_map(|message| {
            if message.role == MessageRole::Intent {
                message
                    .paired_outcome
                    .as_ref()
                    .map(|outcome| (message.key.clone(), outcome.clone()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut changed = ui
        .checkbox(
            &mut project.async_behavior.enabled,
            "Generate Level 3 async backend",
        )
        .changed();
    ui.weak(
        "Native uses egui_mobius Signal/Slot + AsyncDispatcher; WASM uses abortable local futures.",
    );
    ui.weak(
        "Deactivation cancels pending work; outcomes are drained and applied on the UI thread.",
    );

    let mut pending_delete = None;
    for (index, mapping) in project.async_behavior.mappings.iter_mut().enumerate() {
        let before = mapping.clone();
        ui.group(|ui| {
            ui.horizontal(|ui| {
                egui::ComboBox::from_id_salt(("async_intent", index))
                    .selected_text(mapping.intent.display_name())
                    .show_ui(ui, |ui| {
                        for (intent, outcome) in &choices {
                            if ui
                                .selectable_value(
                                    &mut mapping.intent,
                                    intent.clone(),
                                    intent.display_name(),
                                )
                                .clicked()
                            {
                                mapping.outcome = outcome.clone();
                            }
                        }
                    });
                ui.label("→");
                ui.monospace(mapping.outcome.display_name());
                if ui.small_button("Remove").clicked() {
                    pending_delete = Some(index);
                }
            });
            ui.horizontal(|ui| {
                ui.label("Reference delay (ms)");
                changed |= ui
                    .add(egui::DragValue::new(&mut mapping.delay_ms).range(1..=600_000))
                    .changed();
            });
        });
        changed |= *mapping != before;
    }
    if let Some(index) = pending_delete {
        project.async_behavior.mappings.remove(index);
        changed = true;
    }
    if ui.button("Add async mapping").clicked() {
        changed |= project.add_async_mapping();
    }
    changed
}

fn edit_state_value(ui: &mut egui::Ui, value: &mut StateValue) {
    match value {
        StateValue::Bool(value) => {
            ui.checkbox(value, "");
        }
        StateValue::Text(value) => {
            ui.add(egui::TextEdit::singleline(value).desired_width(130.0));
        }
        StateValue::Number(value) => {
            ui.add(egui::DragValue::new(value));
        }
    }
}

fn text_property(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        changed = ui
            .add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY))
            .changed();
    });
    changed
}

fn edit_node_kind(
    ui: &mut egui::Ui,
    id: NodeId,
    kind: &mut NodeKind,
    fields: &[(String, StateType)],
) -> bool {
    let mut changed = false;
    match kind {
        NodeKind::Column | NodeKind::Separator => {}
        NodeKind::Row { wrap } => {
            changed |= ui.checkbox(wrap, "Wrap children").changed();
        }
        NodeKind::Grid { columns, striped } => {
            ui.horizontal(|ui| {
                ui.label("Columns");
                changed |= ui
                    .add(egui::DragValue::new(columns).range(1..=12))
                    .changed();
            });
            changed |= ui.checkbox(striped, "Striped rows").changed();
        }
        NodeKind::Group { title } => {
            changed |= text_property(ui, "Group title", title);
        }
        NodeKind::Scroll { max_height } => {
            ui.horizontal(|ui| {
                ui.label("Maximum height");
                changed |= ui
                    .add(egui::DragValue::new(max_height).range(0.0..=4000.0))
                    .changed();
            });
        }
        NodeKind::Label { text }
        | NodeKind::Heading { text }
        | NodeKind::Button { text }
        | NodeKind::StyledButton { text } => {
            changed |= text_property(ui, "Text", text);
        }
        NodeKind::ReactiveLogger => {
            ui.weak("State and colors are generated as internal egui_lens Dynamic<T> handles.");
        }
        NodeKind::ReactiveEditor { content, language } => {
            changed |= text_property(ui, "Language", language);
            ui.label("Initial content");
            changed |= ui
                .add(
                    egui::TextEdit::multiline(content)
                        .code_editor()
                        .desired_rows(8)
                        .desired_width(f32::INFINITY),
                )
                .changed();
        }
        NodeKind::LinePlot { binding } => {
            changed |= binding_picker(ui, id, binding, StateType::Number, fields);
        }
        NodeKind::Checkbox { text, binding } => {
            changed |= text_property(ui, "Text", text);
            changed |= binding_picker(ui, id, binding, StateType::Bool, fields);
        }
        NodeKind::TextEdit {
            label,
            hint,
            binding,
        } => {
            changed |= text_property(ui, "Label", label);
            changed |= text_property(ui, "Hint", hint);
            changed |= binding_picker(ui, id, binding, StateType::Text, fields);
        }
        NodeKind::Slider {
            label,
            min,
            max,
            binding,
        } => {
            changed |= text_property(ui, "Label", label);
            ui.horizontal(|ui| {
                ui.label("Range");
                changed |= ui.add(egui::DragValue::new(min)).changed();
                changed |= ui.add(egui::DragValue::new(max)).changed();
            });
            changed |= binding_picker(ui, id, binding, StateType::Number, fields);
        }
        NodeKind::ProgressBar {
            binding,
            show_percentage,
        } => {
            changed |= ui.checkbox(show_percentage, "Show percentage").changed();
            changed |= binding_picker(ui, id, binding, StateType::Number, fields);
        }
        NodeKind::Spacer { points } => {
            ui.horizontal(|ui| {
                ui.label("Points");
                changed |= ui
                    .add(egui::DragValue::new(points).range(0.0..=1000.0))
                    .changed();
            });
        }
    }
    changed
}

fn binding_picker(
    ui: &mut egui::Ui,
    node_id: NodeId,
    binding: &mut Option<String>,
    expected: StateType,
    fields: &[(String, StateType)],
) -> bool {
    let before = binding.clone();
    ui.horizontal(|ui| {
        ui.label(format!("Dynamic<{}>", expected.display_name()));
        egui::ComboBox::from_id_salt(("binding", node_id.0))
            .selected_text(binding.as_deref().unwrap_or("Unbound"))
            .show_ui(ui, |ui| {
                ui.selectable_value(binding, None, "Unbound");
                for (name, state_type) in fields {
                    if *state_type == expected {
                        ui.selectable_value(binding, Some(name.clone()), name);
                    }
                }
            });
    });
    *binding != before
}
