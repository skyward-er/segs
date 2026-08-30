# Repository guide

SEGS (Skyward Enhanced Ground Software) is a Rust 2024 workspace for a configurable ground-station desktop application. The workspace currently targets Rust 1.95 and contains these crates:

- `crates/segs`: application binary and core dataflow logic
- `crates/segs-mavlink`: MAVLink parsing, profiles, and connections
- `crates/segs-ui`: reusable UI and layout components
- `crates/segs-plot`: plotting components
- `crates/segs-memory`: persistence and storage
- `crates/segs-assets`: embedded and UI assets

Keep changes within the crate that owns the behavior. Use workspace dependencies when a dependency is shared, and preserve existing public interfaces unless the request requires changing them.

Within the `segs` crate, do not use `pub(crate)`. Use other visibility modifiers (`pub`, `pub(super)`) when they are appropriate.

# Code placement

When adding code to an existing module, preserve its purpose-first ordering. Keep the items that define and implement the module's primary responsibility at the top, and do not insert helpers or secondary types above them. Place inherent implementations after the primary trait implementations, then supporting functions and private types, with tests last. For example, the Skyward MAVLink adapter starts with `SkywardMavlinkAdapter` and its `DataAdapter` implementation because that is the module's main purpose, followed by the inherent `SkywardMavlinkAdapter` implementation, helper functions, supporting types, and tests. Match the existing module's equivalent hierarchy rather than applying this exact item order mechanically.

Before adding substantial code to an existing module, evaluate whether it represents a separate responsibility that belongs in its own module. Prefer a separate module when it creates a clear boundary, improves readability, or prevents the existing module's primary flow from being obscured. Keep code together when splitting it would only add indirection or separate tightly coupled behavior.

# Workflow for every request

1. Read the relevant implementation and nearby tests before editing. Check the working tree and preserve unrelated user changes.
2. Make the smallest coherent change that satisfies the request. Do not add abstractions, helpers, dependencies, or public API without a concrete need.
3. Format Rust changes with `cargo fmt`.
4. Run the narrowest useful checks first, such as `cargo check -p <crate>` and `cargo test -p <crate> <test-name>`. For changes spanning crates or shared interfaces, follow with `cargo check --workspace` and the relevant workspace tests.
5. Report which checks ran and any checks that could not run.

Use `cargo run --bin segs` to launch the application when manual UI verification is relevant.

# Tests

- Write unit tests only for non-trivial behavior: branching logic, state transitions, conversions, validation, error handling, invariants, or regressions that could plausibly recur
- Do not write unit tests for trivial getters, constructors, field wiring, constant mappings, or code that is easily checked once and is unlikely to become incorrect later
- Do not introduce or expose a function solely to make code unit-testable. Test through the natural API or keep the behavior inline
- Prefer focused tests colocated in the owning module. Name tests after the behavior and expected outcome
- When a change affects only trivial code, rely on compilation, formatting, linting, or a concise manual verification instead of adding a low-value test

# Rust documentation

- Write rustdoc for functions, structs, enums, and their fields or variants when their purpose is not obvious from the name or when their use, invariants, lifecycle, protocol role, or constraints would otherwise be hard to discover
- For every function that requires rustdoc, document its return type and what the returned value means. For `Result` and `Option`, explain meaningful success values and the conditions that produce errors or `None`
- Document side effects, ownership or resource-release rules, units, wire-format assumptions, and panics when they matter to correct use
- Keep documentation focused on the contract and rationale instead of restating the implementation
- Trivial private items do not need rustdoc merely for coverage

# Code and comment style

- Separate logical phases with an empty line, such as lookup, validation, conversion, state update, and dispatch
- Add a short comment before a block when its high-level purpose is not immediately clear
- Explain why a block exists or how it fits the protocol or state flow; do not narrate obvious individual statements
- Keep code comments to one simple sentence in general
- Code comments must not end with a period
- Prefer one comment for a cohesive block. Use two adjacent comments only when they communicate distinct constraints that are both necessary
- Inline comments are appropriate for short exceptional control-flow explanations, such as why a loop continues or returns

These comment rules apply to ordinary code comments (`//`). Rustdoc should use normal prose and punctuation.
