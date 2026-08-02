//! Dogfooded egui_citizen application shell.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_mobius_reactive::Dynamic;

use crate::model::{CitizenProject, CitizenTemplate, NodeId};
use crate::panels::{CanvasPanel, GeneratedPanel, InspectorPanel, OutlinePanel};

/// Reactive state shared by all builder Citizens.
#[derive(Clone)]
pub(crate) struct BuilderSharedState {
    pub(crate) project: Dynamic<CitizenProject>,
    pub(crate) selection: Dynamic<Option<NodeId>>,
    status: Dynamic<String>,
    history: Dynamic<DocumentHistory>,
}

#[derive(Clone)]
struct HistoryEntry {
    project: CitizenProject,
    selection: Option<NodeId>,
}

#[derive(Clone, Default)]
struct DocumentHistory {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
}

const HISTORY_LIMIT: usize = 100;

impl BuilderSharedState {
    fn new(project: CitizenProject) -> Self {
        let root_id = project.root.id;
        Self {
            project: Dynamic::new(project),
            selection: Dynamic::new(Some(root_id)),
            status: Dynamic::new("Citizen-first workspace ready".to_owned()),
            history: Dynamic::new(DocumentHistory::default()),
        }
    }

    pub(crate) fn set_status(&self, message: impl Into<String>) {
        self.status.set(message.into());
    }

    pub(crate) fn commit_project(
        &self,
        project: CitizenProject,
        message: impl Into<String>,
    ) -> bool {
        let current = self.project.get();
        if current == project {
            return false;
        }
        let mut history = self.history.get();
        history.undo.push(HistoryEntry {
            project: current,
            selection: self.selection.get(),
        });
        if history.undo.len() > HISTORY_LIMIT {
            history.undo.remove(0);
        }
        history.redo.clear();
        self.project.set(project.clone());
        if self
            .selection
            .get()
            .is_none_or(|selection| project.find_node(selection).is_none())
        {
            self.selection.set(Some(project.root.id));
        }
        self.history.set(history);
        self.set_status(message);
        true
    }

    fn can_undo(&self) -> bool {
        !self.history.get().undo.is_empty()
    }

    fn can_redo(&self) -> bool {
        !self.history.get().redo.is_empty()
    }

    fn undo(&self) -> bool {
        let mut history = self.history.get();
        let Some(previous) = history.undo.pop() else {
            return false;
        };
        history.redo.push(HistoryEntry {
            project: self.project.get(),
            selection: self.selection.get(),
        });
        self.project.set(previous.project);
        self.selection.set(previous.selection);
        self.history.set(history);
        self.set_status("Undid document change");
        true
    }

    fn redo(&self) -> bool {
        let mut history = self.history.get();
        let Some(next) = history.redo.pop() else {
            return false;
        };
        history.undo.push(HistoryEntry {
            project: self.project.get(),
            selection: self.selection.get(),
        });
        self.project.set(next.project);
        self.selection.set(next.selection);
        self.history.set(history);
        self.set_status("Redid document change");
        true
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

/// Visual editor for one standalone egui_mobius Citizen crate.
pub struct CitizenBuilderApp {
    shared: BuilderSharedState,
    dispatcher: Dispatcher,
    dock_state: DockState<BuilderTab>,
    outline: OutlinePanel,
    canvas: CanvasPanel,
    inspector: InspectorPanel,
    generated: GeneratedPanel,
    new_template: CitizenTemplate,
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
            new_template: CitizenTemplate::default(),
            import_json_open: false,
            import_json: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            current_file: None,
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.strong("citizen-builder");
            egui::ComboBox::from_id_salt("new_citizen_template")
                .selected_text(self.new_template.display_name())
                .show_ui(ui, |ui| {
                    for template in CitizenTemplate::ALL {
                        ui.selectable_value(
                            &mut self.new_template,
                            *template,
                            template.display_name(),
                        )
                        .on_hover_text(template.description());
                    }
                });
            if ui.button("New Citizen").clicked() {
                self.replace_project(CitizenProject::from_template(self.new_template));
                #[cfg(not(target_arch = "wasm32"))]
                {
                    self.current_file = None;
                }
                self.shared.set_status(format!(
                    "Created a {} Citizen",
                    self.new_template.display_name()
                ));
            }
            if ui
                .add_enabled(self.shared.can_undo(), egui::Button::new("Undo"))
                .on_hover_text("Undo document change (Ctrl/Cmd+Z)")
                .clicked()
            {
                self.shared.undo();
            }
            if ui
                .add_enabled(self.shared.can_redo(), egui::Button::new("Redo"))
                .on_hover_text("Redo document change (Ctrl/Cmd+Y or Cmd+Shift+Z)")
                .clicked()
            {
                self.shared.redo();
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
                let features = project.inferred_features();
                let summary = if features.is_empty() {
                    "generation ready".to_owned()
                } else {
                    format!("ready · {} features", features.len())
                };
                ui.colored_label(egui::Color32::LIGHT_GREEN, summary);
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
        if self
            .shared
            .commit_project(project, "Replaced Citizen document")
        {
            self.shared.selection.set(Some(root_id));
        }
    }

    fn keyboard_shortcuts(&mut self, context: &egui::Context) {
        if context.egui_wants_keyboard_input() {
            return;
        }
        let input = context.input(|input| {
            (
                input.modifiers,
                input.key_pressed(egui::Key::Z),
                input.key_pressed(egui::Key::Y),
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::Delete) || input.key_pressed(egui::Key::Backspace),
            )
        });
        let (modifiers, z, y, up, down, left, right, delete) = input;
        if modifiers.command && z && modifiers.shift {
            self.shared.redo();
            return;
        }
        if modifiers.command && z {
            self.shared.undo();
            return;
        }
        if modifiers.command && y {
            self.shared.redo();
            return;
        }

        let Some(selected) = self.shared.selection.get() else {
            return;
        };
        if modifiers.alt && (up || down || left || right) {
            let mut project = self.shared.project.get();
            let result = if up {
                project
                    .reorder_node(selected, -1)
                    .then_some("Moved node up")
                    .ok_or_else(|| "Node is already first".to_owned())
            } else if down {
                project
                    .reorder_node(selected, 1)
                    .then_some("Moved node down")
                    .ok_or_else(|| "Node is already last".to_owned())
            } else if left {
                project.outdent_node(selected).map(|()| "Outdented node")
            } else {
                project.indent_node(selected).map(|()| "Indented node")
            };
            match result {
                Ok(message) => {
                    self.shared.commit_project(project, message);
                }
                Err(error) => self.shared.set_status(error),
            }
            return;
        }
        if delete && selected != self.shared.project.get().root.id {
            let mut project = self.shared.project.get();
            let parent = project.parent_id(selected).unwrap_or(project.root.id);
            if project.remove_node(selected) {
                self.shared.commit_project(project, "Deleted node");
                self.shared.selection.set(Some(parent));
            }
            return;
        }
        if up || down {
            let ids = self.shared.project.get().node_ids_depth_first();
            if let Some(index) = ids.iter().position(|id| *id == selected) {
                let next = if up {
                    index.saturating_sub(1)
                } else {
                    (index + 1).min(ids.len().saturating_sub(1))
                };
                self.shared.selection.set(ids.get(next).copied());
                self.shared.set_status("Moved keyboard selection");
            }
        }
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
        self.keyboard_shortcuts(ui.ctx());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_history_round_trips_project_and_selection() {
        let initial = CitizenProject::default();
        let shared = BuilderSharedState::new(initial.clone());
        let selected = initial.root.children[0].id;
        shared.selection.set(Some(selected));

        let mut changed = initial.clone();
        changed.title = "Changed".to_owned();
        assert!(shared.commit_project(changed.clone(), "change"));
        assert!(shared.can_undo());
        assert!(shared.undo());
        assert_eq!(shared.project.get(), initial);
        assert_eq!(shared.selection.get(), Some(selected));
        assert!(shared.redo());
        assert_eq!(shared.project.get(), changed);
    }

    #[test]
    fn new_commit_clears_redo_history() {
        let initial = CitizenProject::default();
        let shared = BuilderSharedState::new(initial.clone());
        let mut first = initial;
        first.title = "First".to_owned();
        shared.commit_project(first, "first");
        shared.undo();
        assert!(shared.can_redo());

        let mut second = shared.project.get();
        second.title = "Second".to_owned();
        shared.commit_project(second, "second");
        assert!(!shared.can_redo());
    }
}
