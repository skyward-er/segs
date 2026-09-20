use std::time::Duration;

use super::{DataKey, DataPoint, DataStream, SourceKey, StreamKey, store::DataStore};

/// Synthetic data owned exclusively by gallery rendering, never the live store.
pub struct PreviewContext {
    /// Isolated samples read by preview widgets.
    data_store: DataStore,
    /// Floating-point sample stream for ordinary numeric widgets.
    pub numeric_stream: StreamKey,
    /// Integer sample stream for state-oriented widgets.
    pub integer_stream: StreamKey,
    /// Delay until the next sample is due.
    pub repaint_after: Duration,
}

impl PreviewContext {
    /// Builds isolated sample history and keys for one gallery render.
    /// Numeric keys are local to this store and are not reserved in live telemetry.
    pub fn new() -> Self {
        // Keep sample identities confined to a store that never receives adapter data
        let mut data_store = DataStore::new();
        let numeric_stream = StreamKey {
            source_key: SourceKey(0),
            data_key: DataKey(0),
        };
        let integer_stream_key = StreamKey {
            source_key: SourceKey(0),
            data_key: DataKey(1),
        };
        const UPDATE_HZ: f64 = 10.;
        const HISTORY_SECONDS: f64 = 60.;
        const SIGNAL_FREQUENCY_HZ: f64 = 0.05;
        const SIGNAL_PHASE_RADIANS: f64 = 0.37;
        const SIGNAL_CENTER: f64 = 42.;
        const SIGNAL_AMPLITUDE: f64 = 12.;

        let update_interval = 1. / UPDATE_HZ;
        let current_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Determine the fixed-rate samples in the current history window
        let newest_index = (current_timestamp * UPDATE_HZ).floor() as u64;
        let history_samples = (HISTORY_SECONDS * UPDATE_HZ).ceil() as u64;
        let first_index = newest_index.saturating_sub(history_samples);

        // Build the floating-point history used by numeric gallery widgets
        let points = (first_index..=newest_index)
            .map(|index| {
                let timestamp = index as f64 * update_interval;
                let phase = std::f64::consts::TAU * SIGNAL_FREQUENCY_HZ * timestamp + SIGNAL_PHASE_RADIANS;
                DataPoint {
                    timestamp,
                    value: SIGNAL_CENTER + SIGNAL_AMPLITUDE * phase.sin(),
                }
            })
            .collect();
        data_store.streams.insert(numeric_stream, DataStream::F64(points));

        // Build the integer sample used by state-oriented gallery widgets
        data_store.streams.insert(
            integer_stream_key,
            DataStream::I64(vec![DataPoint {
                timestamp: newest_index as f64 * update_interval,
                value: (newest_index % 3) as i64,
            }]),
        );

        // Wake the gallery when the next fixed-rate sample becomes due
        let next_timestamp = (newest_index + 1) as f64 * update_interval;
        let repaint_after = Duration::from_secs_f64((next_timestamp - current_timestamp).max(f64::EPSILON));
        Self {
            data_store,
            numeric_stream,
            integer_stream: integer_stream_key,
            repaint_after,
        }
    }

    /// Returns the isolated store for rendering previews, including mutable widget APIs.
    pub fn data_store(&mut self) -> &mut DataStore {
        &mut self.data_store
    }
}
