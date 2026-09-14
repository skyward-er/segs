use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io,
    path::PathBuf,
    sync::{Arc, mpsc},
    thread::{self, JoinHandle},
};

use chrono::Local;
use segs_mavlink::{MavFrame, MavProfile, MavType, MessageInfo, MsgField};

use super::truncate_c_string;
use crate::utils::get_data_dirpath;

const ADAPTER_DIRECTORY: &str = "adapter-skyward-mavlink";

/// One received MAVLink frame queued for CSV persistence.
pub(super) struct LogEntry {
    /// Adapter-relative receive time in seconds.
    timestamp: f64,
    /// Parsed frame received from the MAVLink connection.
    frame: MavFrame,
}

impl LogEntry {
    /// Creates a received-frame log entry.
    ///
    /// Returns an entry containing the adapter-relative receive `timestamp` in
    /// seconds and the parsed MAVLink `frame` owned by the logger.
    pub(super) fn new(timestamp: f64, frame: MavFrame) -> Self {
        Self { timestamp, frame }
    }
}

/// Owns the background writer used by one Skyward MAVLink adapter session.
pub(super) struct MessageLogger {
    /// Writer thread that drains queued frames for this adapter session.
    thread: JoinHandle<()>,
}

impl MessageLogger {
    /// Creates the session directory and starts its background CSV writer.
    ///
    /// Returns the sender used by the RX thread followed by the logger handle
    /// used to drain queued records. An error is returned when the application
    /// data directory, unique session directory, or writer thread cannot be
    /// created.
    pub(super) fn start(profile: Arc<MavProfile>) -> Result<(mpsc::Sender<LogEntry>, Self), Box<dyn Error>> {
        // Create the adapter and exclusive session directories before connecting
        let adapter_directory = get_data_dirpath().join(ADAPTER_DIRECTORY);
        fs::create_dir_all(&adapter_directory)?;
        let session_directory = adapter_directory.join(Local::now().format("%Y%m%d-%H%M%S").to_string());
        fs::create_dir(&session_directory)?;

        // Start the writer with exclusive ownership of its queue receiver
        let (sender, receiver) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("skyward-mavlink-csv".into())
            .spawn(move || run_logger(receiver, profile, session_directory))?;

        Ok((sender, Self { thread }))
    }

    /// Waits for the receive sender to close and all queued records to be attempted.
    pub(super) fn join(self) {
        if self.thread.join().is_err() {
            eprintln!("Skyward MAVLink CSV logger thread panicked");
        }
    }
}

/// Holds the open CSV files and protocol metadata for one logging session.
struct CsvLogger {
    /// Exclusive directory assigned to this adapter session.
    session_directory: PathBuf,
    /// Protocol metadata used to name messages and fields.
    profile: Arc<MavProfile>,
    /// Filesystem directory names keyed by MAVLink system ID.
    source_names: HashMap<u8, String>,
    /// Open CSV writers keyed by MAVLink system and message IDs.
    writers: HashMap<(u8, u32), csv::Writer<File>>,
}

impl CsvLogger {
    /// Creates a writer state for the supplied session and MAVLink profile.
    ///
    /// Returns an empty writer cache with source names resolved from the
    /// profile's `Sysids` enum.
    fn new(session_directory: PathBuf, profile: Arc<MavProfile>) -> Self {
        // Cache source names once for path lookup on every received frame
        let source_names = profile
            .enums
            .get("Sysids")
            .into_iter()
            .flat_map(|descriptor| &descriptor.entries)
            .filter_map(|entry| {
                let value = entry.value?;
                u8::try_from(value).ok().map(|value| (value, entry.name.clone()))
            })
            .collect();

        Self {
            session_directory,
            profile,
            source_names,
            writers: HashMap::new(),
        }
    }

    /// Writes and flushes one received frame.
    ///
    /// Returns `Ok(())` after the complete CSV row reaches the file. An error
    /// is returned for unknown messages, mismatched field counts, or filesystem
    /// and CSV failures.
    fn write(&mut self, entry: LogEntry) -> Result<(), Box<dyn Error>> {
        let system_id = entry.frame.header.system_id;
        let message_id = entry.frame.message.id;
        let key = (system_id, message_id);

        // Resolve an owned descriptor so the writer cache can be updated independently
        let message_info = self.profile.messages.get(&message_id).cloned().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Unknown MAVLink message ID {message_id}"),
            )
        })?;
        if message_info.fields.len() != entry.frame.message.fields.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "MAVLink message {} contains {} fields but its profile describes {}",
                    message_info.name,
                    entry.frame.message.fields.len(),
                    message_info.fields.len()
                ),
            )
            .into());
        }

        // Lazily open one file for each source and message pair
        if !self.writers.contains_key(&key) {
            let writer = self.create_writer(system_id, &message_info)?;
            self.writers.insert(key, writer);
        }
        let writer = self.writers.get_mut(&key).expect("CSV writer was inserted above");

        // Flatten the timestamp and message fields into one plain CSV record
        let mut record = Vec::with_capacity(message_info.fields.len() + 1);
        record.push(entry.timestamp.to_string());
        for field in entry.frame.message.fields {
            append_field_values(field, &mut record);
        }
        writer.write_record(record)?;
        writer.flush()?;

        Ok(())
    }

    /// Creates a CSV writer and writes its descriptor-derived header.
    ///
    /// Returns the writer for `message_info` under the source identified by
    /// `system_id`. An error is returned when its directory, file, or header
    /// cannot be written.
    fn create_writer(&self, system_id: u8, message_info: &MessageInfo) -> Result<csv::Writer<File>, Box<dyn Error>> {
        // Preserve described names and provide a stable fallback for unknown sources
        let source_name = self
            .source_names
            .get(&system_id)
            .cloned()
            .unwrap_or_else(|| format!("source-{system_id}"));
        let source_directory = self.session_directory.join(source_name);
        fs::create_dir_all(&source_directory)?;

        // Build a header matching the flattened representation of each field
        let mut header = Vec::with_capacity(message_info.fields.len() + 1);
        header.push("timestamp".to_owned());
        for field in &message_info.fields {
            append_field_headers(&field.name, &field.mavtype, &mut header);
        }

        // Create the message file once and emit the explicit dynamic header
        let path = source_directory.join(format!("{}.csv", message_info.name));
        let mut writer = csv::WriterBuilder::new().has_headers(false).from_path(path)?;
        writer.write_record(header)?;
        writer.flush()?;

        Ok(writer)
    }
}

/// Runs the logging loop until every sender has been dropped.
fn run_logger(receiver: mpsc::Receiver<LogEntry>, profile: Arc<MavProfile>, session_directory: PathBuf) {
    let mut logger = CsvLogger::new(session_directory, profile);

    // Attempt every entry once so one failed write cannot stop queue consumption
    while let Ok(entry) = receiver.recv() {
        if let Err(error) = logger.write(entry) {
            eprintln!("Failed to write Skyward MAVLink CSV entry, skipping: {error}");
        }
    }
}

/// Adds headers for one MAVLink field, expanding arrays into indexed columns.
fn append_field_headers(name: &str, field_type: &MavType, header: &mut Vec<String>) {
    match field_type {
        MavType::Array(element_type, length) => {
            for index in 0..*length {
                append_field_headers(&format!("{name}[{index}]"), element_type, header);
            }
        }
        _ => header.push(name.to_owned()),
    }
}

/// Adds plain CSV values for one MAVLink field, flattening arrays recursively.
fn append_field_values(field: MsgField, record: &mut Vec<String>) {
    match field {
        MsgField::UInt8(value) => record.push(value.to_string()),
        MsgField::UInt16(value) => record.push(value.to_string()),
        MsgField::UInt32(value) => record.push(value.to_string()),
        MsgField::UInt64(value) => record.push(value.to_string()),
        MsgField::Int8(value) => record.push(value.to_string()),
        MsgField::Int16(value) => record.push(value.to_string()),
        MsgField::Int32(value) => record.push(value.to_string()),
        MsgField::Int64(value) => record.push(value.to_string()),
        MsgField::Char(value) => record.push(value.to_string()),
        MsgField::Float(value) => record.push(value.to_string()),
        MsgField::Double(value) => record.push(value.to_string()),
        MsgField::CharArray(value) => record.push(truncate_c_string(value)),
        MsgField::Array(values) => {
            for value in values {
                append_field_values(value, record);
            }
        }
    }
}
