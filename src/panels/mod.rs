//! Dogfooded Citizen panels that make up the builder workspace.

mod canvas;
mod generated;
mod inspector;
mod outline;

pub(crate) use canvas::CanvasPanel;
pub(crate) use generated::GeneratedPanel;
pub(crate) use inspector::InspectorPanel;
pub(crate) use outline::OutlinePanel;
