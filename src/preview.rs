//! Live semantic preview used by the Citizen canvas panel.

use std::collections::HashMap;

use egui::{Color32, CornerRadius, Stroke};

use crate::model::{CitizenProject, DesignNode, NodeId, NodeKind, StateValue};

#[derive(Clone, Debug, PartialEq)]
enum PreviewValue {
    Bool(bool),
    Text(String),
    Number(f32),
}

/// Mutable preview values corresponding to generated `Dynamic<T>` fields.
#[derive(Default)]
pub(crate) struct PreviewState {
    values: HashMap<String, PreviewValue>,
    fixture_values: HashMap<String, StateValue>,
}

impl PreviewState {
    /// Reconcile preview values with the project's typed state contract.
    pub(crate) fn sync(&mut self, project: &CitizenProject) {
        self.values
            .retain(|name, _| project.state_fields.iter().any(|field| field.name == *name));
        self.fixture_values
            .retain(|name, _| project.state_fields.iter().any(|field| field.name == *name));
        for field in &project.state_fields {
            let fixture = project
                .preview
                .values
                .get(&field.name)
                .unwrap_or(&field.value);
            let type_matches = matches!(
                (self.values.get(&field.name), fixture),
                (Some(PreviewValue::Bool(_)), StateValue::Bool(_))
                    | (Some(PreviewValue::Text(_)), StateValue::Text(_))
                    | (Some(PreviewValue::Number(_)), StateValue::Number(_))
            );
            let fixture_changed = self.fixture_values.get(&field.name) != Some(fixture);
            if !type_matches || fixture_changed {
                let value = match fixture {
                    StateValue::Bool(value) => PreviewValue::Bool(*value),
                    StateValue::Text(value) => PreviewValue::Text(value.clone()),
                    StateValue::Number(value) => PreviewValue::Number(*value),
                };
                self.values.insert(field.name.clone(), value);
            }
            self.fixture_values
                .insert(field.name.clone(), fixture.clone());
        }
    }

    /// Reset interactive values to the saved preview fixture.
    pub(crate) fn reset(&mut self) {
        self.values.clear();
        self.fixture_values.clear();
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
                NodeKind::StyledButton { text } => {
                    let _ = ui.add(
                        egui::Button::new(text)
                            .fill(Color32::from_rgb(70, 105, 145))
                            .corner_radius(6.0),
                    );
                }
                NodeKind::ReactiveLogger => {
                    ui.group(|ui| {
                        ui.strong("egui_lens event log");
                        ui.monospace("[intent] preview::ActionRequested");
                        ui.monospace("[outcome] preview::ActionCompleted");
                    });
                }
                NodeKind::ReactiveEditor { content, language } => {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.strong("egui_quill editor");
                            ui.monospace(language);
                        });
                        let mut visible = content.clone();
                        ui.add(
                            egui::TextEdit::multiline(&mut visible)
                                .code_editor()
                                .interactive(false)
                                .desired_rows(8)
                                .desired_width(f32::INFINITY),
                        );
                    });
                }
                NodeKind::LinePlot { binding } => {
                    if let Some(PreviewValue::Number(value)) =
                        binding.as_ref().and_then(|name| self.values.get(name))
                    {
                        ui.group(|ui| {
                            ui.strong("egui_plot line preview");
                            ui.add(
                                egui::ProgressBar::new((value / 4.0).clamp(0.0, 1.0))
                                    .text(format!("amplitude {value:.2}")),
                            );
                        });
                    } else {
                        ui.colored_label(Color32::LIGHT_RED, "Missing f32 binding");
                    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_fixture_initializes_and_updates_preview_values() {
        let mut state = PreviewState::default();
        let mut project = CitizenProject::default();
        state.sync(&project);
        assert_eq!(
            state.values.get("display_name"),
            Some(&PreviewValue::Text("Preview Citizen".to_owned()))
        );

        project.preview.values.insert(
            "display_name".to_owned(),
            StateValue::Text("Edited fixture".to_owned()),
        );
        state.sync(&project);
        assert_eq!(
            state.values.get("display_name"),
            Some(&PreviewValue::Text("Edited fixture".to_owned()))
        );
        assert_eq!(
            project.state_fields[1].value,
            StateValue::Text("Citizen".to_owned())
        );
    }
}
