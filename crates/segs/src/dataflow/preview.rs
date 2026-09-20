use std::time::{Duration, Instant};

use super::{DataKey, DataPoint, DataStream, SourceKey, StreamKey, store::DataStore};

const NUMERIC_UPDATE_HZ: f64 = 5.;
const INTEGER_UPDATE_HZ: f64 = 0.2;
const HISTORY_SECONDS: f64 = 60.;
const SIGNAL_FREQUENCY_HZ: f64 = 0.05;
const SIGNAL_PHASE_RADIANS: f64 = 0.37;
const SIGNAL_CENTER: f64 = 42.;
const SIGNAL_AMPLITUDE: f64 = 12.;

/// Synthetic data owned exclusively by gallery rendering, never the live store.
pub struct PreviewContext {
    /// Isolated samples read by preview widgets.
    data_store: DataStore,
    /// Monotonic origin used to schedule preview samples.
    started_at: Instant,
    /// Latest generated floating-point sample index.
    numeric_index: u64,
    /// Latest generated integer sample index.
    integer_index: u64,
    /// Floating-point sample stream for ordinary numeric widgets.
    pub numeric_stream: StreamKey,
    /// Integer sample stream for state-oriented widgets.
    pub integer_stream: StreamKey,
    /// Delay until the next sample is due.
    pub repaint_after: Duration,
}

impl PreviewContext {
    /// Builds isolated sample history and keys for one gallery editing session.
    /// Numeric keys are local to this store and are not reserved in live telemetry.
    ///
    /// Returns a context containing a full floating-point history and the initial
    /// integer state.
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
        let numeric_index = (HISTORY_SECONDS * NUMERIC_UPDATE_HZ).ceil() as u64;

        // Build the floating-point history used by numeric gallery widgets
        let points = (0..=numeric_index).map(numeric_point).collect();
        data_store.streams.insert(numeric_stream, DataStream::F64(points));

        // Build the integer sample used by state-oriented gallery widgets
        data_store.streams.insert(
            integer_stream_key,
            DataStream::I64(vec![DataPoint {
                timestamp: HISTORY_SECONDS,
                value: 0,
            }]),
        );

        Self {
            data_store,
            started_at: Instant::now(),
            numeric_index,
            integer_index: 0,
            numeric_stream,
            integer_stream: integer_stream_key,
            repaint_after: numeric_interval(),
        }
    }

    /// Adds any samples due since the previous gallery frame without rebuilding the streams.
    ///
    /// Returns the delay until the next floating-point or integer sample is due.
    pub fn update(&mut self) -> Duration {
        let elapsed = self.started_at.elapsed().as_secs_f64();
        let initial_numeric_index = (HISTORY_SECONDS * NUMERIC_UPDATE_HZ).ceil() as u64;
        let target_numeric_index = initial_numeric_index + (elapsed * NUMERIC_UPDATE_HZ).floor() as u64;
        let target_integer_index = (elapsed * INTEGER_UPDATE_HZ).floor() as u64;

        // Extend the rolling numeric history only when one or more samples are due
        if target_numeric_index > self.numeric_index {
            let Some(DataStream::F64(points)) = self.data_store.streams.get_mut(&self.numeric_stream) else {
                unreachable!("preview numeric stream must remain floating-point");
            };
            let added = (target_numeric_index - self.numeric_index) as usize;
            let retained_samples = initial_numeric_index as usize + 1;
            if added >= retained_samples {
                points.clear();
                points.extend((target_numeric_index - initial_numeric_index..=target_numeric_index).map(numeric_point));
            } else {
                points.drain(..added);
                points.extend((self.numeric_index + 1..=target_numeric_index).map(numeric_point));
            }
            self.numeric_index = target_numeric_index;
        }

        // Replace the single integer state only when its slower sample is due
        if target_integer_index > self.integer_index {
            let Some(DataStream::I64(points)) = self.data_store.streams.get_mut(&self.integer_stream) else {
                unreachable!("preview integer stream must remain integer");
            };
            points[0] = DataPoint {
                timestamp: HISTORY_SECONDS + target_integer_index as f64 / INTEGER_UPDATE_HZ,
                value: (target_integer_index % 3) as i64,
            };
            self.integer_index = target_integer_index;
        }

        // Schedule the next repaint at the earliest stream deadline
        self.repaint_after = delay_until_next_sample(elapsed, NUMERIC_UPDATE_HZ)
            .min(delay_until_next_sample(elapsed, INTEGER_UPDATE_HZ));
        self.repaint_after
    }

    /// Returns the isolated store for rendering previews, including mutable widget APIs.
    pub fn data_store(&mut self) -> &mut DataStore {
        &mut self.data_store
    }
}

impl Default for PreviewContext {
    /// Builds the default gallery preview context.
    fn default() -> Self {
        Self::new()
    }
}

fn numeric_point(index: u64) -> DataPoint<f64> {
    let timestamp = index as f64 / NUMERIC_UPDATE_HZ;
    let phase = std::f64::consts::TAU * SIGNAL_FREQUENCY_HZ * timestamp + SIGNAL_PHASE_RADIANS;
    DataPoint {
        timestamp,
        value: SIGNAL_CENTER + SIGNAL_AMPLITUDE * phase.sin(),
    }
}

fn numeric_interval() -> Duration {
    Duration::from_secs_f64(1. / NUMERIC_UPDATE_HZ)
}

fn delay_until_next_sample(elapsed: f64, update_hz: f64) -> Duration {
    let completed_samples = (elapsed * update_hz).floor();
    let next_sample = (completed_samples + 1.) / update_hz;
    Duration::from_secs_f64((next_sample - elapsed).max(f64::EPSILON))
}
