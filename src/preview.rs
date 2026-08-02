//! Live semantic preview used by the Citizen canvas panel.

use std::collections::HashMap;

use egui::{Color32, CornerRadius, Stroke};

use crate::model::{CitizenProject, DesignNode, NodeId, NodeKind, StateValue};

#[derive(Clone, Debug)]
enum PreviewValue {
    Bool(bool),
    Text(String),
    Number(f32),
}

/// Mutable preview values corresponding to generated `Dynamic<T>` fields.
#[derive(Default)]
pub(crate) struct PreviewState {
    values: HashMap<String, PreviewValue>,
}

impl PreviewState {
    /// Reconcile preview values with the project's typed state contract.
    pub(crate) fn sync(&mut self, project: &CitizenProject) {
        self.values
            .retain(|name, _| project.state_fields.iter().any(|field| field.name == *name));
        for field in &project.state_fields {
            let matches = matches!(
                (self.values.get(&field.name), &field.value),
                (Some(PreviewValue::Bool(_)), StateValue::Bool(_))
                    | (Some(PreviewValue::Text(_)), StateValue::Text(_))
                    | (Some(PreviewValue::Number(_)), StateValue::Number(_))
            );
            if !matches {
                let value = match &field.value {
                    StateValue::Bool(value) => PreviewValue::Bool(*value),
                    StateValue::Text(value) => PreviewValue::Text(value.clone()),
                    StateValue::Number(value) => PreviewValue::Number(*value),
                };
                self.values.insert(field.name.clone(), value);
            }
        }
    }

    /// Render the complete semantic tree and outline the selected node.
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        project: &CitizenProject,
        selected: Option<NodeId>,
    ) {
        self.sync(project);
        self.show_node(ui, &project.root, selected);
    }

    fn show_node(&mut self, ui: &mut egui::Ui, node: &DesignNode, selected: Option<NodeId>) {
        let response = ui
            .scope(|ui| match &node.kind {
                NodeKind::Column => {
                    ui.vertical(|ui| self.show_children(ui, node, selected));
                }
                NodeKind::Row { wrap } => {
                    if *wrap {
                        ui.horizontal_wrapped(|ui| self.show_children(ui, node, selected));
                    } else {
                        ui.horizontal(|ui| self.show_children(ui, node, selected));
                    }
                }
                NodeKind::Grid { columns, striped } => {
                    egui::Grid::new(("preview_grid", node.id.0))
                        .num_columns((*columns).max(1))
                        .striped(*striped)
                        .show(ui, |ui| {
                            for (index, child) in node.children.iter().enumerate() {
                                self.show_node(ui, child, selected);
                                if (index + 1) % (*columns).max(1) == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                }
                NodeKind::Group { title } => {
                    ui.group(|ui| {
                        if !title.is_empty() {
                            ui.strong(title);
                            ui.separator();
                        }
                        self.show_children(ui, node, selected);
                    });
                }
                NodeKind::Scroll { max_height } => {
                    let scroll =
                        egui::ScrollArea::vertical().id_salt(("preview_scroll", node.id.0));
                    if *max_height > 0.0 {
                        scroll
                            .max_height(*max_height)
                            .show(ui, |ui| self.show_children(ui, node, selected));
                    } else {
                        scroll.show(ui, |ui| self.show_children(ui, node, selected));
                    }
                }
                NodeKind::Label { text } => {
                    ui.label(text);
                }
                NodeKind::Heading { text } => {
                    ui.heading(text);
                }
                NodeKind::Button { text } => {
                    let _ = ui.button(text);
                }
                NodeKind::Checkbox { text, binding } => {
                    if let Some(PreviewValue::Bool(value)) =
                        binding.as_ref().and_then(|name| self.values.get_mut(name))
                    {
                        ui.checkbox(value, text);
                    } else {
                        ui.colored_label(Color32::LIGHT_RED, "Missing bool binding");
                    }
                }
                NodeKind::TextEdit {
                    label,
                    hint,
                    binding,
                } => {
                    if let Some(PreviewValue::Text(value)) =
                        binding.as_ref().and_then(|name| self.values.get_mut(name))
                    {
                        ui.horizontal(|ui| {
                            if !label.is_empty() {
                                ui.label(label);
                            }
                            ui.add(egui::TextEdit::singleline(value).hint_text(hint));
                        });
                    } else {
                        ui.colored_label(Color32::LIGHT_RED, "Missing String binding");
                    }
                }
                NodeKind::Slider {
                    label,
                    min,
                    max,
                    binding,
                } => {
                    if let Some(PreviewValue::Number(value)) =
                        binding.as_ref().and_then(|name| self.values.get_mut(name))
                    {
                        ui.add(egui::Slider::new(value, *min..=*max).text(label));
                    } else {
                        ui.colored_label(Color32::LIGHT_RED, "Missing f32 binding");
                    }
                }
                NodeKind::ProgressBar {
                    binding,
                    show_percentage,
                } => {
                    if let Some(PreviewValue::Number(value)) =
                        binding.as_ref().and_then(|name| self.values.get(name))
                    {
                        let progress = egui::ProgressBar::new(value.clamp(0.0, 1.0));
                        if *show_percentage {
                            ui.add(progress.show_percentage());
                        } else {
                            ui.add(progress);
                        }
                    } else {
                        ui.colored_label(Color32::LIGHT_RED, "Missing f32 binding");
                    }
                }
                NodeKind::Separator => {
                    ui.separator();
                }
                NodeKind::Spacer { points } => {
                    ui.add_space(*points);
                }
            })
            .response;

        if selected == Some(node.id) {
            ui.painter().rect_stroke(
                response.rect.expand(3.0),
                CornerRadius::same(3),
                Stroke::new(1.5, Color32::from_rgb(95, 170, 255)),
                egui::StrokeKind::Outside,
            );
        }
    }

    fn show_children(&mut self, ui: &mut egui::Ui, node: &DesignNode, selected: Option<NodeId>) {
        for child in &node.children {
            self.show_node(ui, child, selected);
        }
    }
}
