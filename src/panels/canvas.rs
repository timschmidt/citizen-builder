use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::{CitizenProject, DiagnosticSeverity, DockPlacement, ThemePreset};
use crate::preview::PreviewState;

/// Live semantic preview Citizen.
pub(crate) struct CanvasPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    preview: PreviewState,
}

impl CanvasPanel {
    pub(crate) const ID: &'static str = "builder_canvas";

    pub(crate) fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(Self::ID),
            citizen_state,
            preview: PreviewState::default(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, shared: &BuilderSharedState) {
        let project = shared.project.get();
        let selected = shared.selection.get();
        let diagnostics = project.validate();
        let errors = diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
            .count();
        let warnings = diagnostics.len().saturating_sub(errors);

        ui.horizontal(|ui| {
            ui.heading(&project.title);
            ui.monospace(format!("CitizenId: {}", project.citizen_id));
            if errors == 0 {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "generation ready");
            } else {
                ui.colored_label(egui::Color32::LIGHT_RED, format!("{errors} errors"));
            }
            if warnings > 0 {
                ui.colored_label(egui::Color32::YELLOW, format!("{warnings} warnings"));
            }
            if ui.button("Reset fixture").clicked() {
                self.preview.reset();
                shared.set_status("Reset preview to saved fixture values");
            }
        });
        ui.label(
            "Live Citizen preview — controls mutate fixture values matching Dynamic<T> fields.",
        );
        if project.composition.enabled {
            host_composition_sketch(ui, &project);
        }
        ui.separator();

        egui::ScrollArea::both()
            .id_salt("citizen_canvas_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.scope(|ui| {
                    let mut visuals = match project.theme.preset {
                        ThemePreset::Dark => egui::Visuals::dark(),
                        ThemePreset::Light => egui::Visuals::light(),
                    };
                    visuals.panel_fill = egui::Color32::from_rgb(
                        project.theme.panel_rgb[0],
                        project.theme.panel_rgb[1],
                        project.theme.panel_rgb[2],
                    );
                    visuals.selection.bg_fill = egui::Color32::from_rgb(
                        project.theme.accent_rgb[0],
                        project.theme.accent_rgb[1],
                        project.theme.accent_rgb[2],
                    );
                    *ui.visuals_mut() = visuals;
                    ui.spacing_mut().item_spacing =
                        egui::vec2(project.theme.item_spacing, project.theme.item_spacing);
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().panel_fill)
                        .inner_margin(egui::Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_min_size(egui::vec2(620.0, 420.0));
                            self.preview.show(ui, &project, selected);
                        });
                });
            });

        if let Some(id) = selected
            && let Some(node) = project.find_node(id)
        {
            ui.separator();
            ui.weak(format!(
                "Selected: {} ({})",
                node.name,
                node.kind.display_name()
            ));
        }
    }
}

fn host_composition_sketch(ui: &mut egui::Ui, project: &CitizenProject) {
    ui.group(|ui| {
        ui.horizontal(|ui| {
            ui.strong("Host dock sketch");
            ui.weak("external Citizens render as compile-time placeholders");
        });
        egui::Grid::new("host_composition_sketch")
            .num_columns(3)
            .spacing([8.0, 6.0])
            .show(ui, |ui| {
                ui.label("");
                dock_placeholders(ui, project, DockPlacement::Above);
                ui.label("");
                ui.end_row();

                dock_placeholders(ui, project, DockPlacement::Left);
                ui.vertical_centered(|ui| {
                    ui.strong(format!("Primary · {}", project.title));
                    dock_placeholders(ui, project, DockPlacement::Tab);
                });
                dock_placeholders(ui, project, DockPlacement::Right);
                ui.end_row();

                ui.label("");
                dock_placeholders(ui, project, DockPlacement::Below);
                ui.label("");
                ui.end_row();
            });
    });
}

fn dock_placeholders(ui: &mut egui::Ui, project: &CitizenProject, placement: DockPlacement) {
    ui.vertical(|ui| {
        for external in project
            .composition
            .external_citizens
            .iter()
            .filter(|external| external.placement == placement)
        {
            ui.label(format!("{} · {}", placement.display_name(), external.title));
        }
    });
}

impl Citizen for CanvasPanel {
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
