//! Citizen-first visual authoring and standalone crate generation.
//!
//! One [`model::CitizenProject`] describes one reusable Level 1 Citizen. The
//! [`generator`] validates that semantic layout and emits a deterministic
//! library crate with a native/WASM preview host.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Builder application state and code-generation settings.
pub mod app;
/// Deterministic standalone Citizen crate generation.
pub mod generator;
/// Versioned single-Citizen project model and validation.
pub mod model;
mod panels;
mod preview;
