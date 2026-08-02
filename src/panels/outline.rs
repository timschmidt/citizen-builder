use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};

use crate::app::BuilderSharedState;
use crate::model::{DesignNode, MovePlacement, NodeId, PaletteItem};

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
        let mut pending_move = None;
        egui::ScrollArea::vertical()
            .id_salt("citizen_outline_tree")
            .max_height(300.0)
            .show(ui, |ui| {
                show_tree(
                    ui,
                    &project.root,
                    project.root.id,
                    selected,
                    shared,
                    &mut pending_move,
                )
            });

        if let Some((source, target, placement)) = pending_move {
            let mut updated = project.clone();
            match updated.move_node_relative(source, target, placement) {
                Ok(()) => {
                    shared.commit_project(updated, "Moved node in Citizen tree");
                    shared.selection.set(Some(source));
                }
                Err(error) => shared.set_status(format!("Move rejected: {error}")),
            }
        }

        ui.horizontal_wrapped(|ui| {
            let can_delete = selected.is_some_and(|id| id != project.root.id);
            if ui
                .add_enabled(can_delete, egui::Button::new("Delete selected"))
                .clicked()
                && let Some(id) = selected
            {
                let mut updated = project.clone();
                if updated.remove_node(id) {
                    let root_id = updated.root.id;
                    shared.commit_project(updated, "Deleted node");
                    shared.selection.set(Some(root_id));
                }
            }
            for (label, offset) in [("↑", -1), ("↓", 1)] {
                if ui
                    .add_enabled(can_delete, egui::Button::new(label))
                    .on_hover_text(if offset < 0 {
                        "Move before previous sibling (Alt+Up)"
                    } else {
                        "Move after next sibling (Alt+Down)"
                    })
                    .clicked()
                    && let Some(id) = selected
                {
                    let mut updated = project.clone();
                    if updated.reorder_node(id, offset) {
                        shared.commit_project(updated, "Reordered node");
                    }
                }
            }
            if ui
                .add_enabled(can_delete, egui::Button::new("Outdent"))
                .on_hover_text("Move after parent (Alt+Left)")
                .clicked()
                && let Some(id) = selected
            {
                let mut updated = project.clone();
                match updated.outdent_node(id) {
                    Ok(()) => {
                        shared.commit_project(updated, "Outdented node");
                    }
                    Err(error) => shared.set_status(error),
                }
            }
            if ui
                .add_enabled(can_delete, egui::Button::new("Indent"))
                .on_hover_text("Move into previous layout sibling (Alt+Right)")
                .clicked()
                && let Some(id) = selected
            {
                let mut updated = project.clone();
                match updated.indent_node(id) {
                    Ok(()) => {
                        shared.commit_project(updated, "Indented node");
                    }
                    Err(error) => shared.set_status(error),
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
    root_id: NodeId,
    selected: Option<NodeId>,
    shared: &BuilderSharedState,
    pending_move: &mut Option<(NodeId, NodeId, MovePlacement)>,
) {
    let label = format!("{}  ·  {}", node.name, node.kind.display_name());
    let response = if node.kind.allows_children() {
        egui::CollapsingHeader::new(label)
            .id_salt(("outline_node", node.id.0))
            .default_open(true)
            .show(ui, |ui| {
                for child in &node.children {
                    show_tree(ui, child, root_id, selected, shared, pending_move);
                }
            })
            .header_response
    } else {
        ui.selectable_label(selected == Some(node.id), label)
    };

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

    let drag_response = ui
        .interact(
            response.rect,
            egui::Id::new(("outline_drag", node.id.0)),
            egui::Sense::click_and_drag(),
        )
        .on_hover_cursor(egui::CursorIcon::Grab);
    drag_response.dnd_set_drag_payload(node.id);
    let drop_response = response | drag_response;
    if let Some(source) = drop_response.dnd_hover_payload::<NodeId>()
        && *source != node.id
    {
        let placement = drop_placement(ui, node, root_id, drop_response.rect);
        paint_drop_hint(ui, drop_response.rect, placement);
        if let Some(released) = drop_response.dnd_release_payload::<NodeId>() {
            *pending_move = Some((*released, node.id, placement));
        }
    }
}

fn drop_placement(
    ui: &egui::Ui,
    node: &DesignNode,
    root_id: NodeId,
    rect: egui::Rect,
) -> MovePlacement {
    if node.id == root_id {
        return MovePlacement::Inside;
    }
    let pointer_y = ui
        .ctx()
        .pointer_interact_pos()
        .map_or(rect.center().y, |position| position.y);
    let fraction = ((pointer_y - rect.top()) / rect.height().max(1.0)).clamp(0.0, 1.0);
    if node.kind.allows_children() && (0.25..=0.75).contains(&fraction) {
        MovePlacement::Inside
    } else if fraction < 0.5 {
        MovePlacement::Before
    } else {
        MovePlacement::After
    }
}

fn paint_drop_hint(ui: &egui::Ui, rect: egui::Rect, placement: MovePlacement) {
    let stroke = egui::Stroke::new(2.0, egui::Color32::from_rgb(80, 190, 255));
    match placement {
        MovePlacement::Inside => ui.painter().rect_stroke(
            rect.expand(2.0),
            egui::CornerRadius::same(3),
            stroke,
            egui::StrokeKind::Outside,
        ),
        MovePlacement::Before => ui
            .painter()
            .line_segment([rect.left_top(), rect.right_top()], stroke),
        MovePlacement::After => ui
            .painter()
            .line_segment([rect.left_bottom(), rect.right_bottom()], stroke),
    };
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
                    shared.commit_project(project, format!("Added {}", item.display_name()));
                    shared.selection.set(Some(id));
                }
                if index % 2 == 1 {
                    ui.end_row();
                }
            }
        });
}
