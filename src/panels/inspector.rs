use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::{DiagnosticSeverity, FrameworkSource, NodeId, NodeKind, StateType, StateValue};

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

                ui.separator();
                ui.heading("Selected node");
                let choices = project
                    .state_fields
                    .iter()
                    .map(|field| (field.name.clone(), field.state_type()))
                    .collect::<Vec<_>>();
                let mut delete_node = false;
                if let Some(id) = selected {
                    let root_id = project.root.id;
                    if let Some(node) = project.find_node_mut(id) {
                        ui.label(format!("{} · {}", node.name, node.kind.display_name()));
                        changed |= text_property(ui, "Semantic name", &mut node.name);
                        ui.separator();
                        changed |= edit_node_kind(ui, id, &mut node.kind, &choices);
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
            shared.project.set(project);
            shared.set_status("Citizen design updated");
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
        NodeKind::Label { text } | NodeKind::Heading { text } | NodeKind::Button { text } => {
            changed |= text_property(ui, "Text", text);
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
