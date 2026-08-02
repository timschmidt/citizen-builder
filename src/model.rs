//! Versioned design model for one reusable egui_mobius Citizen.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// Current on-disk project schema.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

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
    /// Typed reactive state required by this Citizen.
    pub state_fields: Vec<StateField>,
    /// Root of the semantic immediate-mode layout tree.
    pub root: DesignNode,
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

/// Supported semantic layouts and Level 1 widgets.
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
    /// Level 1 button. It becomes actionable when Level 2 messages are added.
    Button {
        /// Button text.
        text: String,
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
            Self::Checkbox { .. } => "Checkbox",
            Self::TextEdit { .. } => "Text Edit",
            Self::Slider { .. } => "Slider",
            Self::ProgressBar { .. } => "Progress Bar",
            Self::Separator => "Separator",
            Self::Spacer { .. } => "Spacer",
        }
    }

    fn expected_binding(&self) -> Option<StateType> {
        match self {
            Self::Checkbox { .. } => Some(StateType::Bool),
            Self::TextEdit { .. } => Some(StateType::Text),
            Self::Slider { .. } | Self::ProgressBar { .. } => Some(StateType::Number),
            _ => None,
        }
    }

    /// Current binding name, if this kind is state-bound.
    pub fn binding(&self) -> Option<&str> {
        match self {
            Self::Checkbox { binding, .. }
            | Self::TextEdit { binding, .. }
            | Self::Slider { binding, .. }
            | Self::ProgressBar { binding, .. } => binding.as_deref(),
            _ => None,
        }
    }

    /// Mutable binding slot for state-bound widgets.
    pub fn binding_mut(&mut self) -> Option<&mut Option<String>> {
        match self {
            Self::Checkbox { binding, .. }
            | Self::TextEdit { binding, .. }
            | Self::Slider { binding, .. }
            | Self::ProgressBar { binding, .. } => Some(binding),
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
        match self.value {
            StateValue::Bool(_) => StateType::Bool,
            StateValue::Text(_) => StateType::Text,
            StateValue::Number(_) => StateType::Number,
        }
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
            Self::Checkbox => "checkbox",
            Self::TextEdit => "text_edit",
            Self::Slider => "slider",
            Self::ProgressBar => "progress_bar",
            Self::Separator => "separator",
            Self::Spacer => "spacer",
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

        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            crate_name: "example-citizen".to_owned(),
            citizen_type: "ExampleCitizen".to_owned(),
            citizen_id: "example".to_owned(),
            title: "Example Citizen".to_owned(),
            description: "A reusable Citizen generated by citizen-builder.".to_owned(),
            framework: FrameworkSource::default(),
            state_fields,
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
        remove_node(&mut self.root, id)
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
            value,
        });
        name
    }

    /// Rename a state field and update every widget binding atomically.
    pub fn rename_state_field(&mut self, index: usize, new_name: String) -> bool {
        let Some(field) = self.state_fields.get_mut(index) else {
            return false;
        };
        if field.name == new_name {
            return false;
        }
        let old_name = std::mem::replace(&mut field.name, new_name.clone());
        rewrite_binding(&mut self.root, &old_name, Some(&new_name));
        true
    }

    /// Remove a state field and clear bindings that referenced it.
    pub fn remove_state_field(&mut self, index: usize) -> bool {
        if index >= self.state_fields.len() {
            return false;
        }
        let removed = self.state_fields.remove(index);
        rewrite_binding(&mut self.root, &removed.name, None);
        true
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

        if !self.root.kind.allows_children() {
            diagnostics.push(Diagnostic::error(
                "root",
                "the Citizen root must be a semantic layout",
            ));
        }
        let mut ids = HashSet::new();
        let mut names = HashSet::new();
        validate_node(&self.root, &fields, &mut ids, &mut names, &mut diagnostics);
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
        NodeKind::Button { .. } => diagnostics.push(Diagnostic::warning(
            &path,
            "Level 1 buttons are visual only; bind an AppMessage in the Level 2 milestone",
        )),
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
        validate_node(child, fields, ids, names, diagnostics);
    }
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
