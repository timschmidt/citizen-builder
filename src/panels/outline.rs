use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::{DesignNode, NodeId, PaletteItem};

/// Citizen panel for the semantic node hierarchy and palette.
pub(crate) struct OutlinePanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl OutlinePanel {
    pub(crate) const ID: &'static str = "builder_outline";

    pub(crate) fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(Self::ID),
            citizen_state,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, shared: &BuilderSharedState) {
        ui.horizontal(|ui| {
            ui.heading("Citizen outline");
            if self.is_active() {
                ui.colored_label(egui::Color32::LIGHT_GREEN, "active");
            }
        });
        ui.label("Select a layout to add children beneath it.");
        ui.separator();

        let project = shared.project.get();
        let selected = shared.selection.get();
        egui::ScrollArea::vertical()
            .id_salt("citizen_outline_tree")
            .max_height(300.0)
            .show(ui, |ui| show_tree(ui, &project.root, selected, shared));

        ui.horizontal(|ui| {
            let can_delete = selected.is_some_and(|id| id != project.root.id);
            if ui
                .add_enabled(can_delete, egui::Button::new("Delete selected"))
                .clicked()
                && let Some(id) = selected
            {
                let mut updated = project.clone();
                if updated.remove_node(id) {
                    let root_id = updated.root.id;
                    shared.project.set(updated);
                    shared.selection.set(Some(root_id));
                    shared.set_status("Deleted node");
                }
            }
        });

        ui.separator();
        egui::CollapsingHeader::new("Layouts")
            .default_open(true)
            .show(ui, |ui| {
                palette_grid(ui, shared, selected, PaletteItem::LAYOUTS)
            });
        egui::CollapsingHeader::new("Widgets")
            .default_open(true)
            .show(ui, |ui| {
                palette_grid(ui, shared, selected, PaletteItem::WIDGETS)
            });
    }
}

impl Citizen for OutlinePanel {
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

fn show_tree(
    ui: &mut egui::Ui,
    node: &DesignNode,
    selected: Option<NodeId>,
    shared: &BuilderSharedState,
) {
    let label = format!("{}  ·  {}", node.name, node.kind.display_name());
    if node.kind.allows_children() {
        let response = egui::CollapsingHeader::new(label)
            .id_salt(("outline_node", node.id.0))
            .default_open(true)
            .show(ui, |ui| {
                for child in &node.children {
                    show_tree(ui, child, selected, shared);
                }
            })
            .header_response;
        if response.clicked() {
            shared.selection.set(Some(node.id));
        }
        if selected == Some(node.id) {
            ui.painter().rect_stroke(
                response.rect,
                egui::CornerRadius::same(2),
                egui::Stroke::new(1.0, egui::Color32::from_rgb(95, 170, 255)),
                egui::StrokeKind::Outside,
            );
        }
    } else if ui
        .selectable_label(selected == Some(node.id), label)
        .clicked()
    {
        shared.selection.set(Some(node.id));
    }
}

fn palette_grid(
    ui: &mut egui::Ui,
    shared: &BuilderSharedState,
    selected: Option<NodeId>,
    items: &[PaletteItem],
) {
    egui::Grid::new(("palette_grid", items.as_ptr() as usize))
        .num_columns(2)
        .spacing([6.0, 6.0])
        .show(ui, |ui| {
            for (index, item) in items.iter().copied().enumerate() {
                if ui
                    .add_sized([105.0, 26.0], egui::Button::new(item.display_name()))
                    .clicked()
                {
                    let mut project = shared.project.get();
                    let id = project.add_palette_item(selected, item);
                    shared.project.set(project);
                    shared.selection.set(Some(id));
                    shared.set_status(format!("Added {}", item.display_name()));
                }
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });
}
