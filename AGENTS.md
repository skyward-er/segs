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
4. Run the narrowest useful checks first, such as `cargo check -p <crate>` and `cargo test -p <crate> <test-name>`. For changes spanning crates or shared interfaces, follow with `cargo check` and the relevant workspace tests.
5. Run `cargo clippy` and fix any warnings or errors it reports.
6. Report which checks ran and any checks that could not run.

Use `cargo run` to launch the application when manual UI verification is relevant.

# Tests

- *Never* write unit tests unless explicitly requested
- Do not write unit tests for trivial getters, constructors, field wiring, constant mappings, or code that is easily checked once and is unlikely to become incorrect later
- Do not introduce or expose a function solely to make code unit-testable. Test through the natural API or keep the behavior inline
- If tests were requested, prefer focused tests colocated in the owning module. Name tests after the behavior and expected outcome
- Rely on compilation, formatting, linting, or manual QA verification to test the code

# Rust documentation

- Always write rustdoc for functions, structs, enums, and their fields or variants
- Always document tuple return values
- For every function that requires rustdoc, document its return type and what the returned value means. For `Result` and `Option`, explain meaningful success values and the conditions that produce errors or `None`
- Document side effects, ownership or resource-release rules, units, wire-format assumptions, and panics when they matter to correct use
- Keep documentation focused on the contract and rationale instead of restating the implementation
- Trivial private items do not need rustdoc merely for coverage

# Code and comment style

- Always divide the implementation into logical blocks of code and comment each one describing the high level operation that block is performing
- Always question whether multiple iterations over the same data structure are necessary or if they can be coalesced into a single iteration
- Always question whether a new data structure (e.g. a `Vec` or `HashMap`) is necessary or if an existing one can be extended to include the new data needed for the feature you're adding
- Keep code comments to one simple sentence in general
- Code comments must not end with a period
- Prefer one comment for a cohesive block. Use two adjacent comments only when they communicate distinct constraints that are both necessary
- Inline comments are appropriate for short exceptional control-flow explanations, such as why a loop continues or returns

These comment rules apply to ordinary code comments (`//`). Rustdoc should use normal prose and punctuation.

The overall goal of code comments is to make the code of a function be able to be read immediately by a human reader, without needing to read the code itself. See the example below.

Example of the code commenting style to follow:

```rust
/// Process outgoing data from the data store.
fn process_outgoing(&mut self, data_store: &mut DataStore) {
    // Process any send failures from the RX thread
    while let Ok(failure) = self.send_failures.try_recv() {
        if self.pending.get(failure.pending_slot) != Some(failure.command_id) {
            continue; // No matching command in the pending slot, skip
        }
        // Update the command status
        data_store.command_sequence_mut(failure.command_id).status = CommandStatus::LocalError;
        self.pending.release(failure.pending_slot);
    }

    self.expire_pending_commands(data_store, Instant::now());

    loop {
        let mut pending_slot = 0;

        // Try to acquire a pending slot for the command
        let Some(command_sequence) =
            data_store.next_outgoing_command_if(|command| match self.pending.acquire(command.id) {
                Some(slot) => {
                    pending_slot = slot;
                    true
                }
                None => false,
            })
        else {
            break; // No more commands to process
        };

        let command = &command_sequence.request;
        let command_id = command_sequence.id;

        // Retrieve the message info for the command
        let id = command.key.0 as u32;
        let Some(message_info) = self.profile.messages.get(&id) else {
            self.pending.release(pending_slot);
            command_sequence.status = CommandStatus::LocalError;
            eprintln!("Missing serialization info for message ID {id}, skipping");
            continue;
        };

        // Construct the MAVLink message
        let message = match command_to_mav_message(command, message_info) {
            Ok(message) => message,
            Err(err) => {
                self.pending.release(pending_slot);
                command_sequence.status = CommandStatus::LocalError;
                eprintln!("Failed to construct MAVLink message ID {id}: {err}");
                continue;
            }
        };

        // Use the pending slot ID for correlating responses with a specific request
        // The component ID in the MAVLink header is used for this purpose in the Skyward dialect
        let header = MavHeader {
            system_id: command.target.0 as u8,
            component_id: pending_slot,
            sequence: self.packet_sequence.0,
        };
        self.packet_sequence += 1;

        let outgoing = OutgoingCommand {
            frame: MavFrame {
                version: MavlinkVersion::V1,
                header,
                message,
            },
            pending_slot,
            command_id,
        };

        // Send the command to the TX thread
        if self.outgoing.send(outgoing).is_err() {
            self.pending.release(pending_slot);
            command_sequence.status = CommandStatus::LocalError;
        }
    }

    self.schedule_pending_timeout();
}
```

