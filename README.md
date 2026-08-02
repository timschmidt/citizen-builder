# citizen-builder

`citizen-builder` is a visual editor for creating one reusable
[`egui_mobius`](https://saturn77.github.io/egui_mobius/) Citizen at a time. It
models the Citizen's identity, semantic layout, and typed reactive state, then
exports a standalone Rust library crate with native and WASM preview hosts.

The current `0.3.0` vertical slice is a working Level 1 Citizen builder. It is
an intentionally new product: only the versioned Citizen project schema is
accepted, and no legacy project or generic application format is imported.

## What works today

- One Citizen per project, with editable crate name, Rust type, stable
  `CitizenId`, title, and documentation.
- A child-owning semantic layout tree: columns, rows, grids, groups, and scroll
  areas.
- Labels, headings, buttons, checkboxes, text edits, sliders, progress bars,
  separators, and spacers.
- Typed `bool`, `String`, and `f32` state fields emitted as `Dynamic<T>` values.
- Type-checked widget bindings and validation before generation.
- A live Level 1 preview driven by the same project model as generation.
- A dogfooded editor shell whose Outline, Canvas, Inspector, and Generated Files
  panels are Citizens registered through a `Dispatcher` and hosted by
  `egui_dock`.
- Deterministic generation of a standalone library crate, complete native/WASM
  preview, `citizen.json`, and host-integration guide.
- Exact framework revision pinning, with an editable local workspace/path mode.

Buttons are visual in the Level 1 output. Discrete click/change actions are the
next Level 2 milestone.

## Requirements

- Rust 1.92 or newer
- Native Linux builds may require `libxkbcommon-dev` and `libwayland-dev`
- The `wasm32-unknown-unknown` target and
  [Trunk](https://trunkrs.dev/) for browser development

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

1. Set Citizen and dependency metadata in the Inspector.
2. Select a layout node in the Outline and add layout or widget children.
3. Declare typed reactive fields and bind compatible widgets to them.
4. Resolve errors shown above the live Canvas preview.
5. Inspect the deterministic output in Generated Files.
6. Export into a new directory; existing data is never overwritten.
7. Run the generated native preview or its Trunk WASM preview.
8. Follow `host-integration.md` to register the Citizen in an application.

An exported crate contains:

| File | Purpose |
| --- | --- |
| `src/lib.rs` | Reusable `Citizen` implementation and typed state contract |
| `src/bin/preview.rs` | Native/WASM `Dispatcher` and `egui_dock` reference host |
| `citizen.json` | Exact versioned source project |
| `host-integration.md` | Required host fields, registration, and render wiring |
| `Cargo.toml` | Reproducible dependencies and preview feature |
| `index.html`, `Trunk.toml` | Browser preview entry point |

## Quality checks

```shell
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo check --target wasm32-unknown-unknown
```

CI also materializes a generated Citizen covering every supported node and
compiles its native and WASM preview targets. See [ROADMAP.md](ROADMAP.md) for
the implemented architecture, acceptance criteria, and next milestones.
