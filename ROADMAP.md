# citizen-builder roadmap

- Status: milestones 1–4 implemented
- Builder version: 0.4.0
- Project schema: 4 (current schema only)
- Generator contract: 4
- Framework baseline: egui_mobius 0.5, egui 0.35, egui_dock 0.20
- Pinned framework revision: `cc286df73b6cb3a015b3d0c5159aeaf3f41510cc`

## Product contract

- A project describes exactly one reusable Citizen.
- Its primary output is a standalone Citizen library crate.
- Every output includes a native/WASM preview host and explicit host
  integration guidance.
- Level 1 is typed `Dynamic<T>` state and Dispatcher lifecycle integration.
- Level 2 adds domain-grouped intent/outcome messages and host routing.
- Level 3 adds cancellable signal/slot and asynchronous backends.
- Ecosystem features are inferred from semantic nodes; target restrictions are
  expressed in the generated manifest and code.
- Multi-Citizen composition belongs to the compile-time host. It never changes
  the one-project/one-library-Citizen contract.
- Git dependencies are pinned to an exact revision. A project may instead use
  an explicit local egui_mobius workspace path.
- `citizen.json` is the only editable source format. Only the current versioned
  schema is supported; arbitrary Rust round-tripping and legacy editor formats
  are intentionally unsupported.
- The builder dogfoods Citizen, Dispatcher, reactive state, dock, native, and
  WASM architecture.

## Development loop

```text
template or current-schema JSON
    -> Citizen metadata + typed state + messages + semantic layout
    -> structural, binding, interaction, asset, and composition validation
    -> live themed Citizen and host-dock preview
    -> deterministic standalone crate
    -> native and WASM compile/test gates
    -> single- or multi-Citizen host integration
```

Citizens remain ordinary compile-time Rust dependencies. Runtime dynamic
plug-ins are not a target because that would require an unstable cross-crate
Rust ABI.

## Milestone 1 — hardened Level 1 editor (complete)

Implemented:

- Strict versioned schema, generator/backend metadata, and unknown-field
  rejection.
- One Citizen identity and one semantic layout tree with stable IDs and names.
- Typed bool/text/number state, preview-only fixtures, binding validation, and
  rename/delete propagation.
- Tree drag/reparent/reorder, indent/outdent, keyboard navigation, undo/redo,
  and accessible palette insertion.
- Dogfooded Outline, Canvas, Inspector, and Generated Files Citizens in an
  `egui_dock` shell.
- Native/WASM preview generation, safe new-directory export, exact Git or local
  framework sources, deterministic file snapshots, and representative-node
  goldens.

Acceptance record:

- Nested Settings designs survive current-schema save/load round trips.
- Invalid identities, trees, sources, and bindings block generation with
  actionable diagnostics.
- Default and representative generated crates are syntax- and rustfmt-clean and
  compile on native and WASM targets.

## Milestone 2 — application messages (complete)

Implemented:

- Click, change, and submit interaction editing.
- Separate domain-grouped intent and outcome definitions.
- Validation preventing intents from mutating reactive state and enforcing
  typed UI-thread outcome updates.
- Generated `AppMessage` enums, Citizen outboxes, synchronous reference
  backend, host drain/routing, and outcome application.
- Visible preview message path with built-in logging and optional `egui_lens`.

Acceptance record:

- Generated tests request work, produce the paired outcome, and verify the
  resulting `Dynamic<T>` state change.

## Milestone 3 — cancellable async behavior (complete)

Implemented:

- Opt-in intent/outcome async mappings with reference delays.
- Native `egui_mobius` `Signal`/`Slot` plus `AsyncDispatcher` work queues.
- Abortable WASM local futures with target-gated dependencies.
- Cooperative native cancellation, browser abort handles, generation-based
  stale-result rejection, and lifecycle cancellation on Citizen deactivation.
- UI-thread-only result draining and state mutation.

Acceptance record:

- Generated native tests cover both completion and cancellation.
- Async all-feature generated crates compile for native and
  `wasm32-unknown-unknown`.

## Milestone 4 — ecosystem acceleration (complete)

Implemented:

- Generation-valid Settings, Logger, Editor, Plot, File Browser, and Backend
  Control templates.
- Curated semantic wrappers for `egui_lens`, `egui_quill`, `egui_plot`, and
  `egui_mobius_widgets`.
- Inferred component and async Cargo features. The widgets wrapper is native
  target-gated because its current dependency graph includes Tokio networking;
  WASM receives a functional standard-egui interaction fallback.
- Dark/light themes, accent and panel colors, item spacing, UTF-8 text/SVG
  assets, generated `include_str!` constants, and asset validation.
- Richer constraints for depth, layout child counts, scroll semantics, grids,
  editor language, assets, themes, and dock metadata.
- Visual external-Citizen dock authoring with concrete crate/type/ID metadata,
  tabs and directional splits, generated preview placeholders, and complete
  host fields/registration/render/activation/dock scaffolding.
- Current-schema JSON open/save/import as the supported source-ingestion path.

Acceptance record:

- All six templates validate, round-trip, generate deterministic crates, and
  emit rustfmt-clean Rust.
- Every template passes strict native all-feature compilation.
- A kitchen-sink crate combining every curated component, assets, async work,
  and host composition passes native clippy/tests and all-feature WASM check.
- The generated library remains a single reusable Citizen regardless of host
  composition settings.

## Continuing priorities

The planned product contract is complete. Future work should remain compatible
with it and be driven by real Citizen authoring:

- Add GUI automation where it gives more confidence than model/generator tests.
- Track stable egui_mobius component APIs and remove target fallbacks when their
  upstream dependency graphs support WASM directly.
- Add optional browser archive download without weakening non-overwriting
  native export.
- Grow semantic nodes and templates only when they preserve deterministic,
  readable, host-portable output.
- Consider migrations only between citizen-builder schemas when a concrete
  compatibility need appears; do not import unrelated RAD project formats.

## Quality gates

Every change retains:

```shell
cargo fmt --all -- --check
cargo check --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo build --release --all-features
cargo check --target wasm32-unknown-unknown --all-features
```

Generator changes additionally materialize default, async, template, and
kitchen-sink fixtures and verify:

- deterministic file sets and focused golden snapshots;
- valid, formatter-clean Rust;
- strict native library and preview compilation;
- generated synchronous and asynchronous tests;
- all-feature WASM preview compilation;
- exact dependency-source and target-gate emission; and
- rejection of invalid projects before filesystem writes.

## Risks and controls

- **Framework API churn:** isolate framework-specific emission, pin exact
  revisions, and update only against compiling fixtures.
- **Workspace-only Citizen crates:** keep Git versus local workspace source an
  explicit saved setting; never silently follow a branch.
- **Immediate-mode layout mismatch:** keep the semantic tree authoritative; do
  not store transient pixels as the design model.
- **Unmaintainable generated code:** preserve semantic names, small modules,
  syntax validation, rustfmt compatibility, goldens, and compile tests.
- **Host coupling:** represent neighboring Citizens as host metadata and
  scaffolding; never link them into the exported one-Citizen library.
- **Compatibility drag:** reject unsupported schemas and formats clearly rather
  than carrying an unrelated editor model into the Citizen architecture.
