# citizen-builder roadmap

- Status: first Level 1 vertical slice implemented
- Builder version: 0.3.0
- Framework baseline: egui_mobius 0.5, egui 0.35, egui_dock 0.20
- Pinned framework revision: `cc286df73b6cb3a015b3d0c5159aeaf3f41510cc`

## Product contract

- A project describes exactly one reusable Citizen.
- Its primary output is a standalone Citizen library crate.
- Every output also includes a native/WASM preview host and explicit host
  integration guidance.
- Level 1 means typed `Dynamic<T>` state and Dispatcher lifecycle integration.
- Level 2 adds discrete application messages; Level 3 adds signals, slots, and
  asynchronous backends.
- Git dependencies are pinned to an exact revision. A project may instead use
  an explicit local egui_mobius workspace path.
- Only the current versioned Citizen schema is supported. There is no legacy
  import or compatibility layer.
- The builder must dogfood the same Citizen, Dispatcher, reactive-state, dock,
  native, and WASM architecture it generates.

## Target development loop

```text
Citizen metadata + typed state + semantic layout
    -> structural and binding validation
    -> live docked preview
    -> deterministic standalone crate
    -> native and WASM compile gates
    -> host integration
```

Source generation is the target. Runtime dynamic plug-ins are not: Citizens are
ordinary compile-time Rust dependencies, avoiding an unstable cross-crate ABI.

## Implemented vertical slice

### Versioned project model

- Schema version 1 with strict unknown-field rejection.
- One Citizen identity and one child-owning semantic layout tree.
- Layout nodes for column, row, grid, group, and vertical scroll.
- Level 1 widget nodes for static content and `bool`, `String`, or `f32`
  bindings.
- Stable internal node IDs and semantic names for deterministic output.
- Validation for package/type/field names, duplicate identities, malformed
  trees, binding type mismatches, numeric ranges, and framework sources.
- State rename and deletion operations propagate safely through bindings.

### First-class editor shell

- Outline Citizen for hierarchy navigation and palette insertion.
- Canvas Citizen for live reactive preview and diagnostics.
- Inspector Citizen for metadata, dependency source, state, node properties,
  and bindings.
- Generated Files Citizen for deterministic file inspection, clipboard export,
  and safe new-directory export.
- All four panels are registered with `egui_citizen::Dispatcher`, share
  `Dynamic<CitizenProject>` state, and render in `egui_dock`.

### Standalone crate generator

- Library implementing `egui_citizen::Citizen` with a typed `Dynamic<T>` state
  contract.
- Complete reference host with registration, activation, dock rendering, and
  Dispatcher draining.
- Native `eframe` and WASM `WebRunner` entry points behind one preview feature.
- Exact Git revision or explicit local-path dependencies.
- `citizen.json`, Cargo manifest, README, Trunk files, and focused host wiring
  instructions.
- Rust syntax parsing and deterministic formatting before display or export.
- Refusal to export over an existing directory.

## Milestone 1 — harden the Level 1 MVP

This is the immediate release target.

- Add tree drag/reparent/reorder interactions while retaining palette insertion
  as the accessible fallback.
- Add undo/redo as document-level commands.
- Add focused golden snapshots for every emitted file and representative node
  combination.
- Improve keyboard navigation and selection feedback.
- Add editable preview fixtures without mixing fixture values into generated
  defaults.
- Add explicit generator/backend version metadata to the document.
- Exercise save, load, edit, and export through GUI integration tests where
  practical.

Exit criteria:

- A Settings Citizen with nested layouts and all three state types survives a
  save/load/edit round trip without structural loss.
- The exported crate is formatter-clean and compiles as a library plus native
  and WASM preview.
- Invalid identities, trees, sources, and bindings block export with actionable
  diagnostics.
- The builder itself passes native and WASM quality gates.

## Milestone 2 — Level 2 messages

- Add an interaction editor for click, change, and submit events.
- Model domain-grouped intent and outcome messages separately from continuous
  reactive state.
- Generate `AppMessage` enums, Citizen outboxes, host drain/route scaffolding,
  and a synchronous backend reference.
- Add validation that prevents event bindings from silently becoming mutable
  shared state.
- Offer `egui_lens` logging in the preview host.

Exit criteria:

- A generated Citizen can request work, receive an outcome, update reactive
  state, and expose the complete message path in its reference host.

## Milestone 3 — Level 3 async behavior

- Add opt-in signal/slot and `AsyncDispatcher` mappings.
- Generate cancellable native background work with UI-thread result draining.
- Define WASM-compatible async alternatives and correct target feature gates.
- Add lifecycle-aware cancellation on Citizen deactivation.

Exit criteria:

- A generated Citizen starts, observes, and cancels background work without
  accessing egui state away from the UI thread on either advertised target.

## Milestone 4 — ecosystem acceleration

- Templates for settings, logger, editor, plot, file browser, and backend
  control Citizens.
- Curated wrappers for stable egui_mobius ecosystem components.
- Assets, themes, inferred features, and richer semantic layout constraints.
- Multi-Citizen host composition and visual dock layout authoring, layered on
  the same single-Citizen generator contract.
- Optional source ingestion only after generated output is stable; arbitrary
  Rust round-tripping is not required.

## Quality gates

Every change retains:

```shell
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
cargo check --target wasm32-unknown-unknown
```

Generator changes additionally materialize a fixture and verify:

- deterministic file sets;
- valid, formatter-clean Rust;
- native library and preview compilation;
- WASM preview compilation;
- exact dependency-source emission; and
- rejection of invalid projects before any filesystem write.

## Risks and controls

- **Framework API churn:** isolate framework-specific emission, pin exact
  revisions, and update against compiling fixtures.
- **Workspace-only Citizen crates:** make Git versus local workspace source an
  explicit saved setting; never silently track a branch.
- **Immediate-mode layout mismatch:** keep the semantic layout tree authoritative
  and make any future absolute canvas an explicit node rather than the storage
  model.
- **Unmaintainable generated code:** preserve stable semantic names, small
  templates, syntax validation, formatting, and compile tests.
- **Scope expansion:** finish one Level 1 Citizen end to end before actions,
  async behavior, source ingestion, or multi-Citizen composition.
- **Compatibility drag:** reject unsupported schemas and formats clearly instead
  of carrying an unrelated editor model into the Citizen architecture.
