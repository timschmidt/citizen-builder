# citizen-builder

`citizen-builder` is a Citizen-first visual editor for
[`egui_mobius`](https://saturn77.github.io/egui_mobius/). One project describes
exactly one reusable Citizen and exports it as a standalone Rust library crate,
along with a dogfooded native/WASM preview host and explicit host-integration
guidance.

Version `0.4.0` implements the four planned milestones: typed reactive state,
intent/outcome messages, cancellable async work, curated ecosystem components,
assets and themes, six starting templates, and visual multi-Citizen host
composition. It is a new product with its own strict schema; it carries no
`egui-rad-builder` compatibility layer.

## What works

- Six generation-valid templates: Settings, Logger, Editor, Plot, File Browser,
  and Backend Control.
- A semantic layout tree with column, row, grid, group, and constrained scroll
  layouts; drag/reparent/reorder, keyboard editing, selection feedback, and
  document-level undo/redo.
- Labels, headings, standard and styled buttons, checkboxes, text edits,
  sliders, progress bars, separators, spacers, reactive logging, reactive
  editing, and line plotting.
- Typed `bool`, `String`, and `f32` fields emitted as `Dynamic<T>`, with separate
  editable preview fixtures and type-checked bindings.
- Domain-grouped intent and outcome messages for click, change, and submit
  interactions. Generated Citizens expose an outbox; the reference host drains
  it, routes work, and applies outcomes on the UI thread.
- Optional cancellable async mappings. Native output uses `egui_mobius`
  `Signal`/`Slot` plus `AsyncDispatcher`; WASM output uses abortable local
  futures. Citizen deactivation cancels outstanding work.
- Curated `egui_lens`, `egui_quill`, `egui_plot`, and
  `egui_mobius_widgets` integration with inferred Cargo features. Because the
  current widgets crate pulls a native Tokio stack, styled buttons use the
  ecosystem wrapper natively and a functional standard-egui fallback on WASM.
- Editable dark/light themes, accent and panel colors, spacing, and embedded
  UTF-8 text/SVG assets.
- Visual authoring of external Citizen tabs and left/right/above/below dock
  splits. The exported library remains one Citizen; generated host scaffolding
  composes neighboring compile-time Citizen crates.
- Strict current-schema JSON open/save/import, deterministic generation,
  validation before export, and refusal to overwrite an existing directory.

The builder dogfoods the same architecture: Outline, Canvas, Inspector, and
Generated Files are Citizens registered through a `Dispatcher`, share reactive
project state, and render through `egui_dock`.

## Requirements

- Rust 1.92 or newer
- Native Linux development packages required by `winit`/Wayland on your system
- `wasm32-unknown-unknown` and [Trunk](https://trunkrs.dev/) for browser work

## Run the builder

Native:

```shell
cargo run
```

WASM:

```shell
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve --open
```

## Authoring workflow

1. Choose a template and create a Citizen.
2. Edit crate identity, framework source, theme, assets, and optional host
   composition in the Inspector.
3. Add and reorganize semantic layouts and widgets in the Outline.
4. Declare reactive fields, bind compatible widgets, define domain messages,
   and connect interactions or async mappings.
5. Exercise fixture data and host-layout placeholders in the live Canvas.
6. Resolve generation-blocking diagnostics and inspect every deterministic
   output in Generated Files.
7. Export into a new directory, then run its native preview or Trunk WASM
   preview.
8. Follow the generated integration guides to register, activate, render, and
   route the Citizen in its real host.

## Generated crate

| File | Purpose |
| --- | --- |
| `src/lib.rs` | Reusable `Citizen`, typed state contract, widgets, and outbox |
| `src/messages.rs` | Domain message enums and top-level `AppMessage` |
| `src/backend.rs` | Synchronous reference routing and UI-thread outcomes |
| `src/async_backend.rs` | Optional native/WASM cancellable async routing |
| `src/theme.rs` | Generated egui theme application |
| `src/assets.rs`, `assets/*` | Optional embedded UTF-8 project assets |
| `src/bin/preview.rs` | Native/WASM Dispatcher, dock, lifecycle, and message host |
| `citizen.json` | Exact current-schema source project |
| `host-integration.md` | Single-Citizen host wiring and routing contract |
| `host-composition.md` | Optional multi-Citizen fields, tabs, and dock scaffold |
| `Cargo.toml` | Exact framework source and inferred target-aware features |
| `index.html`, `Trunk.toml` | Browser preview entry point |

Generated dependencies default to the exact egui_mobius revision recorded in
the project. A project may instead select an explicit local egui_mobius
workspace path.

## Source ingestion and compatibility

`citizen.json` is the authoritative source format. The builder opens, saves,
imports, and exports only the current schema and rejects unknown fields or
incompatible schema versions. Arbitrary Rust round-tripping is intentionally
out of scope; generated Rust is an output, not a second editable source model.

## Quality checks

```shell
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release --all-features
cargo check --target wasm32-unknown-unknown --all-features
```

Generator verification additionally materializes template and kitchen-sink
Citizens, checks rustfmt output, runs strict native all-feature compilation and
generated tests, and compiles the all-feature WASM preview. See
[ROADMAP.md](ROADMAP.md) for the acceptance record and continuing maintenance
priorities.
