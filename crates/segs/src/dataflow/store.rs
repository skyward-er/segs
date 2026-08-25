use egui::ahash::HashMap;

use crate::dataflow::{
    Command, CommandId, CommandSequence, CommandStatus, DataPoint, DataStream, DataValue, StreamKey,
};

/// Central data store that holds all processed data streams, raw messages, and command sequences.
///
/// Data adapters will update this store with new data points as they are processed.
/// UI will read from this store to display information to the user.
#[derive(Default)]
pub struct DataStore {
    /// Processed data streams.
    pub(super) streams: HashMap<StreamKey, DataStream>,
    /// Commands sequences, either pending response or completed.
    pub(super) commands: Vec<CommandSequence>,
    /// Next stable command identifier.
    next_command_id: u64,
    /// The index of the first sequence to be sent in the commands vector.
    next_outgoing_index: u64,
}

impl DataStore {
    pub fn new() -> Self {
        Default::default()
    }

    /// Ensures the fixed sample stream used by widget gallery previews exists.
    pub fn ensure_mock_stream(&mut self) {
        self.streams.entry(StreamKey::mock()).or_insert_with(|| {
            DataStream::F64(vec![DataPoint {
                timestamp: 0.,
                value: 42.,
            }])
        });
    }

    /// Returns the complete stream associated with `key`.
    pub fn stream(&self, key: StreamKey) -> Option<&DataStream> {
        self.streams.get(&key)
    }

    /// Returns the most recent value in the stream associated with `key`.
    pub fn latest(&self, key: StreamKey) -> Option<DataValue> {
        self.stream(key).and_then(DataStream::last)
    }

    /// Returns the command sequence with the given datastore-issued identifier.
    pub fn command_sequence(&self, id: CommandId) -> &CommandSequence {
        let sequence = &self.commands[id.0 as usize];
        debug_assert_eq!(sequence.id, id);
        sequence
    }

    /// Returns a mutable reference to the command sequence with the given datastore-issued identifier.
    pub(super) fn command_sequence_mut(&mut self, id: CommandId) -> &mut CommandSequence {
        let sequence = &mut self.commands[id.0 as usize];
        debug_assert_eq!(sequence.id, id);
        sequence
    }

    /// Enqueues the given command for transmission.
    pub fn enqueue_command(&mut self, command: Command) -> CommandId {
        let id = CommandId(self.next_command_id);

        self.commands.push(CommandSequence {
            id,
            status: CommandStatus::Pending,
            request: command,
            responses: Vec::new(),
        });
        self.next_command_id += 1;

        id
    }

    /// Returns the next outgoing command to be sent.
    pub(super) fn next_outgoing_command_if(
        &mut self,
        predicate: impl FnOnce(&CommandSequence) -> bool,
    ) -> Option<&mut CommandSequence> {
        if self.next_outgoing_index >= self.commands.len() as u64 {
            return None;
        }

        let sequence = &mut self.commands[self.next_outgoing_index as usize];
        if !predicate(sequence) {
            return None;
        }

        self.next_outgoing_index += 1;
        Some(sequence)
    }
}
