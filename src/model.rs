//! Versioned design model for one reusable egui_mobius Citizen.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Current on-disk project schema.
pub const CURRENT_SCHEMA_VERSION: u32 = 4;

/// Current deterministic generator contract stored in every project.
pub const CURRENT_GENERATOR_VERSION: u32 = 4;

/// Framework-specific backend emitted by the current generator.
pub const CURRENT_BACKEND_ID: &str = "egui_mobius";

/// Version of the egui_mobius generation backend.
pub const CURRENT_BACKEND_VERSION: &str = "0.5";

/// egui_mobius revision used by new projects and generated manifests.
pub const DEFAULT_FRAMEWORK_REVISION: &str = "cc286df73b6cb3a015b3d0c5159aeaf3f41510cc";

/// Default egui_mobius repository used for Citizen dependencies.
pub const DEFAULT_FRAMEWORK_REPOSITORY: &str = "https://github.com/saturn77/egui_mobius";

/// A complete design document for exactly one reusable Citizen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CitizenProject {
    /// Schema version used to reject incompatible project files explicitly.
    pub schema_version: u32,
    /// Cargo package name for the generated Citizen crate.
    pub crate_name: String,
    /// Public Rust type implementing `egui_citizen::Citizen`.
    pub citizen_type: String,
    /// Stable runtime identifier registered with the Dispatcher.
    pub citizen_id: String,
    /// Human-readable dock-tab title.
    pub title: String,
    /// Crate and Citizen documentation.
    pub description: String,
    /// Source of workspace-only egui_mobius crates.
    pub framework: FrameworkSource,
    /// Generator/backend contract required to interpret this document.
    pub generator: GeneratorMetadata,
    /// Typed reactive state required by this Citizen.
    pub state_fields: Vec<StateField>,
    /// Editable values used only by the live/reference preview.
    pub preview: PreviewFixture,
    /// Domain-grouped application intent and outcome definitions.
    pub messages: Vec<MessageDefinition>,
    /// Widget interaction to intent-message mappings.
    pub interactions: Vec<InteractionBinding>,
    /// Optional Level 3 intent mappings implemented with asynchronous work.
    pub async_behavior: AsyncBehavior,
    /// Generated preview and host visual theme.
    pub theme: ThemeDefinition,
    /// UTF-8 assets embedded into the standalone Citizen crate.
    pub assets: Vec<AssetDefinition>,
    /// Optional external-Citizen dock composition layered around this Citizen.
    pub composition: HostComposition,
    /// Root of the semantic immediate-mode layout tree.
    pub root: DesignNode,
}

/// Explicit generator and backend compatibility metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratorMetadata {
    /// Version of citizen-builder's file-generation contract.
    pub generator_version: u32,
    /// Stable backend identifier.
    pub backend: String,
    /// Backend schema/API version.
    pub backend_version: String,
}

impl Default for GeneratorMetadata {
    fn default() -> Self {
        Self {
            generator_version: CURRENT_GENERATOR_VERSION,
            backend: CURRENT_BACKEND_ID.to_owned(),
            backend_version: CURRENT_BACKEND_VERSION.to_owned(),
        }
    }
}

/// Named preview values kept separate from generated state defaults.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewFixture {
    /// Human-readable fixture name.
    pub name: String,
    /// State-field values keyed by the public field name.
    pub values: BTreeMap<String, StateValue>,
}

/// Stable identity of one application-level message.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageKey {
    /// Lowercase snake_case message domain.
    pub domain: String,
    /// PascalCase variant within the domain.
    pub variant: String,
}

impl MessageKey {
    /// Human-readable `domain::Variant` identity.
    pub fn display_name(&self) -> String {
        format!("{}::{}", self.domain, self.variant)
    }
}

/// Role of a discrete application message.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    /// User/application intent routed to a backend.
    Intent,
    /// Backend/application outcome routed back to the UI.
    Outcome,
}

/// One fixed state update applied when an outcome reaches the UI thread.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateAssignment {
    /// Reactive state field to update.
    pub field: String,
    /// Value written by this reference outcome.
    pub value: StateValue,
}

/// One domain-grouped intent or outcome message.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MessageDefinition {
    /// Stable domain and enum-variant identity.
    pub key: MessageKey,
    /// Whether this message requests work or reports its result.
    pub role: MessageRole,
    /// Documentation emitted beside the generated enum variant.
    pub description: String,
    /// Outcome returned by the synchronous reference backend for an intent.
    pub paired_outcome: Option<MessageKey>,
    /// UI-thread state updates performed by an outcome.
    pub state_updates: Vec<StateAssignment>,
}

/// Supported discrete widget interaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InteractionEvent {
    /// Button or clickable control activation.
    Click,
    /// A bound control changed its continuous value.
    Change,
    /// A text editor submitted with Enter.
    Submit,
}

impl InteractionEvent {
    /// Inspector label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Click => "Click",
            Self::Change => "Change",
            Self::Submit => "Submit",
        }
    }
}

/// Mapping from a node interaction to an intent message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InteractionBinding {
    /// Node that emits the event.
    pub node: NodeId,
    /// Discrete interaction to observe.
    pub event: InteractionEvent,
    /// Intent placed into the Citizen outbox.
    pub message: MessageKey,
}

/// Opt-in Level 3 asynchronous backend configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsyncBehavior {
    /// Whether async generation and preview routing are enabled.
    pub enabled: bool,
    /// Intent/outcome work mappings routed through a signal/slot boundary.
    pub mappings: Vec<AsyncMapping>,
}

/// One intent handled asynchronously before its outcome returns to the UI thread.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AsyncMapping {
    /// Intent submitted by the Citizen.
    pub intent: MessageKey,
    /// Outcome returned after background work.
    pub outcome: MessageKey,
    /// Reference delay used by native and browser preview backends.
    pub delay_ms: u32,
}

/// Preview/host theme emitted as reusable Rust configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeDefinition {
    /// Dark or light base visuals.
    pub preset: ThemePreset,
    /// RGB selection/accent color.
    pub accent_rgb: [u8; 3],
    /// RGB panel background color.
    pub panel_rgb: [u8; 3],
    /// Horizontal and vertical item spacing in egui points.
    pub item_spacing: f32,
}

impl Default for ThemeDefinition {
    fn default() -> Self {
        Self {
            preset: ThemePreset::Dark,
            accent_rgb: [45, 100, 165],
            panel_rgb: [25, 28, 34],
            item_spacing: 8.0,
        }
    }
}

/// Base egui visual preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThemePreset {
    /// Dark egui visuals.
    Dark,
    /// Light egui visuals.
    Light,
}

impl ThemePreset {
    /// Inspector label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
        }
    }
}

/// UTF-8 project asset copied into the generated crate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssetDefinition {
    /// Safe relative file name beneath generated `assets/`.
    pub file_name: String,
    /// Asset interpretation used by tooling and documentation.
    pub kind: AssetKind,
    /// UTF-8 source contents.
    pub contents: String,
}

/// Supported embedded UTF-8 asset classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    /// Plain text, Markdown, JSON, or another UTF-8 resource.
    Text,
    /// Inline scalable vector graphic source.
    Svg,
}

impl AssetKind {
    /// Inspector label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Svg => "SVG",
        }
    }
}

/// Optional visual host composition for external Citizens.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostComposition {
    /// Whether generated preview/scaffolding includes external Citizen placeholders.
    pub enabled: bool,
    /// External compile-time Citizen dependencies surrounding this project Citizen.
    pub external_citizens: Vec<HostCitizen>,
}

/// One external Citizen reference and dock placement.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostCitizen {
    /// Cargo package providing the external Citizen.
    pub crate_name: String,
    /// Public Rust type implementing the external Citizen contract.
    pub citizen_type: String,
    /// Stable external CitizenId.
    pub citizen_id: String,
    /// Dock-tab title shown in the composition preview.
    pub title: String,
    /// Placement relative to the primary generated Citizen.
    pub placement: DockPlacement,
    /// Fraction of the available dock area allocated to this external Citizen.
    pub fraction: f32,
}

/// Dock placement relative to the primary Citizen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockPlacement {
    /// Add as a tab in the primary leaf.
    Tab,
    /// Split to the left.
    Left,
    /// Split to the right.
    Right,
    /// Split above.
    Above,
    /// Split below.
    Below,
}

impl DockPlacement {
    /// Inspector label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Tab => "Tab",
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Above => "Above",
            Self::Below => "Below",
        }
    }
}

/// Dependency source emitted for `egui_citizen` and `egui_mobius_reactive`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "source", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrameworkSource {
    /// A reproducible Git dependency pinned to an exact revision.
    Git {
        /// Git repository containing the egui_mobius workspace.
        repository: String,
        /// Exact commit revision.
        revision: String,
    },
    /// A local egui_mobius workspace root used during framework development.
    Path {
        /// Path whose `crates/` directory contains the framework crates.
        workspace: String,
    },
}

impl Default for FrameworkSource {
    fn default() -> Self {
        Self::Git {
            repository: DEFAULT_FRAMEWORK_REPOSITORY.to_owned(),
            revision: DEFAULT_FRAMEWORK_REVISION.to_owned(),
        }
    }
}

/// Stable editor identity for a layout or widget node.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u64);

/// Placement used by tree drag/drop and keyboard reparenting commands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovePlacement {
    /// Insert immediately before the target in its parent.
    Before,
    /// Append as the target layout's final child.
    Inside,
    /// Insert immediately after the target in its parent.
    After,
}

/// One semantic layout or widget node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesignNode {
    /// Editor-stable identity, independent of the generated Rust name.
    pub id: NodeId,
    /// Stable snake_case name used for generated egui IDs.
    pub name: String,
    /// Layout or widget behavior.
    pub kind: NodeKind,
    /// Ordered children. Only layout nodes may contain children.
    pub children: Vec<DesignNode>,
}

/// Supported semantic layouts and Citizen widgets.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NodeKind {
    /// Vertical immediate-mode layout.
    Column,
    /// Horizontal layout, optionally wrapping at the available width.
    Row {
        /// Whether children wrap onto additional rows.
        wrap: bool,
    },
    /// Multi-column egui grid.
    Grid {
        /// Number of columns before ending a row.
        columns: usize,
        /// Whether alternating rows use a striped background.
        striped: bool,
    },
    /// Framed group with an optional title.
    Group {
        /// Title rendered above the group's children.
        title: String,
    },
    /// Vertically scrolling layout.
    Scroll {
        /// Maximum preview/generated height. Zero means unconstrained.
        max_height: f32,
    },
    /// Static label.
    Label {
        /// Displayed text.
        text: String,
    },
    /// Static heading.
    Heading {
        /// Displayed text.
        text: String,
    },
    /// Standard egui button with an optional click-intent interaction.
    Button {
        /// Button text.
        text: String,
    },
    /// Curated `egui_mobius_widgets::StyledButton` wrapper.
    StyledButton {
        /// Button text.
        text: String,
    },
    /// Curated `egui_lens::ReactiveEventLogger` wrapper.
    ReactiveLogger,
    /// Curated `egui_quill::ReactiveEditor` wrapper.
    ReactiveEditor {
        /// Initial editor contents.
        content: String,
        /// Initial syntax language.
        language: String,
    },
    /// Curated `egui_plot` line-plot wrapper backed by an amplitude field.
    LinePlot {
        /// Numeric amplitude binding.
        binding: Option<String>,
    },
    /// Boolean control backed by `Dynamic<bool>`.
    Checkbox {
        /// Checkbox text.
        text: String,
        /// Reactive field name.
        binding: Option<String>,
    },
    /// Single-line text control backed by `Dynamic<String>`.
    TextEdit {
        /// Label rendered before the editor.
        label: String,
        /// Empty-state hint.
        hint: String,
        /// Reactive field name.
        binding: Option<String>,
    },
    /// Numeric slider backed by `Dynamic<f32>`.
    Slider {
        /// Slider label.
        label: String,
        /// Inclusive minimum.
        min: f32,
        /// Inclusive maximum.
        max: f32,
        /// Reactive field name.
        binding: Option<String>,
    },
    /// Read-only progress display backed by `Dynamic<f32>`.
    ProgressBar {
        /// Reactive field name.
        binding: Option<String>,
        /// Show a percentage next to the bar.
        show_percentage: bool,
    },
    /// Horizontal separator.
    Separator,
    /// Explicit layout spacing.
    Spacer {
        /// Spacing in egui points.
        points: f32,
    },
}

impl NodeKind {
    /// Whether this node may own children.
    pub const fn allows_children(&self) -> bool {
        matches!(
            self,
            Self::Column
                | Self::Row { .. }
                | Self::Grid { .. }
                | Self::Group { .. }
                | Self::Scroll { .. }
        )
    }

    /// Human-readable node kind.
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Column => "Column",
            Self::Row { .. } => "Row",
            Self::Grid { .. } => "Grid",
            Self::Group { .. } => "Group",
            Self::Scroll { .. } => "Scroll",
            Self::Label { .. } => "Label",
            Self::Heading { .. } => "Heading",
            Self::Button { .. } => "Button",
            Self::StyledButton { .. } => "Mobius Styled Button",
            Self::ReactiveLogger => "Lens Logger",
            Self::ReactiveEditor { .. } => "Quill Editor",
            Self::LinePlot { .. } => "Line Plot",
            Self::Checkbox { .. } => "Checkbox",
            Self::TextEdit { .. } => "Text Edit",
            Self::Slider { .. } => "Slider",
            Self::ProgressBar { .. } => "Progress Bar",
            Self::Separator => "Separator",
            Self::Spacer { .. } => "Spacer",
        }
    }

    /// Discrete interactions this widget may emit.
    pub const fn supported_interactions(&self) -> &'static [InteractionEvent] {
        match self {
            Self::Button { .. } | Self::StyledButton { .. } => &[InteractionEvent::Click],
            Self::Checkbox { .. } | Self::Slider { .. } => &[InteractionEvent::Change],
            Self::TextEdit { .. } => &[InteractionEvent::Change, InteractionEvent::Submit],
            _ => &[],
        }
    }

    fn expected_binding(&self) -> Option<StateType> {
        match self {
            Self::Checkbox { .. } => Some(StateType::Bool),
            Self::TextEdit { .. } => Some(StateType::Text),
            Self::Slider { .. } | Self::ProgressBar { .. } | Self::LinePlot { .. } => {
                Some(StateType::Number)
            }
            _ => None,
        }
    }

    /// Current binding name, if this kind is state-bound.
    pub fn binding(&self) -> Option<&str> {
        match self {
            Self::Checkbox { binding, .. }
            | Self::TextEdit { binding, .. }
            | Self::Slider { binding, .. }
            | Self::ProgressBar { binding, .. }
            | Self::LinePlot { binding } => binding.as_deref(),
            _ => None,
        }
    }

    /// Mutable binding slot for state-bound widgets.
    pub fn binding_mut(&mut self) -> Option<&mut Option<String>> {
        match self {
            Self::Checkbox { binding, .. }
            | Self::TextEdit { binding, .. }
            | Self::Slider { binding, .. }
            | Self::ProgressBar { binding, .. }
            | Self::LinePlot { binding } => Some(binding),
            _ => None,
        }
    }
}

/// Typed initial value for one public reactive state field.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "default", rename_all = "snake_case")]
pub enum StateValue {
    /// `Dynamic<bool>`.
    Bool(bool),
    /// `Dynamic<String>`.
    Text(String),
    /// `Dynamic<f32>`.
    Number(f32),
}

impl StateValue {
    /// Type represented by this value.
    pub const fn state_type(&self) -> StateType {
        match self {
            Self::Bool(_) => StateType::Bool,
            Self::Text(_) => StateType::Text,
            Self::Number(_) => StateType::Number,
        }
    }
}

/// One named reactive field in the generated Citizen state contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateField {
    /// Public snake_case Rust field name.
    pub name: String,
    /// Type and default value.
    pub value: StateValue,
}

/// State type used by binding validation and the editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StateType {
    /// Boolean state.
    Bool,
    /// String state.
    Text,
    /// Floating-point state.
    Number,
}

impl StateType {
    /// Display label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Bool => "bool",
            Self::Text => "String",
            Self::Number => "f32",
        }
    }
}

impl StateField {
    /// Type of this field.
    pub const fn state_type(&self) -> StateType {
        self.value.state_type()
    }
}

/// Palette templates supported by the first Citizen-first editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaletteItem {
    /// Column layout.
    Column,
    /// Row layout.
    Row,
    /// Grid layout.
    Grid,
    /// Group layout.
    Group,
    /// Scroll layout.
    Scroll,
    /// Label widget.
    Label,
    /// Heading widget.
    Heading,
    /// Button widget.
    Button,
    /// Curated Mobius styled button.
    StyledButton,
    /// Curated egui_lens logger.
    ReactiveLogger,
    /// Curated egui_quill editor.
    ReactiveEditor,
    /// Curated egui_plot line plot.
    LinePlot,
    /// Bound checkbox.
    Checkbox,
    /// Bound text editor.
    TextEdit,
    /// Bound slider.
    Slider,
    /// Bound progress bar.
    ProgressBar,
    /// Separator.
    Separator,
    /// Spacer.
    Spacer,
}

impl PaletteItem {
    /// All layout templates.
    pub const LAYOUTS: &'static [Self] = &[
        Self::Column,
        Self::Row,
        Self::Grid,
        Self::Group,
        Self::Scroll,
    ];

    /// All widget templates.
    pub const WIDGETS: &'static [Self] = &[
        Self::Label,
        Self::Heading,
        Self::Button,
        Self::StyledButton,
        Self::ReactiveLogger,
        Self::ReactiveEditor,
        Self::LinePlot,
        Self::Checkbox,
        Self::TextEdit,
        Self::Slider,
        Self::ProgressBar,
        Self::Separator,
        Self::Spacer,
    ];

    /// Palette label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Column => "Column",
            Self::Row => "Row",
            Self::Grid => "Grid",
            Self::Group => "Group",
            Self::Scroll => "Scroll",
            Self::Label => "Label",
            Self::Heading => "Heading",
            Self::Button => "Button",
            Self::StyledButton => "Mobius Styled Button",
            Self::ReactiveLogger => "Lens Logger",
            Self::ReactiveEditor => "Quill Editor",
            Self::LinePlot => "Line Plot",
            Self::Checkbox => "Checkbox",
            Self::TextEdit => "Text Edit",
            Self::Slider => "Slider",
            Self::ProgressBar => "Progress Bar",
            Self::Separator => "Separator",
            Self::Spacer => "Spacer",
        }
    }

    const fn base_name(self) -> &'static str {
        match self {
            Self::Column => "column",
            Self::Row => "row",
            Self::Grid => "grid",
            Self::Group => "group",
            Self::Scroll => "scroll",
            Self::Label => "label",
            Self::Heading => "heading",
            Self::Button => "button",
            Self::StyledButton => "styled_button",
            Self::ReactiveLogger => "reactive_logger",
            Self::ReactiveEditor => "reactive_editor",
            Self::LinePlot => "line_plot",
            Self::Checkbox => "checkbox",
            Self::TextEdit => "text_edit",
            Self::Slider => "slider",
            Self::ProgressBar => "progress_bar",
            Self::Separator => "separator",
            Self::Spacer => "spacer",
        }
    }
}

/// Complete one-Citizen starting points for common application roles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CitizenTemplate {
    /// Typed preferences with nested groups and an apply action.
    #[default]
    Settings,
    /// Filterable egui_lens event log.
    Logger,
    /// egui_quill source editor with a save action.
    Editor,
    /// Reactive egui_plot visualization controls.
    Plot,
    /// Path/filter controls and asynchronous refresh.
    FileBrowser,
    /// Cancellable backend start/stop controls and progress state.
    BackendControl,
}

impl CitizenTemplate {
    /// Every supported new-project template.
    pub const ALL: &'static [Self] = &[
        Self::Settings,
        Self::Logger,
        Self::Editor,
        Self::Plot,
        Self::FileBrowser,
        Self::BackendControl,
    ];

    /// Template picker label.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Settings => "Settings",
            Self::Logger => "Logger",
            Self::Editor => "Editor",
            Self::Plot => "Plot",
            Self::FileBrowser => "File Browser",
            Self::BackendControl => "Backend Control",
        }
    }

    /// Short template intent shown in the picker.
    pub const fn description(self) -> &'static str {
        match self {
            Self::Settings => "Nested bool, text, and numeric preferences",
            Self::Logger => "Reactive event log with filter and clear action",
            Self::Editor => "Syntax-aware source editor and save intent",
            Self::Plot => "Reactive amplitude controls and line visualization",
            Self::FileBrowser => "Path filters with cancellable async refresh",
            Self::BackendControl => "Cancellable backend work and progress controls",
        }
    }
}

/// Validation severity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticSeverity {
    /// Generation-blocking problem.
    Error,
    /// Non-blocking design warning.
    Warning,
}

/// One model validation result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    /// Severity.
    pub severity: DiagnosticSeverity,
    /// Model path or semantic node name.
    pub path: String,
    /// Human-readable explanation.
    pub message: String,
}

impl Diagnostic {
    fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Error,
            path: path.into(),
            message: message.into(),
        }
    }

    fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: DiagnosticSeverity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }
}

impl Default for CitizenProject {
    fn default() -> Self {
        let state_fields = vec![
            StateField {
                name: "enabled".to_owned(),
                value: StateValue::Bool(true),
            },
            StateField {
                name: "display_name".to_owned(),
                value: StateValue::Text("Citizen".to_owned()),
            },
            StateField {
                name: "level".to_owned(),
                value: StateValue::Number(0.5),
            },
        ];
        let preview = PreviewFixture {
            name: "Example data".to_owned(),
            values: BTreeMap::from([
                ("enabled".to_owned(), StateValue::Bool(false)),
                (
                    "display_name".to_owned(),
                    StateValue::Text("Preview Citizen".to_owned()),
                ),
                ("level".to_owned(), StateValue::Number(0.75)),
            ]),
        };
        let apply_requested = MessageKey {
            domain: "settings".to_owned(),
            variant: "ApplyRequested".to_owned(),
        };
        let apply_completed = MessageKey {
            domain: "settings".to_owned(),
            variant: "ApplyCompleted".to_owned(),
        };
        let messages = vec![
            MessageDefinition {
                key: apply_requested.clone(),
                role: MessageRole::Intent,
                description: "Request that the current settings be applied.".to_owned(),
                paired_outcome: Some(apply_completed.clone()),
                state_updates: Vec::new(),
            },
            MessageDefinition {
                key: apply_completed.clone(),
                role: MessageRole::Outcome,
                description: "Report that the settings were applied.".to_owned(),
                paired_outcome: None,
                state_updates: vec![StateAssignment {
                    field: "display_name".to_owned(),
                    value: StateValue::Text("Applied".to_owned()),
                }],
            },
        ];
        let interactions = vec![InteractionBinding {
            node: NodeId(8),
            event: InteractionEvent::Click,
            message: apply_requested.clone(),
        }];

        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            crate_name: "example-citizen".to_owned(),
            citizen_type: "ExampleCitizen".to_owned(),
            citizen_id: "example".to_owned(),
            title: "Example Citizen".to_owned(),
            description: "A reusable Citizen generated by citizen-builder.".to_owned(),
            framework: FrameworkSource::default(),
            generator: GeneratorMetadata::default(),
            state_fields,
            preview,
            messages,
            interactions,
            async_behavior: AsyncBehavior {
                enabled: false,
                mappings: vec![AsyncMapping {
                    intent: apply_requested,
                    outcome: apply_completed,
                    delay_ms: 350,
                }],
            },
            theme: ThemeDefinition::default(),
            assets: Vec::new(),
            composition: HostComposition::default(),
            root: DesignNode {
                id: NodeId(1),
                name: "root".to_owned(),
                kind: NodeKind::Column,
                children: vec![
                    DesignNode {
                        id: NodeId(2),
                        name: "title".to_owned(),
                        kind: NodeKind::Heading {
                            text: "Example Citizen".to_owned(),
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(3),
                        name: "description".to_owned(),
                        kind: NodeKind::Label {
                            text: "Edit this Citizen visually, then export a standalone crate."
                                .to_owned(),
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(4),
                        name: "enabled_control".to_owned(),
                        kind: NodeKind::Checkbox {
                            text: "Enabled".to_owned(),
                            binding: Some("enabled".to_owned()),
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(5),
                        name: "name_control".to_owned(),
                        kind: NodeKind::TextEdit {
                            label: "Name".to_owned(),
                            hint: "Citizen name".to_owned(),
                            binding: Some("display_name".to_owned()),
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(6),
                        name: "level_control".to_owned(),
                        kind: NodeKind::Slider {
                            label: "Level".to_owned(),
                            min: 0.0,
                            max: 1.0,
                            binding: Some("level".to_owned()),
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(7),
                        name: "level_progress".to_owned(),
                        kind: NodeKind::ProgressBar {
                            binding: Some("level".to_owned()),
                            show_percentage: true,
                        },
                        children: Vec::new(),
                    },
                    DesignNode {
                        id: NodeId(8),
                        name: "apply_button".to_owned(),
                        kind: NodeKind::Button {
                            text: "Apply".to_owned(),
                        },
                        children: Vec::new(),
                    },
                ],
            },
        }
    }
}

impl CitizenProject {
    /// Parse only the current Citizen schema.
    pub fn from_json(json: &str) -> Result<Self, String> {
        let project: Self = serde_json::from_str(json).map_err(|error| error.to_string())?;
        if project.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(format!(
                "unsupported Citizen project schema {}; expected {}",
                project.schema_version, CURRENT_SCHEMA_VERSION
            ));
        }
        Ok(project)
    }

    /// Serialize the current Citizen schema deterministically for project files.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Convert the Cargo package name to the Rust crate identifier.
    pub fn crate_ident(&self) -> String {
        self.crate_name.replace('-', "_")
    }

    /// Build a complete, generation-valid project from a curated Citizen template.
    pub fn from_template(template: CitizenTemplate) -> Self {
        match template {
            CitizenTemplate::Settings => Self::settings_template(),
            CitizenTemplate::Logger => Self::logger_template(),
            CitizenTemplate::Editor => Self::editor_template(),
            CitizenTemplate::Plot => Self::plot_template(),
            CitizenTemplate::FileBrowser => Self::file_browser_template(),
            CitizenTemplate::BackendControl => Self::backend_control_template(),
        }
    }

    /// Cargo/UI capabilities inferred entirely from the semantic project.
    pub fn inferred_features(&self) -> BTreeSet<String> {
        let mut features = BTreeSet::new();
        if self.async_behavior.enabled {
            features.insert("async-backend".to_owned());
        }
        if !self.assets.is_empty() {
            features.insert("embedded-assets".to_owned());
        }
        if self.composition.enabled {
            features.insert("host-composition".to_owned());
        }
        collect_inferred_node_features(&self.root, &mut features);
        features
    }

    fn template_base(
        crate_name: &str,
        citizen_type: &str,
        citizen_id: &str,
        title: &str,
        description: &str,
    ) -> Self {
        let mut project = Self {
            crate_name: crate_name.to_owned(),
            citizen_type: citizen_type.to_owned(),
            citizen_id: citizen_id.to_owned(),
            title: title.to_owned(),
            description: description.to_owned(),
            ..Self::default()
        };
        project.state_fields.clear();
        project.preview = PreviewFixture {
            name: format!("{title} example"),
            values: BTreeMap::new(),
        };
        project.messages.clear();
        project.interactions.clear();
        project.async_behavior = AsyncBehavior::default();
        project.theme = ThemeDefinition::default();
        project.assets.clear();
        project.composition = HostComposition::default();
        project.root = DesignNode {
            id: NodeId(1),
            name: "root".to_owned(),
            kind: NodeKind::Column,
            children: Vec::new(),
        };
        project
    }

    fn add_action_pair(
        &mut self,
        domain: &str,
        intent_variant: &str,
        outcome_variant: &str,
        node: NodeId,
        state_updates: Vec<StateAssignment>,
        async_delay_ms: Option<u32>,
    ) {
        let intent = MessageKey {
            domain: domain.to_owned(),
            variant: intent_variant.to_owned(),
        };
        let outcome = MessageKey {
            domain: domain.to_owned(),
            variant: outcome_variant.to_owned(),
        };
        self.messages.push(MessageDefinition {
            key: intent.clone(),
            role: MessageRole::Intent,
            description: format!("Request {domain} work from the host application."),
            paired_outcome: Some(outcome.clone()),
            state_updates: Vec::new(),
        });
        self.messages.push(MessageDefinition {
            key: outcome.clone(),
            role: MessageRole::Outcome,
            description: format!("Report that {domain} work completed."),
            paired_outcome: None,
            state_updates,
        });
        self.interactions.push(InteractionBinding {
            node,
            event: InteractionEvent::Click,
            message: intent.clone(),
        });
        if let Some(delay_ms) = async_delay_ms {
            self.async_behavior.enabled = true;
            self.async_behavior.mappings.push(AsyncMapping {
                intent,
                outcome,
                delay_ms,
            });
        }
    }

    fn settings_template() -> Self {
        let mut project = Self::template_base(
            "settings-citizen",
            "SettingsCitizen",
            "settings",
            "Settings",
            "Reusable nested settings panel with typed reactive preferences.",
        );
        let root = project.root.id;
        let heading = project.add_palette_item(Some(root), PaletteItem::Heading);
        project.find_node_mut(heading).unwrap().kind = NodeKind::Heading {
            text: "Application Settings".to_owned(),
        };
        let general = project.add_palette_item(Some(root), PaletteItem::Group);
        project.find_node_mut(general).unwrap().kind = NodeKind::Group {
            title: "General".to_owned(),
        };
        project.add_palette_item(Some(general), PaletteItem::Checkbox);
        project.add_palette_item(Some(general), PaletteItem::TextEdit);
        project.rename_state_field(1, "display_name".to_owned());
        let appearance = project.add_palette_item(Some(root), PaletteItem::Group);
        project.find_node_mut(appearance).unwrap().kind = NodeKind::Group {
            title: "Appearance".to_owned(),
        };
        project.add_palette_item(Some(appearance), PaletteItem::Slider);
        project.rename_state_field(2, "level".to_owned());
        let apply = project.add_palette_item(Some(root), PaletteItem::Button);
        project.find_node_mut(apply).unwrap().kind = NodeKind::Button {
            text: "Apply Settings".to_owned(),
        };
        project.add_action_pair(
            "settings",
            "ApplyRequested",
            "ApplyCompleted",
            apply,
            vec![StateAssignment {
                field: "display_name".to_owned(),
                value: StateValue::Text("Saved settings".to_owned()),
            }],
            None,
        );
        project.assets.push(AssetDefinition {
            file_name: "settings-help.md".to_owned(),
            kind: AssetKind::Text,
            contents: "# Settings\n\nDescribe host-specific preferences here.\n".to_owned(),
        });
        project
    }

    fn logger_template() -> Self {
        let mut project = Self::template_base(
            "logger-citizen",
            "LoggerCitizen",
            "logger",
            "Logger",
            "Filterable egui_lens event logger Citizen.",
        );
        let root = project.root.id;
        let filter = project.add_palette_item(Some(root), PaletteItem::TextEdit);
        project.rename_state_field(0, "filter".to_owned());
        project.find_node_mut(filter).unwrap().kind = NodeKind::TextEdit {
            label: "Filter".to_owned(),
            hint: "message text".to_owned(),
            binding: Some("filter".to_owned()),
        };
        project.add_palette_item(Some(root), PaletteItem::Checkbox);
        project.rename_state_field(1, "follow_tail".to_owned());
        project.add_palette_item(Some(root), PaletteItem::ReactiveLogger);
        let clear = project.add_palette_item(Some(root), PaletteItem::StyledButton);
        project.find_node_mut(clear).unwrap().kind = NodeKind::StyledButton {
            text: "Clear Log".to_owned(),
        };
        project.add_action_pair(
            "logger",
            "ClearRequested",
            "ClearCompleted",
            clear,
            vec![StateAssignment {
                field: "filter".to_owned(),
                value: StateValue::Text(String::new()),
            }],
            None,
        );
        project
    }

    fn editor_template() -> Self {
        let mut project = Self::template_base(
            "editor-citizen",
            "EditorCitizen",
            "editor",
            "Editor",
            "Syntax-aware egui_quill source editor Citizen.",
        );
        let root = project.root.id;
        let file_name = project.add_palette_item(Some(root), PaletteItem::TextEdit);
        project.rename_state_field(0, "file_name".to_owned());
        project.find_node_mut(file_name).unwrap().kind = NodeKind::TextEdit {
            label: "File".to_owned(),
            hint: "src/main.rs".to_owned(),
            binding: Some("file_name".to_owned()),
        };
        let editor = project.add_palette_item(Some(root), PaletteItem::ReactiveEditor);
        project.find_node_mut(editor).unwrap().kind = NodeKind::ReactiveEditor {
            content: "fn main() {\n    println!(\"Hello, Citizen!\");\n}\n".to_owned(),
            language: "Rust".to_owned(),
        };
        let save = project.add_palette_item(Some(root), PaletteItem::StyledButton);
        project.find_node_mut(save).unwrap().kind = NodeKind::StyledButton {
            text: "Save".to_owned(),
        };
        project.add_action_pair(
            "editor",
            "SaveRequested",
            "SaveCompleted",
            save,
            Vec::new(),
            None,
        );
        project.assets.push(AssetDefinition {
            file_name: "starter.rs".to_owned(),
            kind: AssetKind::Text,
            contents: "fn main() {\n    println!(\"Hello, Citizen!\");\n}\n".to_owned(),
        });
        project
    }

    fn plot_template() -> Self {
        let mut project = Self::template_base(
            "plot-citizen",
            "PlotCitizen",
            "plot",
            "Plot",
            "Reactive egui_plot line visualization Citizen.",
        );
        let root = project.root.id;
        let plot = project.add_palette_item(Some(root), PaletteItem::LinePlot);
        project.rename_state_field(0, "amplitude".to_owned());
        *project
            .find_node_mut(plot)
            .unwrap()
            .kind
            .binding_mut()
            .unwrap() = Some("amplitude".to_owned());
        let slider = project.add_palette_item(Some(root), PaletteItem::Slider);
        project.remove_state_field(1);
        project.find_node_mut(slider).unwrap().kind = NodeKind::Slider {
            label: "Amplitude".to_owned(),
            min: 0.0,
            max: 4.0,
            binding: Some("amplitude".to_owned()),
        };
        project.add_palette_item(Some(root), PaletteItem::Checkbox);
        project.rename_state_field(1, "auto_range".to_owned());
        let reset = project.add_palette_item(Some(root), PaletteItem::StyledButton);
        project.find_node_mut(reset).unwrap().kind = NodeKind::StyledButton {
            text: "Reset Plot".to_owned(),
        };
        project.add_action_pair(
            "plot",
            "ResetRequested",
            "ResetCompleted",
            reset,
            vec![StateAssignment {
                field: "amplitude".to_owned(),
                value: StateValue::Number(1.0),
            }],
            None,
        );
        project
    }

    fn file_browser_template() -> Self {
        let mut project = Self::template_base(
            "file-browser-citizen",
            "FileBrowserCitizen",
            "file_browser",
            "File Browser",
            "Host-portable file browser controls with cancellable refresh.",
        );
        let root = project.root.id;
        let path = project.add_palette_item(Some(root), PaletteItem::TextEdit);
        project.rename_state_field(0, "path".to_owned());
        project.find_node_mut(path).unwrap().kind = NodeKind::TextEdit {
            label: "Path".to_owned(),
            hint: "/workspace".to_owned(),
            binding: Some("path".to_owned()),
        };
        project.add_palette_item(Some(root), PaletteItem::Checkbox);
        project.rename_state_field(1, "show_hidden".to_owned());
        let refresh = project.add_palette_item(Some(root), PaletteItem::Button);
        project.find_node_mut(refresh).unwrap().kind = NodeKind::Button {
            text: "Refresh".to_owned(),
        };
        project.add_action_pair(
            "files",
            "RefreshRequested",
            "RefreshCompleted",
            refresh,
            Vec::new(),
            Some(250),
        );
        project
    }

    fn backend_control_template() -> Self {
        let mut project = Self::template_base(
            "backend-control-citizen",
            "BackendControlCitizen",
            "backend_control",
            "Backend Control",
            "Cancellable backend control and progress Citizen.",
        );
        let root = project.root.id;
        let endpoint = project.add_palette_item(Some(root), PaletteItem::TextEdit);
        project.rename_state_field(0, "endpoint".to_owned());
        project.find_node_mut(endpoint).unwrap().kind = NodeKind::TextEdit {
            label: "Endpoint".to_owned(),
            hint: "https://example.invalid".to_owned(),
            binding: Some("endpoint".to_owned()),
        };
        project.add_palette_item(Some(root), PaletteItem::Checkbox);
        project.rename_state_field(1, "connected".to_owned());
        let slider = project.add_palette_item(Some(root), PaletteItem::Slider);
        project.rename_state_field(2, "progress".to_owned());
        project.find_node_mut(slider).unwrap().kind = NodeKind::Slider {
            label: "Progress".to_owned(),
            min: 0.0,
            max: 1.0,
            binding: Some("progress".to_owned()),
        };
        let progress = project.add_palette_item(Some(root), PaletteItem::ProgressBar);
        project.remove_state_field(3);
        *project
            .find_node_mut(progress)
            .unwrap()
            .kind
            .binding_mut()
            .unwrap() = Some("progress".to_owned());
        let start = project.add_palette_item(Some(root), PaletteItem::StyledButton);
        project.find_node_mut(start).unwrap().kind = NodeKind::StyledButton {
            text: "Start Backend".to_owned(),
        };
        project.add_action_pair(
            "backend",
            "StartRequested",
            "StartCompleted",
            start,
            vec![
                StateAssignment {
                    field: "connected".to_owned(),
                    value: StateValue::Bool(true),
                },
                StateAssignment {
                    field: "progress".to_owned(),
                    value: StateValue::Number(1.0),
                },
            ],
            Some(600),
        );
        project
    }

    /// Locate one node immutably.
    pub fn find_node(&self, id: NodeId) -> Option<&DesignNode> {
        find_node(&self.root, id)
    }

    /// Locate one node mutably.
    pub fn find_node_mut(&mut self, id: NodeId) -> Option<&mut DesignNode> {
        find_node_mut(&mut self.root, id)
    }

    /// Add a palette template below the selected layout, falling back to root.
    pub fn add_palette_item(&mut self, selected: Option<NodeId>, item: PaletteItem) -> NodeId {
        let parent = selected
            .and_then(|id| {
                self.find_node(id)
                    .filter(|node| node.kind.allows_children())
            })
            .map_or(self.root.id, |node| node.id);
        let id = NodeId(self.next_node_id());
        let name = self.unique_node_name(item.base_name());
        let kind = self.kind_for_palette(item);
        let node = DesignNode {
            id,
            name,
            kind,
            children: Vec::new(),
        };
        self.find_node_mut(parent)
            .expect("root and selected parent must exist")
            .children
            .push(node);
        id
    }

    /// Remove a non-root node and all of its descendants.
    pub fn remove_node(&mut self, id: NodeId) -> bool {
        if id == self.root.id {
            return false;
        }
        let mut removed_ids = Vec::new();
        if let Some(node) = self.find_node(id) {
            collect_node_ids(node, &mut removed_ids);
        }
        if remove_node(&mut self.root, id) {
            self.interactions
                .retain(|interaction| !removed_ids.contains(&interaction.node));
            true
        } else {
            false
        }
    }

    /// IDs in visual depth-first order for keyboard navigation.
    pub fn node_ids_depth_first(&self) -> Vec<NodeId> {
        let mut ids = Vec::new();
        collect_node_ids(&self.root, &mut ids);
        ids
    }

    /// Parent of a non-root node.
    pub fn parent_id(&self, id: NodeId) -> Option<NodeId> {
        node_location(&self.root, id).map(|(parent, _)| parent)
    }

    /// Move a node before, inside, or after another node without allowing cycles.
    pub fn move_node_relative(
        &mut self,
        source: NodeId,
        target: NodeId,
        placement: MovePlacement,
    ) -> Result<(), String> {
        if source == self.root.id {
            return Err("the root layout cannot be moved".to_owned());
        }
        if source == target {
            return Err("a node cannot be moved relative to itself".to_owned());
        }
        let source_node = self
            .find_node(source)
            .ok_or_else(|| "the dragged node no longer exists".to_owned())?;
        let destination_parent = match placement {
            MovePlacement::Inside => {
                let target_node = self
                    .find_node(target)
                    .ok_or_else(|| "the drop target no longer exists".to_owned())?;
                if !target_node.kind.allows_children() {
                    return Err("only layout nodes can receive children".to_owned());
                }
                target
            }
            MovePlacement::Before | MovePlacement::After => node_location(&self.root, target)
                .map(|(parent, _)| parent)
                .ok_or_else(|| "the root layout has no sibling position".to_owned())?,
        };
        if find_node(source_node, destination_parent).is_some() {
            return Err("a node cannot be moved into its own descendants".to_owned());
        }

        let moved = take_node(&mut self.root, source)
            .ok_or_else(|| "the dragged node could not be detached".to_owned())?;
        match placement {
            MovePlacement::Inside => {
                self.find_node_mut(target)
                    .expect("validated target remains after cycle check")
                    .children
                    .push(moved);
            }
            MovePlacement::Before | MovePlacement::After => {
                let (parent, target_index) = node_location(&self.root, target)
                    .expect("validated sibling target remains after detach");
                let insertion =
                    target_index + usize::from(matches!(placement, MovePlacement::After));
                self.find_node_mut(parent)
                    .expect("validated target parent remains after detach")
                    .children
                    .insert(insertion, moved);
            }
        }
        Ok(())
    }

    /// Reorder a node among its siblings by a signed offset.
    pub fn reorder_node(&mut self, id: NodeId, offset: isize) -> bool {
        let Some((parent, index)) = node_location(&self.root, id) else {
            return false;
        };
        let Some(new_index) = index.checked_add_signed(offset) else {
            return false;
        };
        let siblings = &mut self
            .find_node_mut(parent)
            .expect("located parent exists")
            .children;
        if new_index >= siblings.len() {
            return false;
        }
        siblings.swap(index, new_index);
        true
    }

    /// Reparent a node into its preceding layout sibling.
    pub fn indent_node(&mut self, id: NodeId) -> Result<(), String> {
        let (parent, index) = node_location(&self.root, id)
            .ok_or_else(|| "the root layout cannot be indented".to_owned())?;
        if index == 0 {
            return Err("there is no preceding sibling to indent into".to_owned());
        }
        let target = self
            .find_node(parent)
            .expect("located parent exists")
            .children[index - 1]
            .id;
        self.move_node_relative(id, target, MovePlacement::Inside)
    }

    /// Move a node out of its parent, immediately after that parent.
    pub fn outdent_node(&mut self, id: NodeId) -> Result<(), String> {
        let parent = self
            .parent_id(id)
            .ok_or_else(|| "the root layout cannot be outdented".to_owned())?;
        if parent == self.root.id {
            return Err("this node is already directly below the root".to_owned());
        }
        self.move_node_relative(id, parent, MovePlacement::After)
    }

    /// Add a uniquely named state field of the requested type.
    pub fn add_state_field(&mut self, state_type: StateType) -> String {
        let base = match state_type {
            StateType::Bool => "enabled",
            StateType::Text => "text",
            StateType::Number => "value",
        };
        let name = self.unique_state_name(base);
        let value = match state_type {
            StateType::Bool => StateValue::Bool(false),
            StateType::Text => StateValue::Text(String::new()),
            StateType::Number => StateValue::Number(0.0),
        };
        self.state_fields.push(StateField {
            name: name.clone(),
            value: value.clone(),
        });
        self.preview.values.insert(name.clone(), value);
        name
    }

    /// Rename a state field and update every widget binding atomically.
    pub fn rename_state_field(&mut self, index: usize, new_name: String) -> bool {
        if self
            .state_fields
            .iter()
            .enumerate()
            .any(|(other_index, field)| other_index != index && field.name == new_name)
        {
            return false;
        }
        let Some(field) = self.state_fields.get_mut(index) else {
            return false;
        };
        if field.name == new_name {
            return false;
        }
        let old_name = std::mem::replace(&mut field.name, new_name.clone());
        rewrite_binding(&mut self.root, &old_name, Some(&new_name));
        for message in &mut self.messages {
            for update in &mut message.state_updates {
                if update.field == old_name {
                    update.field = new_name.clone();
                }
            }
        }
        if let Some(value) = self.preview.values.remove(&old_name) {
            self.preview.values.insert(new_name, value);
        }
        true
    }

    /// Remove a state field and clear bindings that referenced it.
    pub fn remove_state_field(&mut self, index: usize) -> bool {
        if index >= self.state_fields.len() {
            return false;
        }
        let removed = self.state_fields.remove(index);
        rewrite_binding(&mut self.root, &removed.name, None);
        for message in &mut self.messages {
            message
                .state_updates
                .retain(|update| update.field != removed.name);
        }
        self.preview.values.remove(&removed.name);
        true
    }

    /// Look up one application message definition.
    pub fn message(&self, key: &MessageKey) -> Option<&MessageDefinition> {
        self.messages.iter().find(|message| &message.key == key)
    }

    /// Add a uniquely named message in the default `app` domain.
    pub fn add_message(&mut self, role: MessageRole) -> usize {
        let base = match role {
            MessageRole::Intent => "ActionRequested",
            MessageRole::Outcome => "ActionCompleted",
        };
        let mut variant = base.to_owned();
        for suffix in 2.. {
            if !self
                .messages
                .iter()
                .any(|message| message.key.domain == "app" && message.key.variant == variant)
            {
                break;
            }
            variant = format!("{base}{suffix}");
        }
        self.messages.push(MessageDefinition {
            key: MessageKey {
                domain: "app".to_owned(),
                variant,
            },
            role,
            description: match role {
                MessageRole::Intent => "Request application work.".to_owned(),
                MessageRole::Outcome => "Report an application outcome.".to_owned(),
            },
            paired_outcome: None,
            state_updates: Vec::new(),
        });
        self.messages.len() - 1
    }

    /// Rename a message and update all pair/interaction references atomically.
    pub fn rename_message(&mut self, index: usize, new_key: MessageKey) -> bool {
        if self
            .messages
            .iter()
            .enumerate()
            .any(|(other, message)| other != index && message.key == new_key)
        {
            return false;
        }
        let Some(message) = self.messages.get_mut(index) else {
            return false;
        };
        if message.key == new_key {
            return false;
        }
        let old_key = std::mem::replace(&mut message.key, new_key.clone());
        for message in &mut self.messages {
            if message.paired_outcome.as_ref() == Some(&old_key) {
                message.paired_outcome = Some(new_key.clone());
            }
        }
        for interaction in &mut self.interactions {
            if interaction.message == old_key {
                interaction.message = new_key.clone();
            }
        }
        for mapping in &mut self.async_behavior.mappings {
            if mapping.intent == old_key {
                mapping.intent = new_key.clone();
            }
            if mapping.outcome == old_key {
                mapping.outcome = new_key.clone();
            }
        }
        true
    }

    /// Remove a message and mappings that reference it.
    pub fn remove_message(&mut self, index: usize) -> bool {
        if index >= self.messages.len() {
            return false;
        }
        let removed = self.messages.remove(index);
        self.interactions
            .retain(|interaction| interaction.message != removed.key);
        self.async_behavior
            .mappings
            .retain(|mapping| mapping.intent != removed.key && mapping.outcome != removed.key);
        for message in &mut self.messages {
            if message.paired_outcome.as_ref() == Some(&removed.key) {
                message.paired_outcome = None;
            }
        }
        true
    }

    /// Add the first paired intent not already assigned to asynchronous work.
    pub fn add_async_mapping(&mut self) -> bool {
        let candidate = self.messages.iter().find_map(|message| {
            if message.role != MessageRole::Intent
                || self
                    .async_behavior
                    .mappings
                    .iter()
                    .any(|mapping| mapping.intent == message.key)
            {
                return None;
            }
            message
                .paired_outcome
                .as_ref()
                .map(|outcome| (message.key.clone(), outcome.clone()))
        });
        let Some((intent, outcome)) = candidate else {
            return false;
        };
        self.async_behavior.mappings.push(AsyncMapping {
            intent,
            outcome,
            delay_ms: 350,
        });
        true
    }

    /// Set or clear the one message emitted for a node interaction.
    pub fn set_interaction(
        &mut self,
        node: NodeId,
        event: InteractionEvent,
        message: Option<MessageKey>,
    ) {
        self.interactions
            .retain(|binding| !(binding.node == node && binding.event == event));
        if let Some(message) = message {
            self.interactions.push(InteractionBinding {
                node,
                event,
                message,
            });
        }
    }

    /// Validate names, tree structure, dependency pinning, and typed bindings.
    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        if self.schema_version != CURRENT_SCHEMA_VERSION {
            diagnostics.push(Diagnostic::error(
                "schema_version",
                format!(
                    "schema {} is unsupported; expected {}",
                    self.schema_version, CURRENT_SCHEMA_VERSION
                ),
            ));
        }
        if !is_kebab_case(&self.crate_name) {
            diagnostics.push(Diagnostic::error(
                "crate_name",
                "use lowercase kebab-case, starting with a letter",
            ));
        } else if is_rust_keyword(&self.crate_ident()) {
            diagnostics.push(Diagnostic::error(
                "crate_name",
                "the generated Rust crate identifier cannot be a keyword",
            ));
        }
        if !is_type_name(&self.citizen_type) {
            diagnostics.push(Diagnostic::error(
                "citizen_type",
                "use a PascalCase Rust type name, starting with an uppercase letter",
            ));
        }
        if !is_snake_case(&self.citizen_id) {
            diagnostics.push(Diagnostic::error(
                "citizen_id",
                "use a lowercase snake_case stable CitizenId",
            ));
        }
        if self.title.trim().is_empty() {
            diagnostics.push(Diagnostic::error("title", "dock title cannot be empty"));
        }
        match &self.framework {
            FrameworkSource::Git {
                repository,
                revision,
            } => {
                if repository.trim().is_empty() {
                    diagnostics.push(Diagnostic::error(
                        "framework.repository",
                        "Git repository cannot be empty",
                    ));
                }
                if revision.len() != 40 || !revision.chars().all(|ch| ch.is_ascii_hexdigit()) {
                    diagnostics.push(Diagnostic::error(
                        "framework.revision",
                        "pin an exact 40-character Git commit",
                    ));
                }
            }
            FrameworkSource::Path { workspace } if workspace.trim().is_empty() => {
                diagnostics.push(Diagnostic::error(
                    "framework.workspace",
                    "workspace path cannot be empty",
                ));
            }
            FrameworkSource::Path { .. } => {}
        }
        if self.generator.generator_version != CURRENT_GENERATOR_VERSION {
            diagnostics.push(Diagnostic::error(
                "generator.generator_version",
                format!(
                    "generator contract {} is unsupported; expected {}",
                    self.generator.generator_version, CURRENT_GENERATOR_VERSION
                ),
            ));
        }
        if self.generator.backend != CURRENT_BACKEND_ID {
            diagnostics.push(Diagnostic::error(
                "generator.backend",
                format!("backend must be `{CURRENT_BACKEND_ID}`"),
            ));
        }
        if self.generator.backend_version != CURRENT_BACKEND_VERSION {
            diagnostics.push(Diagnostic::error(
                "generator.backend_version",
                format!("backend version must be `{CURRENT_BACKEND_VERSION}`"),
            ));
        }
        if self.preview.name.trim().is_empty() {
            diagnostics.push(Diagnostic::error(
                "preview.name",
                "preview fixture name cannot be empty",
            ));
        }
        if !self.theme.item_spacing.is_finite()
            || self.theme.item_spacing < 0.0
            || self.theme.item_spacing > 64.0
        {
            diagnostics.push(Diagnostic::error(
                "theme.item_spacing",
                "item spacing must be finite and between 0 and 64 points",
            ));
        }

        let mut asset_names = HashSet::new();
        let mut asset_constants = HashSet::new();
        for asset in &self.assets {
            let path = format!("asset.{}", asset.file_name);
            if !is_safe_asset_file_name(&asset.file_name) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "use a flat ASCII file name of at most 64 bytes containing only letters, digits, '.', '-', or '_'",
                ));
            }
            if !asset_names.insert(asset.file_name.as_str()) {
                diagnostics.push(Diagnostic::error(&path, "asset file names must be unique"));
            }
            if !asset_constants.insert(asset_constant_name(&asset.file_name)) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "asset file names must produce unique Rust constant names",
                ));
            }
            if asset.contents.is_empty() {
                diagnostics.push(Diagnostic::warning(&path, "asset contents are empty"));
            }
            if asset.kind == AssetKind::Svg && !asset.contents.contains("<svg") {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "SVG assets must contain an <svg element",
                ));
            }
        }

        if self.composition.enabled && self.composition.external_citizens.is_empty() {
            diagnostics.push(Diagnostic::error(
                "composition.external_citizens",
                "add at least one external Citizen or disable host composition",
            ));
        }
        let mut host_ids = HashSet::new();
        host_ids.insert(self.citizen_id.as_str());
        for (index, citizen) in self.composition.external_citizens.iter().enumerate() {
            let path = format!("composition.citizen.{index}");
            if !is_kebab_case(&citizen.crate_name)
                || is_rust_keyword(&citizen.crate_name.replace('-', "_"))
            {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "external Cargo packages must be lowercase kebab-case",
                ));
            }
            if !is_snake_case(&citizen.citizen_id) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "external CitizenId values must be lowercase snake_case",
                ));
            }
            if !is_type_name(&citizen.citizen_type) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "external Citizen types must be PascalCase Rust identifiers",
                ));
            }
            if !host_ids.insert(citizen.citizen_id.as_str()) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "composition CitizenId values must be unique",
                ));
            }
            if citizen.title.trim().is_empty() {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "external dock title cannot be empty",
                ));
            }
            if !citizen.fraction.is_finite() || citizen.fraction < 0.1 || citizen.fraction > 0.9 {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "dock fractions must be finite and between 0.1 and 0.9",
                ));
            }
        }

        let mut fields = HashMap::new();
        for field in &self.state_fields {
            if !is_snake_case(&field.name) {
                diagnostics.push(Diagnostic::error(
                    format!("state.{}", field.name),
                    "state fields must be lowercase snake_case Rust identifiers",
                ));
            }
            if fields
                .insert(field.name.as_str(), field.state_type())
                .is_some()
            {
                diagnostics.push(Diagnostic::error(
                    format!("state.{}", field.name),
                    "state field names must be unique",
                ));
            }
            if let StateValue::Number(value) = field.value
                && !value.is_finite()
            {
                diagnostics.push(Diagnostic::error(
                    format!("state.{}", field.name),
                    "numeric defaults must be finite",
                ));
            }
        }
        for field in &self.state_fields {
            match self.preview.values.get(&field.name) {
                None => diagnostics.push(Diagnostic::error(
                    format!("preview.{}", field.name),
                    "preview fixture is missing this state field",
                )),
                Some(value) if value.state_type() != field.state_type() => {
                    diagnostics.push(Diagnostic::error(
                        format!("preview.{}", field.name),
                        format!(
                            "preview value is {}, but state field is {}",
                            value.state_type().display_name(),
                            field.state_type().display_name()
                        ),
                    ));
                }
                Some(StateValue::Number(value)) if !value.is_finite() => {
                    diagnostics.push(Diagnostic::error(
                        format!("preview.{}", field.name),
                        "preview numeric values must be finite",
                    ));
                }
                Some(_) => {}
            }
        }
        for name in self.preview.values.keys() {
            if !fields.contains_key(name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    format!("preview.{name}"),
                    "preview fixture references an unknown state field",
                ));
            }
        }

        let mut message_roles = HashMap::new();
        for message in &self.messages {
            let path = format!("message.{}", message.key.display_name());
            if !is_snake_case(&message.key.domain) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "message domains must be lowercase snake_case identifiers",
                ));
            }
            if !is_type_name(&message.key.variant) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "message variants must be PascalCase Rust identifiers",
                ));
            }
            if message.description.trim().is_empty() {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "message documentation cannot be empty",
                ));
            }
            if message_roles
                .insert(message.key.clone(), message.role)
                .is_some()
            {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "message identities must be unique",
                ));
            }
            match message.role {
                MessageRole::Intent if !message.state_updates.is_empty() => {
                    diagnostics.push(Diagnostic::error(
                        &path,
                        "intent messages cannot mutate reactive state; use a paired outcome",
                    ));
                }
                MessageRole::Outcome if message.paired_outcome.is_some() => {
                    diagnostics.push(Diagnostic::error(
                        &path,
                        "only intent messages may declare a paired outcome",
                    ));
                }
                _ => {}
            }
            let mut updated_fields = HashSet::new();
            for update in &message.state_updates {
                if !updated_fields.insert(update.field.as_str()) {
                    diagnostics.push(Diagnostic::error(
                        &path,
                        format!(
                            "outcome updates state field `{}` more than once",
                            update.field
                        ),
                    ));
                }
                match fields.get(update.field.as_str()) {
                    None => diagnostics.push(Diagnostic::error(
                        &path,
                        format!(
                            "outcome update references unknown state field `{}`",
                            update.field
                        ),
                    )),
                    Some(expected) if *expected != update.value.state_type() => {
                        diagnostics.push(Diagnostic::error(
                            &path,
                            format!(
                                "outcome update for `{}` is {}, but the field is {}",
                                update.field,
                                update.value.state_type().display_name(),
                                expected.display_name()
                            ),
                        ));
                    }
                    Some(_) => {}
                }
                if let StateValue::Number(value) = update.value
                    && !value.is_finite()
                {
                    diagnostics.push(Diagnostic::error(
                        &path,
                        "outcome numeric values must be finite",
                    ));
                }
            }
        }
        for message in &self.messages {
            if let Some(outcome) = &message.paired_outcome {
                match message_roles.get(outcome) {
                    None => diagnostics.push(Diagnostic::error(
                        format!("message.{}", message.key.display_name()),
                        format!("paired outcome `{}` does not exist", outcome.display_name()),
                    )),
                    Some(MessageRole::Intent) => diagnostics.push(Diagnostic::error(
                        format!("message.{}", message.key.display_name()),
                        "paired message must have the outcome role",
                    )),
                    Some(MessageRole::Outcome) => {}
                }
            }
        }

        if self.async_behavior.enabled && self.async_behavior.mappings.is_empty() {
            diagnostics.push(Diagnostic::error(
                "async.mappings",
                "enable at least one intent/outcome mapping or disable async generation",
            ));
        }
        let mut async_intents = HashSet::new();
        for (index, mapping) in self.async_behavior.mappings.iter().enumerate() {
            let path = format!("async.mapping.{index}");
            if !async_intents.insert(&mapping.intent) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "each intent may have only one asynchronous mapping",
                ));
            }
            match message_roles.get(&mapping.intent) {
                None => diagnostics.push(Diagnostic::error(
                    &path,
                    format!(
                        "async intent `{}` does not exist",
                        mapping.intent.display_name()
                    ),
                )),
                Some(MessageRole::Outcome) => diagnostics.push(Diagnostic::error(
                    &path,
                    "asynchronous work must start from an intent",
                )),
                Some(MessageRole::Intent) => {
                    if self
                        .message(&mapping.intent)
                        .and_then(|message| message.paired_outcome.as_ref())
                        != Some(&mapping.outcome)
                    {
                        diagnostics.push(Diagnostic::error(
                            &path,
                            "async outcome must equal the intent's reference outcome",
                        ));
                    }
                }
            }
            match message_roles.get(&mapping.outcome) {
                None => diagnostics.push(Diagnostic::error(
                    &path,
                    format!(
                        "async outcome `{}` does not exist",
                        mapping.outcome.display_name()
                    ),
                )),
                Some(MessageRole::Intent) => diagnostics.push(Diagnostic::error(
                    &path,
                    "asynchronous work must return an outcome",
                )),
                Some(MessageRole::Outcome) => {}
            }
            if mapping.delay_ms == 0 || mapping.delay_ms > 600_000 {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "reference delay must be between 1 and 600000 milliseconds",
                ));
            }
        }

        let mut mapped_events = HashSet::new();
        for interaction in &self.interactions {
            let path = format!(
                "interaction.{}.{}",
                interaction.node.0,
                interaction.event.display_name()
            );
            if !mapped_events.insert((interaction.node, interaction.event)) {
                diagnostics.push(Diagnostic::error(
                    &path,
                    "each node interaction may emit only one intent",
                ));
            }
            match self.find_node(interaction.node) {
                None => diagnostics.push(Diagnostic::error(
                    &path,
                    "interaction references an unknown node",
                )),
                Some(node)
                    if !node
                        .kind
                        .supported_interactions()
                        .contains(&interaction.event) =>
                {
                    diagnostics.push(Diagnostic::error(
                        &path,
                        format!(
                            "{} does not support the {} interaction",
                            node.kind.display_name(),
                            interaction.event.display_name()
                        ),
                    ));
                }
                Some(_) => {}
            }
            match message_roles.get(&interaction.message) {
                None => diagnostics.push(Diagnostic::error(
                    &path,
                    format!(
                        "interaction intent `{}` does not exist",
                        interaction.message.display_name()
                    ),
                )),
                Some(MessageRole::Outcome) => diagnostics.push(Diagnostic::error(
                    &path,
                    "widget interactions must emit intents, never outcomes",
                )),
                Some(MessageRole::Intent) => {}
            }
        }
        for id in self.node_ids_depth_first() {
            let node = self.find_node(id).expect("collected node exists");
            if matches!(node.kind, NodeKind::Button { .. })
                && !self.interactions.iter().any(|interaction| {
                    interaction.node == id && interaction.event == InteractionEvent::Click
                })
            {
                diagnostics.push(Diagnostic::warning(
                    format!("node.{}", node.name),
                    "button is visual only until its Click interaction emits an intent",
                ));
            }
        }

        if !self.root.kind.allows_children() {
            diagnostics.push(Diagnostic::error(
                "root",
                "the Citizen root must be a semantic layout",
            ));
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        validate_node(
            &self.root,
            &fields,
            &mut ids,
            &mut names,
            &mut diagnostics,
            0,
        );
        diagnostics
    }

    /// Whether validation contains a generation-blocking error.
    pub fn has_errors(&self) -> bool {
        self.validate()
            .iter()
            .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    }

    fn next_node_id(&self) -> u64 {
        fn max_id(node: &DesignNode) -> u64 {
            node.children.iter().map(max_id).fold(node.id.0, u64::max)
        }
        max_id(&self.root) + 1
    }

    fn unique_node_name(&self, base: &str) -> String {
        let mut used = HashSet::new();
        collect_node_names(&self.root, &mut used);
        unique_name(base, |candidate| used.contains(candidate))
    }

    fn unique_state_name(&self, base: &str) -> String {
        unique_name(base, |candidate| {
            self.state_fields
                .iter()
                .any(|field| field.name == candidate)
        })
    }

    fn kind_for_palette(&mut self, item: PaletteItem) -> NodeKind {
        match item {
            PaletteItem::Column => NodeKind::Column,
            PaletteItem::Row => NodeKind::Row { wrap: false },
            PaletteItem::Grid => NodeKind::Grid {
                columns: 2,
                striped: false,
            },
            PaletteItem::Group => NodeKind::Group {
                title: "Group".to_owned(),
            },
            PaletteItem::Scroll => NodeKind::Scroll { max_height: 320.0 },
            PaletteItem::Label => NodeKind::Label {
                text: "Label".to_owned(),
            },
            PaletteItem::Heading => NodeKind::Heading {
                text: "Heading".to_owned(),
            },
            PaletteItem::Button => NodeKind::Button {
                text: "Button".to_owned(),
            },
            PaletteItem::StyledButton => NodeKind::StyledButton {
                text: "Styled Button".to_owned(),
            },
            PaletteItem::ReactiveLogger => NodeKind::ReactiveLogger,
            PaletteItem::ReactiveEditor => NodeKind::ReactiveEditor {
                content: "// Start editing\n".to_owned(),
                language: "Rust".to_owned(),
            },
            PaletteItem::LinePlot => {
                let binding = self.add_state_field(StateType::Number);
                NodeKind::LinePlot {
                    binding: Some(binding),
                }
            }
            PaletteItem::Checkbox => {
                let binding = self.add_state_field(StateType::Bool);
                NodeKind::Checkbox {
                    text: "Enabled".to_owned(),
                    binding: Some(binding),
                }
            }
            PaletteItem::TextEdit => {
                let binding = self.add_state_field(StateType::Text);
                NodeKind::TextEdit {
                    label: "Text".to_owned(),
                    hint: "Enter text".to_owned(),
                    binding: Some(binding),
                }
            }
            PaletteItem::Slider => {
                let binding = self.add_state_field(StateType::Number);
                NodeKind::Slider {
                    label: "Value".to_owned(),
                    min: 0.0,
                    max: 1.0,
                    binding: Some(binding),
                }
            }
            PaletteItem::ProgressBar => {
                let binding = self.add_state_field(StateType::Number);
                NodeKind::ProgressBar {
                    binding: Some(binding),
                    show_percentage: true,
                }
            }
            PaletteItem::Separator => NodeKind::Separator,
            PaletteItem::Spacer => NodeKind::Spacer { points: 8.0 },
        }
    }
}

fn find_node(node: &DesignNode, id: NodeId) -> Option<&DesignNode> {
    if node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|child| find_node(child, id))
}

fn find_node_mut(node: &mut DesignNode, id: NodeId) -> Option<&mut DesignNode> {
    if node.id == id {
        return Some(node);
    }
    node.children
        .iter_mut()
        .find_map(|child| find_node_mut(child, id))
}

fn collect_node_ids(node: &DesignNode, ids: &mut Vec<NodeId>) {
    ids.push(node.id);
    for child in &node.children {
        collect_node_ids(child, ids);
    }
}

fn collect_inferred_node_features(node: &DesignNode, features: &mut BTreeSet<String>) {
    match &node.kind {
        NodeKind::StyledButton { .. } => {
            features.insert("component-widgets".to_owned());
        }
        NodeKind::ReactiveLogger => {
            features.insert("component-lens".to_owned());
        }
        NodeKind::ReactiveEditor { .. } => {
            features.insert("component-quill".to_owned());
        }
        NodeKind::LinePlot { .. } => {
            features.insert("component-plot".to_owned());
        }
        _ => {}
    }
    for child in &node.children {
        collect_inferred_node_features(child, features);
    }
}

fn node_location(node: &DesignNode, id: NodeId) -> Option<(NodeId, usize)> {
    for (index, child) in node.children.iter().enumerate() {
        if child.id == id {
            return Some((node.id, index));
        }
        if let Some(location) = node_location(child, id) {
            return Some(location);
        }
    }
    None
}

fn take_node(node: &mut DesignNode, id: NodeId) -> Option<DesignNode> {
    if let Some(index) = node.children.iter().position(|child| child.id == id) {
        return Some(node.children.remove(index));
    }
    node.children
        .iter_mut()
        .find_map(|child| take_node(child, id))
}

fn remove_node(node: &mut DesignNode, id: NodeId) -> bool {
    let original_len = node.children.len();
    node.children.retain(|child| child.id != id);
    if node.children.len() != original_len {
        return true;
    }
    node.children.iter_mut().any(|child| remove_node(child, id))
}

fn rewrite_binding(node: &mut DesignNode, old_name: &str, new_name: Option<&str>) {
    if let Some(binding) = node.kind.binding_mut()
        && binding.as_deref() == Some(old_name)
    {
        *binding = new_name.map(str::to_owned);
    }
    for child in &mut node.children {
        rewrite_binding(child, old_name, new_name);
    }
}

fn collect_node_names<'a>(node: &'a DesignNode, names: &mut HashSet<&'a str>) {
    names.insert(&node.name);
    for child in &node.children {
        collect_node_names(child, names);
    }
}

fn unique_name(base: &str, exists: impl Fn(&str) -> bool) -> String {
    if !exists(base) {
        return base.to_owned();
    }
    for suffix in 2.. {
        let candidate = format!("{base}_{suffix}");
        if !exists(&candidate) {
            return candidate;
        }
    }
    unreachable!("an unused numeric suffix always exists")
}

fn validate_node<'a>(
    node: &'a DesignNode,
    fields: &HashMap<&str, StateType>,
    ids: &mut HashSet<NodeId>,
    names: &mut HashSet<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    let path = format!("node.{}", node.name);
    if !ids.insert(node.id) {
        diagnostics.push(Diagnostic::error(&path, "node IDs must be unique"));
    }
    if !is_snake_case(&node.name) {
        diagnostics.push(Diagnostic::error(
            &path,
            "semantic node names must be lowercase snake_case identifiers",
        ));
    }
    if !names.insert(&node.name) {
        diagnostics.push(Diagnostic::error(
            &path,
            "semantic node names must be unique",
        ));
    }
    if !node.kind.allows_children() && !node.children.is_empty() {
        diagnostics.push(Diagnostic::error(
            &path,
            "widgets cannot contain child nodes",
        ));
    }
    if depth > 16 {
        diagnostics.push(Diagnostic::error(
            &path,
            "semantic layout depth cannot exceed 16 levels",
        ));
    }
    if node.children.len() > 64 {
        diagnostics.push(Diagnostic::error(
            &path,
            "one layout may contain at most 64 direct children",
        ));
    }
    if node.kind.allows_children() && node.children.is_empty() {
        diagnostics.push(Diagnostic::warning(
            &path,
            "empty layout does not contribute visible UI",
        ));
    }
    if matches!(&node.kind, NodeKind::Scroll { .. }) && node.children.len() > 1 {
        diagnostics.push(Diagnostic::error(
            &path,
            "a scroll layout accepts one semantic child; wrap multiple items in a column",
        ));
    }
    if let NodeKind::Grid { columns, .. } = &node.kind
        && *columns > 0
        && !node.children.len().is_multiple_of(*columns)
    {
        diagnostics.push(Diagnostic::warning(
            &path,
            "grid has an incomplete final row",
        ));
    }
    if matches!(&node.kind, NodeKind::Grid { .. })
        && node
            .children
            .iter()
            .any(|child| child.kind.allows_children())
    {
        diagnostics.push(Diagnostic::error(
            &path,
            "grid children must be leaf widgets; compose nested layouts outside the grid",
        ));
    }
    match &node.kind {
        NodeKind::Grid { columns, .. } if *columns == 0 => {
            diagnostics.push(Diagnostic::error(
                &path,
                "grid columns must be at least one",
            ));
        }
        NodeKind::Scroll { max_height } if !max_height.is_finite() || *max_height < 0.0 => {
            diagnostics.push(Diagnostic::error(
                &path,
                "scroll height must be finite and non-negative",
            ));
        }
        NodeKind::Slider { min, max, .. } if !min.is_finite() || !max.is_finite() || min >= max => {
            diagnostics.push(Diagnostic::error(
                &path,
                "slider range must be finite with min < max",
            ));
        }
        NodeKind::Spacer { points } if !points.is_finite() || *points < 0.0 => {
            diagnostics.push(Diagnostic::error(
                &path,
                "spacer points must be finite and non-negative",
            ));
        }
        NodeKind::ReactiveEditor { language, .. } if language.trim().is_empty() => {
            diagnostics.push(Diagnostic::error(
                &path,
                "Quill editor language cannot be empty",
            ));
        }
        _ => {}
    }

    if let Some(expected) = node.kind.expected_binding() {
        match node.kind.binding() {
            None => diagnostics.push(Diagnostic::error(
                &path,
                format!(
                    "{} requires a Dynamic<{}> binding",
                    node.kind.display_name(),
                    expected.display_name()
                ),
            )),
            Some(binding) => match fields.get(binding) {
                None => diagnostics.push(Diagnostic::error(
                    &path,
                    format!("binding `{binding}` does not name a state field"),
                )),
                Some(actual) if *actual != expected => diagnostics.push(Diagnostic::error(
                    &path,
                    format!(
                        "binding `{binding}` is {}, but this widget requires {}",
                        actual.display_name(),
                        expected.display_name()
                    ),
                )),
                Some(_) => {}
            },
        }
    }

    for child in &node.children {
        validate_node(child, fields, ids, names, diagnostics, depth + 1);
    }
}

fn is_safe_asset_file_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with('.')
        && !value.contains("..")
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
}

fn asset_constant_name(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_uppercase()
            } else {
                '_'
            }
        })
        .collect()
}

fn is_kebab_case(value: &str) -> bool {
    is_delimited_lower_identifier(value, '-')
}

fn is_snake_case(value: &str) -> bool {
    !is_rust_keyword(value) && is_delimited_lower_identifier(value, '_')
}

fn is_delimited_lower_identifier(value: &str, delimiter: char) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_lowercase())
        && !value.ends_with(delimiter)
        && !value.contains(&format!("{delimiter}{delimiter}"))
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == delimiter)
}

fn is_type_name(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_alphanumeric())
        && !is_rust_keyword(value)
}

fn is_rust_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_project_is_generation_valid() {
        let diagnostics = CitizenProject::default().validate();
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
            "{diagnostics:#?}"
        );
    }

    #[test]
    fn every_curated_template_is_valid_and_round_trips() {
        for template in CitizenTemplate::ALL {
            let project = CitizenProject::from_template(*template);
            let diagnostics = project.validate();
            assert!(
                diagnostics
                    .iter()
                    .all(|diagnostic| diagnostic.severity != DiagnosticSeverity::Error),
                "{} template diagnostics: {diagnostics:#?}",
                template.display_name()
            );
            let json = project.to_json_pretty().unwrap();
            assert_eq!(CitizenProject::from_json(&json).unwrap(), project);
        }
    }

    #[test]
    fn semantic_design_infers_component_and_host_features() {
        let mut project = CitizenProject::from_template(CitizenTemplate::Editor);
        project.composition.enabled = true;
        project.composition.external_citizens.push(HostCitizen {
            crate_name: "logger-citizen".to_owned(),
            citizen_type: "LoggerCitizen".to_owned(),
            citizen_id: "logger".to_owned(),
            title: "Logger".to_owned(),
            placement: DockPlacement::Right,
            fraction: 0.3,
        });
        let features = project.inferred_features();
        assert!(features.contains("component-quill"));
        assert!(features.contains("component-widgets"));
        assert!(features.contains("embedded-assets"));
        assert!(features.contains("host-composition"));
        assert!(!project.has_errors());
    }

    #[test]
    fn unsafe_assets_and_incomplete_compositions_are_rejected() {
        let mut project = CitizenProject::default();
        project.assets.push(AssetDefinition {
            file_name: "../secret.txt".to_owned(),
            kind: AssetKind::Text,
            contents: "nope".to_owned(),
        });
        project.composition.enabled = true;
        assert!(project.has_errors());
    }

    #[test]
    fn preview_fixture_is_independent_from_generated_defaults() {
        let project = CitizenProject::default();
        assert_eq!(
            project.state_fields[1].value,
            StateValue::Text("Citizen".to_owned())
        );
        assert_eq!(
            project.preview.values["display_name"],
            StateValue::Text("Preview Citizen".to_owned())
        );
    }

    #[test]
    fn project_json_round_trips_exactly() {
        let project = CitizenProject::default();
        let json = project.to_json_pretty().unwrap();
        assert_eq!(CitizenProject::from_json(&json).unwrap(), project);
    }

    #[test]
    fn incompatible_schema_is_rejected() {
        let project = CitizenProject {
            schema_version: CURRENT_SCHEMA_VERSION + 1,
            ..CitizenProject::default()
        };
        let json = project.to_json_pretty().unwrap();
        assert!(CitizenProject::from_json(&json).is_err());
    }

    #[test]
    fn unknown_schema_fields_are_rejected() {
        let json = CitizenProject::default()
            .to_json_pretty()
            .unwrap()
            .replacen("{", "{\n  \"unexpected\": true,", 1);
        assert!(CitizenProject::from_json(&json).is_err());
    }

    #[test]
    fn keyword_crate_identifier_is_rejected() {
        let project = CitizenProject {
            crate_name: "crate".to_owned(),
            ..CitizenProject::default()
        };
        assert!(project.has_errors());
    }

    #[test]
    fn palette_bound_widgets_create_typed_fields() {
        let mut project = CitizenProject::default();
        let original = project.state_fields.len();
        let id = project.add_palette_item(Some(project.root.id), PaletteItem::Checkbox);
        let node = project.find_node(id).unwrap();
        let binding = node.kind.binding().unwrap();
        assert_eq!(project.state_fields.len(), original + 1);
        assert_eq!(
            project
                .state_fields
                .iter()
                .find(|field| field.name == binding)
                .unwrap()
                .state_type(),
            StateType::Bool
        );
    }

    #[test]
    fn generated_numeric_suffixes_remain_valid_identifiers() {
        let mut project = CitizenProject::default();
        project.add_palette_item(Some(project.root.id), PaletteItem::Checkbox);
        project.add_palette_item(Some(project.root.id), PaletteItem::Slider);
        assert!(!project.has_errors());
        assert!(
            project
                .state_fields
                .iter()
                .any(|field| field.name == "enabled_2")
        );
        assert!(
            project
                .state_fields
                .iter()
                .any(|field| field.name == "value")
        );
    }

    #[test]
    fn widgets_added_below_widgets_fall_back_to_root() {
        let mut project = CitizenProject::default();
        let widget_id = project.root.children[0].id;
        let added = project.add_palette_item(Some(widget_id), PaletteItem::Label);
        assert!(project.root.children.iter().any(|node| node.id == added));
    }

    #[test]
    fn root_cannot_be_removed() {
        let mut project = CitizenProject::default();
        assert!(!project.remove_node(project.root.id));
    }

    #[test]
    fn wrong_binding_type_is_an_error() {
        let mut project = CitizenProject::default();
        let checkbox = project
            .root
            .children
            .iter_mut()
            .find(|node| matches!(node.kind, NodeKind::Checkbox { .. }))
            .unwrap();
        *checkbox.kind.binding_mut().unwrap() = Some("display_name".to_owned());
        assert!(project.has_errors());
    }

    #[test]
    fn state_rename_updates_all_bindings() {
        let mut project = CitizenProject::default();
        assert!(project.rename_state_field(2, "intensity".to_owned()));
        let bindings = project
            .root
            .children
            .iter()
            .filter_map(|node| node.kind.binding())
            .collect::<Vec<_>>();
        assert_eq!(
            bindings
                .iter()
                .filter(|binding| **binding == "intensity")
                .count(),
            2
        );
        assert!(!bindings.contains(&"level"));
        assert!(project.preview.values.contains_key("intensity"));
        assert!(!project.preview.values.contains_key("level"));
    }

    #[test]
    fn state_removal_clears_all_bindings() {
        let mut project = CitizenProject::default();
        assert!(project.remove_state_field(2));
        assert!(
            project
                .root
                .children
                .iter()
                .filter(|node| matches!(
                    node.kind,
                    NodeKind::Slider { .. } | NodeKind::ProgressBar { .. }
                ))
                .all(|node| node.kind.binding().is_none())
        );
        assert!(!project.preview.values.contains_key("level"));
    }

    #[test]
    fn tree_moves_reparent_reorder_and_reject_cycles() {
        let mut project = CitizenProject::default();
        let root = project.root.id;
        let row = project.add_palette_item(Some(root), PaletteItem::Row);
        let group = project.add_palette_item(Some(root), PaletteItem::Group);
        let label = project.add_palette_item(Some(root), PaletteItem::Label);

        project
            .move_node_relative(label, group, MovePlacement::Inside)
            .unwrap();
        assert_eq!(project.parent_id(label), Some(group));
        project
            .move_node_relative(group, row, MovePlacement::Before)
            .unwrap();
        let group_index = project
            .root
            .children
            .iter()
            .position(|node| node.id == group)
            .unwrap();
        let row_index = project
            .root
            .children
            .iter()
            .position(|node| node.id == row)
            .unwrap();
        assert!(group_index < row_index);

        let nested = project.add_palette_item(Some(group), PaletteItem::Column);
        assert!(
            project
                .move_node_relative(group, nested, MovePlacement::Inside)
                .is_err()
        );
    }

    #[test]
    fn indent_and_outdent_are_inverse_document_commands() {
        let mut project = CitizenProject::default();
        let root = project.root.id;
        let row = project.add_palette_item(Some(root), PaletteItem::Row);
        let label = project.add_palette_item(Some(root), PaletteItem::Label);
        project.indent_node(label).unwrap();
        assert_eq!(project.parent_id(label), Some(row));
        project.outdent_node(label).unwrap();
        assert_eq!(project.parent_id(label), Some(root));
    }

    #[test]
    fn nested_settings_document_round_trips_without_loss() {
        let mut project = CitizenProject {
            crate_name: "settings-citizen".to_owned(),
            citizen_type: "SettingsCitizen".to_owned(),
            citizen_id: "settings".to_owned(),
            title: "Settings".to_owned(),
            ..CitizenProject::default()
        };
        let root = project.root.id;
        let group = project.add_palette_item(Some(root), PaletteItem::Group);
        let row = project.add_palette_item(Some(group), PaletteItem::Row);
        project.add_palette_item(Some(row), PaletteItem::Checkbox);
        project.add_palette_item(Some(group), PaletteItem::TextEdit);
        project.add_palette_item(Some(group), PaletteItem::Slider);
        let json = project.to_json_pretty().unwrap();
        let decoded = CitizenProject::from_json(&json).unwrap();
        assert_eq!(decoded, project);
        assert!(!decoded.has_errors());
    }

    #[test]
    fn message_rename_propagates_pairs_and_interactions() {
        let mut project = CitizenProject::default();
        let renamed = MessageKey {
            domain: "preferences".to_owned(),
            variant: "SaveRequested".to_owned(),
        };
        assert!(project.rename_message(0, renamed.clone()));
        assert_eq!(project.interactions[0].message, renamed);
        assert_eq!(project.async_behavior.mappings[0].intent, renamed);

        let renamed_outcome = MessageKey {
            domain: "preferences".to_owned(),
            variant: "SaveCompleted".to_owned(),
        };
        assert!(project.rename_message(1, renamed_outcome.clone()));
        assert_eq!(project.messages[0].paired_outcome, Some(renamed_outcome));
        assert_eq!(
            project.async_behavior.mappings[0].outcome,
            project.messages[0].paired_outcome.clone().unwrap()
        );
        assert!(!project.has_errors());
    }

    #[test]
    fn removing_a_subtree_removes_all_interaction_bindings() {
        let mut project = CitizenProject::default();
        let apply_button = project.interactions[0].node;
        assert!(project.remove_node(apply_button));
        assert!(project.interactions.is_empty());
        assert!(!project.has_errors());
    }

    #[test]
    fn interactions_may_emit_only_supported_intents() {
        let mut project = CitizenProject::default();
        let button = project.interactions[0].node;
        let outcome = project.messages[1].key.clone();
        project.set_interaction(button, InteractionEvent::Click, Some(outcome));
        assert!(project.has_errors());

        let mut project = CitizenProject::default();
        let button = project.interactions[0].node;
        let intent = project.messages[0].key.clone();
        project.set_interaction(button, InteractionEvent::Change, Some(intent));
        assert!(project.has_errors());
    }

    #[test]
    fn intents_cannot_hide_state_mutation_and_outcomes_remain_typed() {
        let mut project = CitizenProject::default();
        project.messages[0].state_updates.push(StateAssignment {
            field: "enabled".to_owned(),
            value: StateValue::Bool(true),
        });
        assert!(project.has_errors());

        let mut project = CitizenProject::default();
        project.messages[1].state_updates.push(StateAssignment {
            field: "enabled".to_owned(),
            value: StateValue::Text("wrong".to_owned()),
        });
        assert!(project.has_errors());
    }

    #[test]
    fn async_mappings_require_a_paired_intent_and_outcome() {
        let mut project = CitizenProject::default();
        project.async_behavior.enabled = true;
        assert!(!project.has_errors());

        project.async_behavior.mappings[0].outcome = MessageKey {
            domain: "missing".to_owned(),
            variant: "MissingOutcome".to_owned(),
        };
        assert!(project.has_errors());

        project.async_behavior.mappings.clear();
        assert!(project.has_errors());
    }

    #[test]
    fn deleting_a_message_removes_its_async_mapping() {
        let mut project = CitizenProject::default();
        assert!(project.remove_message(0));
        assert!(project.async_behavior.mappings.is_empty());
    }

    #[test]
    fn names_are_made_unique() {
        let mut project = CitizenProject::default();
        let first = project.add_palette_item(None, PaletteItem::Label);
        let second = project.add_palette_item(None, PaletteItem::Label);
        assert_ne!(
            project.find_node(first).unwrap().name,
            project.find_node(second).unwrap().name
        );
    }
}
