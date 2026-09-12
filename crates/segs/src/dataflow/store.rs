use ahash::HashMap;

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

    /// Ensures the live sample stream used by widget gallery previews is current.
    ///
    /// The returned duration is the time until the next mock sample should be
    /// generated and can be used to schedule a repaint.
    pub fn ensure_mock_stream(&mut self) -> std::time::Duration {
        const UPDATE_HZ: f64 = 10.;
        const HISTORY_SECONDS: f64 = 60.;
        const SIGNAL_FREQUENCY_HZ: f64 = 0.05;
        const SIGNAL_PHASE_RADIANS: f64 = 0.37;
        const SIGNAL_CENTER: f64 = 42.;
        const SIGNAL_AMPLITUDE: f64 = 12.;

        fn signal_value(timestamp: f64) -> f64 {
            let phase = std::f64::consts::TAU * SIGNAL_FREQUENCY_HZ * timestamp + SIGNAL_PHASE_RADIANS;
            SIGNAL_CENTER + SIGNAL_AMPLITUDE * phase.sin()
        }

        let update_interval = 1. / UPDATE_HZ;
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Determine the fixed-rate samples in the current history window
        let newest_index = (current_timestamp * UPDATE_HZ).floor() as u64;
        let history_samples = (HISTORY_SECONDS * UPDATE_HZ).ceil() as u64;
        let first_index = newest_index.saturating_sub(history_samples);

        // Rebuild the shared stream so all gallery widgets observe the same latest value
        let stream = self
            .streams
            .entry(StreamKey::mock())
            .or_insert_with(|| DataStream::F64(Vec::new()));
        if let DataStream::F64(points) = stream {
            points.clear();
            points.reserve(history_samples.saturating_add(1) as usize);
            for index in first_index..=newest_index {
                let timestamp = index as f64 * update_interval;
                points.push(DataPoint {
                    timestamp,
                    value: signal_value(timestamp),
                });
            }
        }

        // Wake the gallery when the next fixed-rate sample becomes due
        let next_timestamp = (newest_index + 1) as f64 * update_interval;
        std::time::Duration::from_secs_f64((next_timestamp - current_timestamp).max(f64::EPSILON))
    }

    /// Returns the complete stream associated with `key`.
    pub fn stream(&self, key: StreamKey) -> Option<&DataStream> {
        self.streams.get(&key)
    }

    /// Returns the most recent sample in the stream associated with `key`.
    ///
    /// The returned tuple contains the adapter-relative timestamp in seconds
    /// followed by the sample value. `None` means the stream is absent or has
    /// no samples.
    pub fn latest(&self, key: StreamKey) -> Option<(f64, DataValue)> {
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
