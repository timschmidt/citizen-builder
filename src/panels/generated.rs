use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::generator::generate;
use crate::model::DiagnosticSeverity;

/// Generated file-set preview and safe export Citizen.
pub(crate) struct GeneratedPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    selected_file: String,
}

impl GeneratedPanel {
    pub(crate) const ID: &'static str = "builder_generated";

    pub(crate) fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(Self::ID),
            citizen_state,
            selected_file: "src/lib.rs".to_owned(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, shared: &BuilderSharedState) {
        let project = shared.project.get();
        ui.horizontal(|ui| {
            ui.heading("Generated Citizen crate");
            ui.monospace(&project.crate_name);
        });

        let diagnostics = project.validate();
        for diagnostic in diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Warning)
        {
            ui.colored_label(
                egui::Color32::YELLOW,
                format!("{}: {}", diagnostic.path, diagnostic.message),
            );
        }

        let generated = match generate(&project) {
            Ok(generated) => generated,
            Err(diagnostics) => {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    "Resolve validation errors before generation.",
                );
                for diagnostic in diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
                {
                    ui.label(format!("{}: {}", diagnostic.path, diagnostic.message));
                }
                return;
            }
        };

        if !generated.files.contains_key(&self.selected_file) {
            self.selected_file = "src/lib.rs".to_owned();
        }

        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("generated_file_picker")
                .selected_text(&self.selected_file)
                .show_ui(ui, |ui| {
                    for path in generated.files.keys() {
                        ui.selectable_value(&mut self.selected_file, path.clone(), path);
                    }
                });
            if ui.button("Copy file").clicked()
                && let Some(contents) = generated.file(&self.selected_file)
            {
                ui.ctx().copy_text(contents.to_owned());
                shared.set_status(format!("Copied {}", self.selected_file));
            }

            #[cfg(not(target_arch = "wasm32"))]
            if ui.button("Export new crate…").clicked()
                && let Some(parent) = rfd::FileDialog::new().pick_folder()
            {
                match generated.write_new(&parent) {
                    Ok(path) => shared.set_status(format!("Exported {}", path.display())),
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                        shared.set_status(format!(
                            "Refused to overwrite existing {}/{}",
                            parent.display(),
                            generated.directory_name
                        ));
                    }
                    Err(error) => shared.set_status(format!("Export failed: {error}")),
                }
            }

            #[cfg(target_arch = "wasm32")]
            ui.weak("Copy generated files individually from the browser preview.");
        });
        ui.separator();

        if let Some(contents) = generated.file(&self.selected_file) {
            let mut visible = contents.to_owned();
            egui::ScrollArea::both()
                .id_salt("generated_source_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut visible)
                            .code_editor()
                            .interactive(false)
                            .desired_width(f32::INFINITY),
                    );
                });
        }
    }
}

impl Citizen for GeneratedPanel {
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
