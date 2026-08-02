use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::DiagnosticSeverity;
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
        });
        ui.label(
            "Live Level 1 preview — controls mutate preview values matching Dynamic<T> fields.",
        );
        ui.separator();

        egui::ScrollArea::both()
            .id_salt("citizen_canvas_scroll")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                egui::Frame::group(ui.style())
                    .inner_margin(egui::Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_min_size(egui::vec2(620.0, 420.0));
                        self.preview.show(ui, &project, selected);
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
