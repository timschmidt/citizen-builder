//! Dogfooded egui_citizen application shell.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_mobius_reactive::Dynamic;

use crate::model::{CitizenProject, NodeId};
use crate::panels::{CanvasPanel, GeneratedPanel, InspectorPanel, OutlinePanel};

/// Reactive state shared by all builder Citizens.
#[derive(Clone)]
pub(crate) struct BuilderSharedState {
    pub(crate) project: Dynamic<CitizenProject>,
    pub(crate) selection: Dynamic<Option<NodeId>>,
    status: Dynamic<String>,
}

impl BuilderSharedState {
    fn new(project: CitizenProject) -> Self {
        let root_id = project.root.id;
        Self {
            project: Dynamic::new(project),
            selection: Dynamic::new(Some(root_id)),
            status: Dynamic::new("Citizen-first workspace ready".to_owned()),
        }
    }

    pub(crate) fn set_status(&self, message: impl Into<String>) {
        self.status.set(message.into());
    }
}

#[derive(Clone, Copy)]
enum BuilderTabKind {
    Outline,
    Canvas,
    Inspector,
    Generated,
}

#[derive(Clone)]
struct BuilderTab {
    kind: BuilderTabKind,
}

impl BuilderTab {
    const fn new(kind: BuilderTabKind) -> Self {
        Self { kind }
    }

    const fn title(&self) -> &'static str {
        match self.kind {
            BuilderTabKind::Outline => "Outline + Palette",
            BuilderTabKind::Canvas => "Citizen Preview",
            BuilderTabKind::Inspector => "Inspector",
            BuilderTabKind::Generated => "Generated Crate",
        }
    }

    fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            BuilderTabKind::Outline => OutlinePanel::ID,
            BuilderTabKind::Canvas => CanvasPanel::ID,
            BuilderTabKind::Inspector => InspectorPanel::ID,
            BuilderTabKind::Generated => GeneratedPanel::ID,
        })
    }
}

struct BuilderTabViewer<'a> {
    shared: &'a BuilderSharedState,
    dispatcher: &'a mut Dispatcher,
    outline: &'a mut OutlinePanel,
    canvas: &'a mut CanvasPanel,
    inspector: &'a mut InspectorPanel,
    generated: &'a mut GeneratedPanel,
}

impl egui_dock::TabViewer for BuilderTabViewer<'_> {
    type Tab = BuilderTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            BuilderTabKind::Outline => self.outline.show(ui, self.shared),
            BuilderTabKind::Canvas => self.canvas.show(ui, self.shared),
            BuilderTabKind::Inspector => self.inspector.show(ui, self.shared),
            BuilderTabKind::Generated => self.generated.show(ui, self.shared),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }
}

/// Visual editor for one standalone Level 1 egui_mobius Citizen crate.
pub struct CitizenBuilderApp {
    shared: BuilderSharedState,
    dispatcher: Dispatcher,
    dock_state: DockState<BuilderTab>,
    outline: OutlinePanel,
    canvas: CanvasPanel,
    inspector: InspectorPanel,
    generated: GeneratedPanel,
    import_json_open: bool,
    import_json: String,
    #[cfg(not(target_arch = "wasm32"))]
    current_file: Option<PathBuf>,
}

impl CitizenBuilderApp {
    /// Construct the dogfooded docked Citizen workspace.
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        configure_visuals(&creation_context.egui_ctx);

        let project = CitizenProject::default();
        let shared = BuilderSharedState::new(project);
        let mut dispatcher = Dispatcher::new();
        let outline_state = dispatcher.register(CitizenId::new(OutlinePanel::ID));
        let canvas_state = dispatcher.register(CitizenId::new(CanvasPanel::ID));
        let inspector_state = dispatcher.register(CitizenId::new(InspectorPanel::ID));
        let generated_state = dispatcher.register(CitizenId::new(GeneratedPanel::ID));
        dispatcher.activate(&CitizenId::new(CanvasPanel::ID));
        let _ = dispatcher.drain_messages();

        let mut dock_state = DockState::new(vec![
            BuilderTab::new(BuilderTabKind::Canvas),
            BuilderTab::new(BuilderTabKind::Generated),
        ]);
        let [_left, center] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.22,
            vec![BuilderTab::new(BuilderTabKind::Outline)],
        );
        let [_center, _right] = dock_state.main_surface_mut().split_right(
            center,
            0.70,
            vec![BuilderTab::new(BuilderTabKind::Inspector)],
        );

        Self {
            shared,
            dispatcher,
            dock_state,
            outline: OutlinePanel::new(outline_state),
            canvas: CanvasPanel::new(canvas_state),
            inspector: InspectorPanel::new(inspector_state),
            generated: GeneratedPanel::new(generated_state),
            import_json_open: false,
            import_json: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            current_file: None,
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.strong("citizen-builder");
            if ui.button("New Citizen").clicked() {
                self.replace_project(CitizenProject::default());
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.current_file = None;
                }
                self.shared.set_status("Created a new Citizen design");
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                if ui.button("Open…").clicked() {
                    self.open_project();
                }
                if ui.button("Save").clicked() {
                    self.save_project(false);
                }
                if ui.button("Save As…").clicked() {
                    self.save_project(true);
                }
            }

            if ui.button("Import JSON").clicked() {
                self.import_json = self
                    .shared
                    .project
                    .get()
                    .to_json_pretty()
                    .unwrap_or_default();
                self.import_json_open = true;
            }
            if ui.button("Copy project JSON").clicked()
                && let Ok(json) = self.shared.project.get().to_json_pretty()
            {
                ui.ctx().copy_text(json);
                self.shared.set_status("Copied Citizen project JSON");
            }

            ui.separator();
            let project = self.shared.project.get();
            let error_count = project
                .validate()
                .iter()
                .filter(|diagnostic| diagnostic.severity == crate::model::DiagnosticSeverity::Error)
                .count();
            if error_count == 0 {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "Level 1 ready");
            } else {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("{error_count} validation errors"),
                );
            }
            ui.weak(self.shared.status.get());
        });
    }

    fn import_editor(&mut self, ui: &mut egui::Ui) {
        if !self.import_json_open {
            return;
        }
        let mut apply = false;
        let mut cancel = false;
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.strong("Import current Citizen schema JSON");
                ui.weak("Only the current Citizen schema is accepted.");
            });
            ui.add(
                egui::TextEdit::multiline(&mut self.import_json)
                    .code_editor()
                    .desired_rows(10)
                    .desired_width(f32::INFINITY),
            );
            ui.horizontal(|ui| {
                apply = ui.button("Apply").clicked();
                cancel = ui.button("Cancel").clicked();
            });
        });
        if apply {
            match CitizenProject::from_json(&self.import_json) {
                Ok(project) => {
                    self.replace_project(project);
                    self.shared.set_status("Imported Citizen project JSON");
                    self.import_json_open = false;
                }
                Err(error) => self.shared.set_status(format!("Import failed: {error}")),
            }
        } else if cancel {
            self.import_json_open = false;
        }
    }

    fn replace_project(&mut self, project: CitizenProject) {
        let root_id = project.root.id;
        self.shared.project.set(project);
        self.shared.selection.set(Some(root_id));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn open_project(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Citizen project", &["json"])
            .pick_file()
        else {
            return;
        };
        match std::fs::read_to_string(&path)
            .map_err(|error| error.to_string())
            .and_then(|json| CitizenProject::from_json(&json))
        {
            Ok(project) => {
                self.replace_project(project);
                self.current_file = Some(path.clone());
                self.shared.set_status(format!("Opened {}", path.display()));
            }
            Err(error) => self.shared.set_status(format!("Open failed: {error}")),
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_project(&mut self, force_dialog: bool) {
        let project = self.shared.project.get();
        let path = if !force_dialog {
            self.current_file.clone()
        } else {
            None
        }
        .or_else(|| {
            rfd::FileDialog::new()
                .add_filter("Citizen project", &["json"])
                .set_file_name(format!("{}.citizen.json", project.crate_name))
                .save_file()
        });
        let Some(path) = path else {
            return;
        };
        match project
            .to_json_pretty()
            .map_err(|error| error.to_string())
            .and_then(|json| std::fs::write(&path, json).map_err(|error| error.to_string()))
        {
            Ok(()) => {
                self.current_file = Some(path.clone());
                self.shared.set_status(format!("Saved {}", path.display()));
            }
            Err(error) => self.shared.set_status(format!("Save failed: {error}")),
        }
    }
}

impl eframe::App for CitizenBuilderApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.toolbar(ui);
        self.import_editor(ui);
        ui.separator();

        DockArea::new(&mut self.dock_state).show_inside(
            ui,
            &mut BuilderTabViewer {
                shared: &self.shared,
                dispatcher: &mut self.dispatcher,
                outline: &mut self.outline,
                canvas: &mut self.canvas,
                inspector: &mut self.inspector,
                generated: &mut self.generated,
            },
        );

        let _messages = self.dispatcher.drain_messages();
    }
}

fn configure_visuals(context: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = egui::Color32::from_rgb(25, 28, 34);
    visuals.extreme_bg_color = egui::Color32::from_rgb(17, 19, 24);
    visuals.selection.bg_fill = egui::Color32::from_rgb(45, 100, 165);
    context.set_visuals(visuals);
}
