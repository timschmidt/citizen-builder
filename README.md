# egui-rad-builder

`egui-rad-builder` is an early-stage RAD GUI builder for Rust [`egui`](https://github.com/emilk/egui) applications. It provides a visual drag-and-drop canvas, widget inspector, JSON project save/load, and Rust code generation for `eframe`/`egui` apps.

Current crate version: `0.2.1`

The generated UIs are intended to compile and run, but the project is still under active development. Expect rough edges in layout fidelity, widget nesting, and generated-code ergonomics.

<img src="doc/egui-rad-builder-screenshot.png" width=45% alt="egui RAD builder screenshot"/> <img src="doc/ui-screenshot.png" width=45% alt="Generated UI"/>

## Current Status

Implemented:

- Drag-and-drop widget palette with categorized widgets.
- Central canvas plus optional top, bottom, left, and right panel areas.
- Inspector for widget text, sizing, position, values, colors, item lists, URLs, tooltips, enabled state, and widget-specific options.
- Multi-select, copy/paste, duplicate, delete, select all, alignment, distribution, and match-size actions.
- Configurable grid display and grid snapping.
- Edit and preview modes. Preview mode hides selection handles and lets widgets behave more like the generated app.
- Project save/load through native file dialogs using JSON-based `.json` or `.rad` files.
- JSON export/import through the generated output panel.
- Rust code generation with single-file, separate-file, and UI-function-only output modes.
- Optional generated-code comments, auto-generation, and syntax-highlighted code preview.
- CI checks for `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test`, and release build.

Supported widget types include:

- Basic: Label, Button, Image + Text Button, Checkbox, Link, Hyperlink, Selectable Label, Separator
- Text/display: Heading, Small, Monospace, Progress Bar, Spinner, Image, Placeholder
- Input: TextEdit, Text Area, Password, Slider, Drag Value, Combo Box, Radio Group, Date Picker, Angle Selector, Color Picker
- Containers/layout: Group, Scroll Box, Columns, Tab Bar, Window, Collapsing Header
- Advanced: Menu Button, Tree, Code Editor

## Build

```shell
cargo build
```

On Linux, native GUI dependencies for `eframe`/`winit` may be required. The CI environment installs:

```shell
sudo apt-get install -y libxkbcommon-dev libwayland-dev
```

## Run

```shell
cargo run
```

## Test

```shell
cargo test
```

Current local result: `20 passed; 0 failed`.

## Basic Workflow

1. Run the builder with `cargo run`.
2. Drag widgets from the palette into the central canvas or enabled panel areas.
3. Select a widget and edit its properties in the Inspector tab.
4. Use the Settings menu to change canvas size, grid size, panel visibility, and code generation options.
5. Use Preview mode or `F5` to interact with the designed UI without selection handles.
6. Click `Generate Code` or press `Ctrl+G`.
7. Copy the generated Rust into an app, or use the separate-file output as a starting point.

Generated single-file apps currently expect dependencies similar to:

```toml
[dependencies]
chrono = "0.4.42"
eframe = "0.33.0"
egui = "0.33.0"
egui_extras = { version = "0.33.0", features = ["chrono"] }
```

The builder itself also depends on `rfd`, `serde`, `serde_json`, and `syntect` for file dialogs, project serialization, and highlighted code preview.

## Shortcuts

| Shortcut | Action |
| --- | --- |
| `Arrow Keys` | Nudge selected widgets by the current grid size |
| `Delete` / `Backspace` | Delete selected widgets |
| `Ctrl+C` | Copy the first selected widget |
| `Ctrl+V` | Paste copied widget |
| `Ctrl+D` | Duplicate selected widgets |
| `Ctrl+G` | Generate code |
| `[` | Send selected widgets backward in z-order |
| `]` | Bring selected widgets forward in z-order |
| `F5` | Toggle Edit/Preview mode |

## Known Limitations

- This is still a builder app, not a complete visual programming environment.
- Widget nesting is limited. Container widgets generate representative layout code, but arbitrary child composition is not yet fully modeled.
- Generated code is useful boilerplate, but complex apps will still need hand editing for state, events, data binding, and business logic.
- Some generated widgets are placeholders or simple approximations of richer egui behavior.
- The main application implementation is still concentrated in `src/app.rs` and will likely be split as the project matures.

## Near-Term Ideas

- Improve nested layouts and child widget ownership.
- Add richer event/action modeling for generated apps.
- Expand image, table, plot, modal, tooltip, and context-menu support.
- Refine generated project export and dependency handling.
- Consider a fuller docking/workspace system if the current panel and tab UI becomes limiting.

Issues and PRs are welcome.
