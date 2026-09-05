use std::collections::{
    VecDeque,
    hash_map::{DefaultHasher, Entry},
};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind::{Interrupted, TimedOut, WouldBlock};
use std::iter::zip;
use std::num::Wrapping;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc, mpsc::Receiver, mpsc::Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use segs_mavlink::connection::{Connection, MavConnection};
use segs_mavlink::{MavFrame, MavHeader, MavMessage, MavProfile, MavType, MavlinkVersion, MessageReadError, MsgField};

use crate::dataflow::adapter::{DataAdapter, Stats, Status};
use crate::dataflow::mapping::{DataMapping, MappingDescriptor, MappingType};
use crate::dataflow::protocol::{
    EnumDescriptor, FieldDescriptor, MessageDescriptor, ProtocolDescriptor, SourceDescriptor,
};
use crate::dataflow::transport::DataTransport;
use crate::dataflow::{
    Command, DataKey, DataPoint, DataStream, DataType, DataValue, SourceKey, StreamKey, store::DataStore,
};
use crate::dataflow::{CommandId, CommandSequence, CommandStatus, MessageKey};

/// Component ID used for telemetry messages. All other values are used for command request-response correlation.
const TELEMETRY_COMPONENT_ID: u8 = 0;
const TC_MESSAGE_SUFFIX: &str = "_TC";
const TC_TIMESTAMP_FIELD: &str = "timestamp";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(10);
const STATS_RATE_WINDOW: Duration = Duration::from_secs(4);

/// Skyward-specific adapter implementation for the MAVLink protocol.
/// Uses a local XML file mapping source that defines the MAVLink message formats to be processed
pub struct SkywardMavlinkAdapter {
    transport: DataTransport,
    mapping: DataMapping,
    ctx: egui::Context,
    stop_flag: Arc<AtomicBool>,
    incoming: Receiver<MavFrame>,
    outgoing: Sender<OutgoingCommand>,
    send_failures: Receiver<SendFailure>,
    rx_stats: Arc<Mutex<IoStats>>,
    tx_stats: Arc<Mutex<IoStats>>,
    /// Outgoing packet sequence number.
    packet_sequence: Wrapping<u8>,
    /// Maps custom MAVLink component IDs to stable datastore command IDs.
    pending: PendingCommandSlots,
    profile: Arc<MavProfile>,
    /// The MAVLink message ID used for ACK responses, cached from the MAVLink XML profile.
    ack_message_id: u32,
    /// The MAVLink message ID used for WACK responses, cached from the MAVLink XML profile.
    wack_message_id: u32,
    /// The MAVLink message ID used for NACK responses, cached from the MAVLink XML profile.
    nack_message_id: u32,
    /// Cached protocol descriptor
    /// TODO: move caching logic to [DataAdapterInstance]
    protocol: ProtocolDescriptor,
    created_at: Instant,
}

impl DataAdapter for SkywardMavlinkAdapter {
    fn get_mapping_sources() -> Vec<MappingDescriptor>
    where
        Self: Sized,
    {
        vec![MappingDescriptor {
            method: MappingType::LocalFile,
            description: "MAVLink XML message definition file".into(),
        }]
    }

    fn new(ctx: egui::Context, transport: DataTransport, mapping: DataMapping) -> Result<Self, Box<dyn Error>> {
        let profile = match &mapping {
            DataMapping::LocalFile(path) => {
                let mav_profile = segs_mavlink::parse_profile(path)?;
                Arc::new(segs_mavlink::MavProfile::from_profile_info(&mav_profile))
            }
            _ => return Err("Unsupported definition source method".into()),
        };

        // Cache message IDs for ACK/WACK/NACK messages for faster lookup
        let Some(ack_message_id) = profile.messages.values().find(|m| m.name == "ACK_TM").map(|m| m.id) else {
            return Err("Unsupported MAVLink profile: ACK_TM not found".into());
        };
        let Some(wack_message_id) = profile.messages.values().find(|m| m.name == "WACK_TM").map(|m| m.id) else {
            return Err("Unsupported MAVLink profile: WACK_TM not found".into());
        };
        let Some(nack_message_id) = profile.messages.values().find(|m| m.name == "NACK_TM").map(|m| m.id) else {
            return Err("Unsupported MAVLink profile: NACK_TM not found".into());
        };

        let (incoming_tx, incoming_rx) = mpsc::channel::<MavFrame>();
        let (outgoing_tx, outgoing_rx) = mpsc::channel::<OutgoingCommand>();
        let (send_failure_tx, send_failure_rx) = mpsc::channel::<SendFailure>();

        let stop_flag = Arc::new(AtomicBool::new(false));

        let connection = match &transport {
            DataTransport::Ethernet {
                recv_socket,
                send_socket,
            } => Connection::udp(recv_socket, *send_socket, profile.clone())?,
            DataTransport::Serial { tty, baud_rate } => Connection::serial(tty.clone(), *baud_rate, profile.clone())?,
        };
        let connection = Arc::new(connection);
        let created_at = Instant::now();
        let rx_stats = Arc::new(Mutex::new(IoStats::new(created_at)));
        let tx_stats = Arc::new(Mutex::new(IoStats::new(created_at)));

        let rx_conn = connection.clone();
        let rx_stop_flag = stop_flag.clone();
        let rx_ctx = ctx.clone();
        let rx_thread_stats = rx_stats.clone();
        // RX thread: receives incoming MAVLink frames and notifies the UI to update
        thread::spawn(move || {
            while !rx_stop_flag.load(Ordering::Relaxed) {
                // Receive the frame
                match rx_conn.recv_frame() {
                    Ok(frame) => {
                        rx_thread_stats.lock().unwrap().record_success(Instant::now());

                        // Send the frame to the incoming channel
                        let Ok(_) = incoming_tx.send(frame) else {
                            break; // Receiver has been dropped, exit the thread
                        };

                        // Notify the UI to update with new data
                        rx_ctx.request_repaint_of(egui::ViewportId::ROOT)
                    }
                    Err(MessageReadError::Io(e)) => match e.kind() {
                        WouldBlock | TimedOut | Interrupted => continue, // retry
                        _ => {
                            rx_thread_stats.lock().unwrap().record_error();
                            eprintln!("Failed to read MAVLink message: {e}");
                        }
                    },
                    Err(MessageReadError::Parse(e)) => {
                        rx_thread_stats.lock().unwrap().record_error();
                        eprintln!("Failed to parse MAVLink message: {e}");
                    }
                }
            }
        });

        let tx_conn = connection.clone();
        let tx_ctx = ctx.clone();
        let tx_thread_stats = tx_stats.clone();
        // TX thread: sends outgoing MAVLink frames and notifies the UI of errors
        thread::spawn(move || {
            // Get the next outgoing frame
            while let Ok(outgoing) = outgoing_rx.recv() {
                // Send the frame out
                match tx_conn.send_frame(outgoing.frame) {
                    Ok(_) => tx_thread_stats.lock().unwrap().record_success(Instant::now()),
                    Err(error) => {
                        tx_thread_stats.lock().unwrap().record_error();
                        eprintln!("Failed to send MAVLink message: {error}");

                        let failure = SendFailure {
                            pending_slot: outgoing.pending_slot,
                            command_id: outgoing.command_id,
                        };
                        if send_failure_tx.send(failure).is_err() {
                            break;
                        }
                        tx_ctx.request_repaint_of(egui::ViewportId::ROOT);
                    }
                }
            }
        });

        Ok(Self {
            transport,
            mapping,
            ctx,
            stop_flag,
            incoming: incoming_rx,
            outgoing: outgoing_tx,
            send_failures: send_failure_rx,
            rx_stats,
            tx_stats,
            packet_sequence: Wrapping(0),
            pending: PendingCommandSlots::default(),
            protocol: make_protocol_descriptor(&profile),
            profile,
            ack_message_id,
            wack_message_id,
            nack_message_id,
            created_at,
        })
    }

    fn describe_protocol(&self) -> &ProtocolDescriptor {
        &self.protocol
    }

    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool {
        let mut processed = 0;

        // Receive the message from the RX thread
        while let Ok(frame) = self.incoming.try_recv() {
            // Skyward MAVLink dialect differentiates between telemetry and command messages by component ID
            match frame.header.component_id == TELEMETRY_COMPONENT_ID {
                true => self.handle_message(frame, data_store),
                false => self.handle_command(frame, data_store),
            }

            processed += 1;
        }

        processed > 0
    }

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

    fn status(&self) -> Status {
        let now = Instant::now();
        let rx = self.rx_stats.lock().unwrap().snapshot(now);
        let tx = self.tx_stats.lock().unwrap().snapshot(now);

        Status {
            transport: self.transport.clone(),
            mapping: self.mapping.clone(),
            rx,
            tx,
        }
    }
}

impl Drop for SkywardMavlinkAdapter {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
    }
}

impl SkywardMavlinkAdapter {
    fn handle_command(&mut self, frame: MavFrame, data_store: &mut DataStore) {
        let MavFrame { header, message, .. } = frame;

        let Some(message_info) = self.profile.messages.get(&message.id) else {
            eprintln!(
                "Cannot parse message with unknown ID {}, is the MAVLink profile correct?",
                message.id
            );
            return;
        };

        let pending_slot = header.component_id;
        let Some(command_id) = self.pending.get(pending_slot) else {
            eprintln!(
                "Received command response {} ({}) with invalid pending slot {pending_slot}",
                message_info.name, message.id
            );
            return;
        };

        let command_sequence = data_store.command_sequence_mut(command_id);
        // Handle ACK/WACK/NACK messages as special command status updates
        if self.update_sequence_status(command_sequence, message.id) {
            self.pending.release(pending_slot);
            return;
        }

        let target = command_sequence.request.target;

        match command_from_mav_message(message, message_info, target) {
            Ok(response) => {
                command_sequence.responses.push(response);
            }
            Err(error) => {
                eprintln!("Failed to process response for command {}: {error}", command_id.0);
            }
        }
    }

    fn handle_message(&mut self, frame: MavFrame, data_store: &mut DataStore) {
        let MavFrame { header, message, .. } = frame;

        let timestamp = Instant::now().duration_since(self.created_at).as_secs_f64();

        let Some(message_info) = self.profile.messages.get(&message.id) else {
            eprintln!(
                "Cannot parse message with unknown ID {}, is the MAVLink profile correct?",
                message.id
            );
            return;
        };

        for (i, (field, field_info)) in zip(message.fields, &message_info.fields).enumerate() {
            let stream_key = StreamKey {
                source_key: SourceKey(header.system_id as u32),
                data_key: compute_data_key(message.id, i as u32, &field_info.name),
            };

            let stream = match data_store.streams.entry(stream_key) {
                Entry::Occupied(e) => e.into_mut(),
                Entry::Vacant(e) => {
                    // Create the new stream since it doesn't exist yet
                    let new_stream = match field {
                        MsgField::Int8(_)
                        | MsgField::Int16(_)
                        | MsgField::Int32(_)
                        | MsgField::Int64(_)
                        | MsgField::UInt8(_)
                        | MsgField::UInt16(_)
                        | MsgField::UInt32(_)
                        | MsgField::UInt64(_) => DataStream::I64(Vec::new()),
                        MsgField::Float(_) | MsgField::Double(_) => DataStream::F64(Vec::new()),
                        MsgField::CharArray(_) => DataStream::String(Vec::new()),
                        _ => {
                            eprintln!(
                                "Unsupported field type for field {} in message ID {}",
                                field_info.name, message.id
                            );
                            continue;
                        }
                    };
                    e.insert(new_stream)
                }
            };

            if !insert_field_to_stream(field, stream, timestamp) {
                eprintln!(
                    "Type mismatch for field {} in message ID {}",
                    field_info.name, message.id
                );
            }
        }
    }

    fn expire_pending_commands(&mut self, data_store: &mut DataStore, now: Instant) {
        while let Some(PendingDeadline { sequence, expires_at }) = self.pending.oldest {
            if now < expires_at {
                break;
            }

            let pending =
                self.pending.slots[sequence as usize].expect("Oldest pending sequence must reference an occupied slot");

            self.pending.release(sequence);
            data_store.command_sequence_mut(pending.command_id).status = CommandStatus::TimedOut;
        }
    }

    fn schedule_pending_timeout(&self) {
        let Some(PendingDeadline { expires_at, .. }) = self.pending.oldest else {
            return;
        };

        self.ctx
            .request_repaint_after(expires_at.saturating_duration_since(Instant::now()));
    }

    /// Updates the status of a command sequence on ACK/WACK/NACK messages.
    ///
    /// Returns `true` if the status was updated to a final state.
    /// No more future responses are expected and associated resources may be released.
    fn update_sequence_status(&self, command_sequence: &mut CommandSequence, message_id: u32) -> bool {
        if message_id == self.ack_message_id || message_id == self.wack_message_id {
            command_sequence.status = CommandStatus::Success;
            true
        } else if message_id == self.nack_message_id {
            command_sequence.status = CommandStatus::Rejected;
            true
        } else {
            false
        }
    }
}

fn insert_field_to_stream(field: MsgField, stream: &mut DataStream, timestamp: f64) -> bool {
    match (field, stream) {
        (MsgField::Int8(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::Int16(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::Int32(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::Int64(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint { timestamp, value });
        }
        (MsgField::UInt8(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::UInt16(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::UInt32(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::UInt64(value), DataStream::I64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
        }
        (MsgField::Float(value), DataStream::F64(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value as f64,
            });
        }
        (MsgField::Double(value), DataStream::F64(inner_stream)) => {
            inner_stream.push(DataPoint { timestamp, value });
        }
        (MsgField::CharArray(value), DataStream::String(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: truncate_c_string(value),
            });
        }
        _ => return false,
    }
    true
}

/// Maps the `MavType` to the exact decoded `DataType` used by canonical message schemas.
fn mavtype_to_datatype(mav: MavType) -> DataType {
    match mav {
        MavType::UInt8 | MavType::Char => DataType::U8,
        MavType::UInt16 => DataType::U16,
        MavType::UInt32 => DataType::U32,
        MavType::UInt64 => DataType::U64,
        MavType::Int8 => DataType::I8,
        MavType::Int16 => DataType::I16,
        MavType::Int32 => DataType::I32,
        MavType::Int64 => DataType::I64,
        MavType::Float => DataType::F32,
        MavType::Double => DataType::F64,
        MavType::CharArray(_) => DataType::String,
        _ => unimplemented!("MAVLink {mav:?} type is not supported in message schemas"),
    }
}

fn command_to_mav_message(command: &Command, message_info: &segs_mavlink::MessageInfo) -> Result<MavMessage, String> {
    let fields = message_info
        .fields
        .iter()
        .enumerate()
        // Map each field to a `Result`
        // Allows collecting `Ok` values into a `Vec` or returning `Err` if any are invalid
        .map(|(i, field)| {
            if is_managed_tc_timestamp(message_info, &field.name) {
                return Ok(system_time_to_mav_field(SystemTime::now(), &field.mavtype));
            }

            let data_key = compute_data_key(message_info.id, i as u32, &field.name);
            let value = command
                .fields
                .get(&data_key)
                .ok_or_else(|| format!("Missing value for field '{}'", field.name))?;

            data_value_to_msg_field(value, &field.mavtype)
                .map_err(|error| format!("Invalid value for field '{}': {error}", field.name))
        })
        // `Result`'s `FromIterator` collects every `Ok` value into the `Vec`
        // If any `Err` is encountered (see above), that `Err` is returned
        .collect::<Result<Vec<_>, _>>()?;

    Ok(MavMessage {
        id: message_info.id,
        fields,
    })
}

/// Returns `true` if the field is a command timestamp.
///
/// Command timestamps are managed internally by the adapter and will not be included in the protocol descriptor.
fn is_managed_tc_timestamp(message_info: &segs_mavlink::MessageInfo, field_name: &str) -> bool {
    message_info.name.ends_with(TC_MESSAGE_SUFFIX) && field_name == TC_TIMESTAMP_FIELD
}

fn system_time_to_mav_field(timestamp: SystemTime, mav_type: &MavType) -> MsgField {
    let seconds = timestamp.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    match mav_type {
        MavType::UInt8MavlinkVersion | MavType::UInt8 => MsgField::UInt8(seconds as u8),
        MavType::UInt16 => MsgField::UInt16(seconds as u16),
        MavType::UInt32 => MsgField::UInt32(seconds as u32),
        MavType::UInt64 => MsgField::UInt64(seconds),
        MavType::Int8 => MsgField::Int8(seconds as i8),
        MavType::Int16 => MsgField::Int16(seconds as i16),
        MavType::Int32 => MsgField::Int32(seconds as i32),
        MavType::Int64 => MsgField::Int64(seconds as i64),
        MavType::Float => MsgField::Float(seconds as f32),
        MavType::Double => MsgField::Double(seconds as f64),
        MavType::Char => MsgField::Char(char::from(seconds as u8)),
        MavType::CharArray(_) | MavType::Array(_, _) => {
            unreachable!("Skyward TC timestamps must use a numeric MAVLink type")
        }
    }
}

/// Converts a correlated MAVLink response into the protocol-independent command representation.
fn command_from_mav_message(
    message: MavMessage,
    message_info: &segs_mavlink::MessageInfo,
    target: SourceKey,
) -> Result<Command, String> {
    let fields = zip(message.fields, &message_info.fields)
        .enumerate()
        .map(|(index, (field, field_info))| {
            let key = compute_data_key(message_info.id, index as u32, &field_info.name);
            msg_field_to_data_value(field).map(|value| (key, value))
        })
        .collect::<Result<_, _>>()?;

    Ok(Command {
        key: MessageKey(message.id as u64),
        timestamp: SystemTime::now(),
        target,
        fields,
    })
}

fn msg_field_to_data_value(field: MsgField) -> Result<DataValue, String> {
    match field {
        MsgField::UInt8(value) => Ok(DataValue::U8(value)),
        MsgField::UInt16(value) => Ok(DataValue::U16(value)),
        MsgField::UInt32(value) => Ok(DataValue::U32(value)),
        MsgField::UInt64(value) => Ok(DataValue::U64(value)),
        MsgField::Int8(value) => Ok(DataValue::I8(value)),
        MsgField::Int16(value) => Ok(DataValue::I16(value)),
        MsgField::Int32(value) => Ok(DataValue::I32(value)),
        MsgField::Int64(value) => Ok(DataValue::I64(value)),
        MsgField::Char(value) => u8::try_from(u32::from(value))
            .map(DataValue::U8)
            .map_err(|_| format!("Character {value:?} does not fit in a MAVLink byte")),
        MsgField::Float(value) => Ok(DataValue::F32(value)),
        MsgField::Double(value) => Ok(DataValue::F64(value)),
        MsgField::CharArray(value) => Ok(DataValue::String(truncate_c_string(value))),
        MsgField::Array(_) => Err("MAVLink array fields are not supported in command responses".into()),
    }
}

fn truncate_c_string(mut value: String) -> String {
    if let Some(terminator) = value.find('\0') {
        value.truncate(terminator);
    }
    value
}

/// Maps the `DataValue` to the corresponding `MsgField` based on the `MavType`.
fn data_value_to_msg_field(value: &DataValue, mav_type: &MavType) -> Result<MsgField, String> {
    let field = match (value, mav_type) {
        (DataValue::U8(value), MavType::UInt8) => MsgField::UInt8(*value),
        (DataValue::U16(value), MavType::UInt16) => MsgField::UInt16(*value),
        (DataValue::U32(value), MavType::UInt32) => MsgField::UInt32(*value),
        (DataValue::U64(value), MavType::UInt64) => MsgField::UInt64(*value),
        (DataValue::I8(value), MavType::Int8) => MsgField::Int8(*value),
        (DataValue::I16(value), MavType::Int16) => MsgField::Int16(*value),
        (DataValue::I32(value), MavType::Int32) => MsgField::Int32(*value),
        (DataValue::I64(value), MavType::Int64) => MsgField::Int64(*value),
        (DataValue::F32(value), MavType::Float) => MsgField::Float(*value),
        (DataValue::F64(value), MavType::Double) => MsgField::Double(*value),
        (DataValue::U8(value), MavType::Char) => MsgField::Char(char::from(*value)),
        (DataValue::String(value), MavType::CharArray(length)) => {
            if value.len() > *length {
                return Err(format!(
                    "String is {} bytes long, but the MAVLink field allows at most {length}",
                    value.len()
                ));
            }

            let mut value = value.clone();
            value.extend(std::iter::repeat_n('\0', *length - value.len()));
            MsgField::CharArray(value)
        }
        (_, MavType::Array(_, _)) => return Err("MAVLink array fields are not supported".into()),
        _ => {
            return Err(format!(
                "Data type {:?} does not match the MAVLink field type {:?}",
                value, mav_type
            ));
        }
    };

    Ok(field)
}

fn compute_data_key(message_id: u32, field_id: u32, field_name: &str) -> DataKey {
    let mut hasher = DefaultHasher::new();
    message_id.hash(&mut hasher);
    field_id.hash(&mut hasher);
    field_name.hash(&mut hasher);

    DataKey(hasher.finish())
}

fn make_protocol_descriptor(profile: &MavProfile) -> ProtocolDescriptor {
    // Build each stream-visible or sendable schema exactly once
    let mut messages = profile.messages.values().collect::<Vec<_>>();
    messages.sort_by(|left, right| left.name.cmp(&right.name));

    let message_schemas = messages
        .iter()
        .map(|message| {
            let fields = message
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| !is_managed_tc_timestamp(message, &field.name))
                .map(|(i, field)| {
                    let data_key = compute_data_key(message.id, i as u32, &field.name);

                    // Preserve ordinary enum metadata when the referenced declaration is usable
                    if let Some(descriptor) = field
                        .enumtype
                        .as_ref()
                        .and_then(|name| profile.enums.get(name))
                        .and_then(|mavenum| make_enum_descriptor(mavenum, &field.mavtype))
                    {
                        FieldDescriptor::EnumField {
                            name: field.name.clone(),
                            descriptor,
                            data_key,
                        }
                    } else {
                        FieldDescriptor::Field {
                            name: field.name.clone(),
                            field_type: mavtype_to_datatype(field.mavtype.clone()),
                            data_key,
                        }
                    }
                })
                .collect();
            let key = MessageKey(message.id as u64);

            let descriptor = MessageDescriptor {
                name: message.name.clone(),
                fields,
            };

            (key, descriptor)
        })
        .collect();

    // Roles contain references and preserve the canonical name ordering
    let stream_messages = messages
        .iter()
        .filter(|message| !message.name.ends_with(TC_MESSAGE_SUFFIX))
        .map(|message| MessageKey(message.id as u64))
        .collect();
    let command_messages = messages
        .iter()
        .filter(|message| message.name.ends_with(TC_MESSAGE_SUFFIX))
        .map(|message| MessageKey(message.id as u64))
        .collect();

    let sources = profile
        .enums
        .iter()
        .find(|(name, _)| *name == "Sysids") // Weird capitalization by mavlink parser
        .map(|(_, mavenum)| {
            mavenum
                .entries
                .iter()
                .map(|entry| SourceDescriptor {
                    name: entry.name.clone(),
                    key: SourceKey(entry.value.expect("Found SysID enum member without explicit value") as u32),
                })
                .collect()
        })
        .unwrap_or_default();

    ProtocolDescriptor {
        message_schemas,
        stream_messages,
        command_messages,
        sources,
    }
}

/// Builds protocol-independent metadata for an ordinary MAVLink enum.
///
/// Returns the enum name and its declaration-ordered typed values. `None`
/// means the enum is empty, represents a bitmask, or contains a value that
/// cannot be represented by the MAVLink field type.
fn make_enum_descriptor(mavenum: &segs_mavlink::EnumInfo, mavtype: &MavType) -> Option<EnumDescriptor> {
    if mavenum.bitmask || mavenum.entries.is_empty() {
        return None;
    }

    let mut previous_value = 0;
    let mut variants = Vec::with_capacity(mavenum.entries.len());

    // Resolve implicit values using the MAVLink parser's declaration semantics
    for entry in &mavenum.entries {
        let value = match entry.value {
            Some(value) => {
                previous_value = previous_value.max(value);
                value
            }
            None => {
                previous_value = previous_value.checked_add(1)?;
                previous_value
            }
        };
        variants.push((entry.name.clone(), enum_value_to_data_value(value, mavtype)?));
    }

    Some(EnumDescriptor {
        name: mavenum.name.clone(),
        variants,
    })
}

/// Converts an unsigned MAVLink enum code to the field's exact primitive value.
///
/// Returns the typed value when the code fits the integer field type, or
/// `None` for unsupported types and out-of-range codes.
fn enum_value_to_data_value(value: u64, mavtype: &MavType) -> Option<DataValue> {
    match mavtype {
        MavType::UInt8 | MavType::Char => u8::try_from(value).ok().map(DataValue::U8),
        MavType::UInt16 => u16::try_from(value).ok().map(DataValue::U16),
        MavType::UInt32 => u32::try_from(value).ok().map(DataValue::U32),
        MavType::UInt64 => Some(DataValue::U64(value)),
        MavType::Int8 => i8::try_from(value).ok().map(DataValue::I8),
        MavType::Int16 => i16::try_from(value).ok().map(DataValue::I16),
        MavType::Int32 => i32::try_from(value).ok().map(DataValue::I32),
        MavType::Int64 => i64::try_from(value).ok().map(DataValue::I64),
        MavType::UInt8MavlinkVersion
        | MavType::Float
        | MavType::Double
        | MavType::CharArray(_)
        | MavType::Array(_, _) => None,
    }
}

/// Tracks cumulative I/O totals and recent successful frames for rate calculation.
struct IoStats {
    stats: Stats,
    recent_frames: VecDeque<Instant>,
}

impl IoStats {
    fn new(created_at: Instant) -> Self {
        Self {
            stats: Stats {
                last_time: created_at,
                rate: 0.,
                count: 0,
                errors: 0,
            },
            recent_frames: VecDeque::new(),
        }
    }

    fn record_success(&mut self, now: Instant) {
        self.recent_frames.push_back(now);
        self.update_rate(now);
        self.stats.last_time = now;
        self.stats.count = self.stats.count.saturating_add(1);
    }

    fn record_error(&mut self) {
        self.stats.errors = self.stats.errors.saturating_add(1);
    }

    fn snapshot(&mut self, now: Instant) -> Stats {
        self.update_rate(now);
        self.stats
    }

    fn update_rate(&mut self, now: Instant) {
        while self
            .recent_frames
            .front()
            .is_some_and(|frame| now.saturating_duration_since(*frame) >= STATS_RATE_WINDOW)
        {
            self.recent_frames.pop_front();
        }

        self.stats.rate = self.recent_frames.len() as f32 / STATS_RATE_WINDOW.as_secs_f32();
    }
}

/// Adapter-local state for pending commands, tracking which command sequence IDs are in use.
struct PendingCommandSlots {
    slots: [Option<PendingCommandSlot>; 256],
    next_sequence: u8,
    oldest: Option<PendingDeadline>,
}

impl Default for PendingCommandSlots {
    fn default() -> Self {
        Self {
            slots: [None; 256],
            next_sequence: 1,
            oldest: None,
        }
    }
}

impl PendingCommandSlots {
    fn acquire(&mut self, command_id: CommandId) -> Option<u8> {
        let mut sequence = self.next_sequence;

        // Find the next free slot
        while self.slots[sequence as usize].is_some() {
            sequence = Self::advance(sequence);
            if sequence == self.next_sequence {
                return None;
            }
        }

        let sent_at = Instant::now();
        self.slots[sequence as usize] = Some(PendingCommandSlot { command_id, sent_at });
        self.oldest.get_or_insert(PendingDeadline {
            sequence,
            expires_at: sent_at + COMMAND_TIMEOUT,
        });
        self.next_sequence = Self::advance(sequence);
        Some(sequence)
    }

    fn release(&mut self, sequence: u8) {
        self.slots[sequence as usize] = None;

        if self.oldest.is_some_and(|oldest| oldest.sequence == sequence) {
            self.oldest = self
                .slots
                .iter()
                .enumerate()
                .filter_map(|(index, slot)| Some((index as u8, slot.as_ref()?.sent_at)))
                .min_by_key(|(_, sent_at)| *sent_at)
                .map(|(sequence, sent_at)| PendingDeadline {
                    sequence,
                    expires_at: sent_at + COMMAND_TIMEOUT,
                });
        }
    }

    fn get(&self, sequence: u8) -> Option<CommandId> {
        self.slots[sequence as usize].map(|slot| slot.command_id)
    }

    fn advance(sequence: u8) -> u8 {
        sequence.checked_add(1).unwrap_or(1)
    }
}

#[derive(Clone, Copy)]
struct PendingCommandSlot {
    command_id: CommandId,
    sent_at: Instant,
}

#[derive(Clone, Copy)]
struct PendingDeadline {
    sequence: u8,
    expires_at: Instant,
}

struct OutgoingCommand {
    frame: MavFrame,
    pending_slot: u8,
    command_id: CommandId,
}

#[derive(Clone, Copy)]
struct SendFailure {
    pending_slot: u8,
    command_id: CommandId,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use egui::ahash::{HashMap, HashMapExt};

    use super::*;

    fn test_command() -> Command {
        Command {
            key: MessageKey(1),
            target: SourceKey(1),
            timestamp: SystemTime::now(),
            fields: HashMap::new(),
        }
    }

    #[test]
    fn correlation_slots_allocate_sequentially_and_wrap_without_using_zero() {
        let mut slots = PendingCommandSlots::default();

        for expected in 1..=u8::MAX {
            let command_id = CommandId(u64::from(expected));
            let allocated = slots.acquire(command_id).unwrap();
            assert_eq!(allocated, expected);
            assert_eq!(slots.get(allocated), Some(command_id));
            slots.release(allocated);
            assert_eq!(slots.get(allocated), None);
        }

        assert_eq!(slots.acquire(CommandId(256)), Some(1));
        assert_eq!(slots.get(TELEMETRY_COMPONENT_ID), None);
    }

    #[test]
    fn correlation_slots_skip_active_ids_and_report_exhaustion() {
        let mut slots = PendingCommandSlots::default();
        assert_eq!(slots.acquire(CommandId(1000)), Some(1));

        assert_eq!(slots.acquire(CommandId(1001)), Some(2));

        let mut full = PendingCommandSlots::default();
        for id in 0..u8::MAX {
            full.acquire(CommandId(u64::from(id))).unwrap();
        }
        // All slots should be taken, so allocation should fail
        assert_eq!(full.acquire(CommandId(256)), None);

        full.release(42);
        assert_eq!(full.acquire(CommandId(257)), Some(42));
        assert_eq!(full.get(42), Some(CommandId(257)));
    }

    #[test]
    fn acquired_slots_are_unavailable_until_released() {
        let mut slots = PendingCommandSlots::default();

        assert_eq!(slots.acquire(CommandId(1)), Some(1));
        assert_eq!(slots.get(1), Some(CommandId(1)));
        assert_eq!(slots.acquire(CommandId(2)), Some(2));
        slots.release(1);
        slots.next_sequence = 1;
        assert_eq!(slots.acquire(CommandId(3)), Some(1));
        assert_eq!(slots.get(1), Some(CommandId(3)));
    }

    #[test]
    fn canonical_datatypes_preserve_mavlink_scalar_widths() {
        assert!(matches!(mavtype_to_datatype(MavType::UInt8), DataType::U8));
        assert!(matches!(mavtype_to_datatype(MavType::UInt16), DataType::U16));
        assert!(matches!(mavtype_to_datatype(MavType::UInt32), DataType::U32));
        assert!(matches!(mavtype_to_datatype(MavType::UInt64), DataType::U64));
        assert!(matches!(mavtype_to_datatype(MavType::Int8), DataType::I8));
        assert!(matches!(mavtype_to_datatype(MavType::Int16), DataType::I16));
        assert!(matches!(mavtype_to_datatype(MavType::Int32), DataType::I32));
        assert!(matches!(mavtype_to_datatype(MavType::Int64), DataType::I64));
        assert!(matches!(mavtype_to_datatype(MavType::Float), DataType::F32));
        assert!(matches!(mavtype_to_datatype(MavType::Double), DataType::F64));
        assert!(matches!(mavtype_to_datatype(MavType::Char), DataType::U8));
        assert!(matches!(mavtype_to_datatype(MavType::CharArray(4)), DataType::String));
    }

    #[test]
    fn protocol_descriptor_reuses_exact_schemas_through_sorted_roles() {
        let message = |id, name: &str, mavtype| {
            let mut message = segs_mavlink::MessageInfo {
                id,
                name: name.into(),
                ..Default::default()
            };
            message.fields.push(Default::default());
            message.fields[0].name = "value".into();
            message.fields[0].mavtype = mavtype;
            message
        };
        let mut alpha_tm = message(2, "ALPHA_TM", MavType::Float);
        alpha_tm.fields.push(Default::default());
        alpha_tm.fields[1].name = TC_TIMESTAMP_FIELD.into();
        alpha_tm.fields[1].mavtype = MavType::UInt32;
        let mut alpha_tc = message(4, "ALPHA_TC", MavType::Double);
        alpha_tc.fields.push(Default::default());
        alpha_tc.fields[1].name = TC_TIMESTAMP_FIELD.into();
        alpha_tc.fields[1].mavtype = MavType::UInt32;
        let profile = MavProfile {
            enums: BTreeMap::new(),
            messages: BTreeMap::from([
                (1, message(1, "ZETA_TM", MavType::UInt8)),
                (2, alpha_tm),
                (3, message(3, "BETA_TC", MavType::UInt16)),
                (4, alpha_tc),
                (5, message(5, "UNSUFFIXED", MavType::Int32)),
            ]),
        };

        let descriptor = make_protocol_descriptor(&profile);
        let stream_names = descriptor
            .stream_messages
            .iter()
            .filter_map(|key| descriptor.message_schemas.get(key).map(|message| message.name.as_str()))
            .collect::<Vec<_>>();
        let sendable_names = descriptor
            .command_messages
            .iter()
            .filter_map(|key| descriptor.message_schemas.get(key).map(|message| message.name.as_str()))
            .collect::<Vec<_>>();

        assert_eq!(descriptor.message_schemas.len(), 5);
        assert_eq!(stream_names, ["ALPHA_TM", "UNSUFFIXED", "ZETA_TM"]);
        assert_eq!(sendable_names, ["ALPHA_TC", "BETA_TC"]);
        assert!(matches!(
            descriptor
                .message_schemas
                .get(&MessageKey(1))
                .unwrap()
                .fields
                .as_slice(),
            [FieldDescriptor::Field {
                field_type: DataType::U8,
                ..
            }]
        ));
        assert!(matches!(
            descriptor
                .message_schemas
                .get(&MessageKey(2))
                .unwrap()
                .fields
                .as_slice(),
            [
                FieldDescriptor::Field {
                    field_type: DataType::F32,
                    ..
                },
                FieldDescriptor::Field {
                    name,
                    field_type: DataType::U32,
                    ..
                }
            ] if name == TC_TIMESTAMP_FIELD
        ));
        assert!(matches!(
            descriptor
                .message_schemas
                .get(&MessageKey(4))
                .unwrap()
                .fields
                .as_slice(),
            [FieldDescriptor::Field {
                field_type: DataType::F64,
                ..
            }]
        ));
    }

    #[test]
    fn converts_every_supported_command_scalar() {
        assert_eq!(
            data_value_to_msg_field(&DataValue::U8(1), &MavType::UInt8).unwrap(),
            MsgField::UInt8(1)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::U16(2), &MavType::UInt16).unwrap(),
            MsgField::UInt16(2)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::U32(3), &MavType::UInt32).unwrap(),
            MsgField::UInt32(3)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::U64(4), &MavType::UInt64).unwrap(),
            MsgField::UInt64(4)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::I8(-1), &MavType::Int8).unwrap(),
            MsgField::Int8(-1)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::I16(-2), &MavType::Int16).unwrap(),
            MsgField::Int16(-2)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::I32(-3), &MavType::Int32).unwrap(),
            MsgField::Int32(-3)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::I64(-4), &MavType::Int64).unwrap(),
            MsgField::Int64(-4)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::F32(1.5), &MavType::Float).unwrap(),
            MsgField::Float(1.5)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::F64(2.5), &MavType::Double).unwrap(),
            MsgField::Double(2.5)
        );
        assert_eq!(
            data_value_to_msg_field(&DataValue::U8(b'A'), &MavType::Char).unwrap(),
            MsgField::Char('A')
        );
    }

    #[test]
    fn fixed_length_strings_are_padded_and_oversized_values_are_rejected() {
        assert_eq!(
            data_value_to_msg_field(&DataValue::String("TC".into()), &MavType::CharArray(4)).unwrap(),
            MsgField::CharArray("TC\0\0".into())
        );
        assert!(data_value_to_msg_field(&DataValue::String("TOO LONG".into()), &MavType::CharArray(4)).is_err());
    }

    #[test]
    fn mismatched_and_unsupported_values_are_rejected() {
        assert!(data_value_to_msg_field(&DataValue::U16(1), &MavType::UInt8).is_err());
        assert!(data_value_to_msg_field(&DataValue::Bool(true), &MavType::UInt8).is_err());
        assert!(data_value_to_msg_field(&DataValue::U8(1), &MavType::Array(Box::new(MavType::UInt8), 2)).is_err());
    }

    #[test]
    fn command_conversion_uses_profile_order_and_requires_every_field() {
        let mut message = segs_mavlink::MessageInfo {
            id: 42,
            name: "ORDERED_TC".into(),
            ..Default::default()
        };
        message.fields.push(Default::default());
        message.fields[0].name = "first".into();
        message.fields[0].mavtype = MavType::UInt8;
        message.fields.push(Default::default());
        message.fields[1].name = "second".into();
        message.fields[1].mavtype = MavType::Int16;

        let first_key = compute_data_key(message.id, 0, "first");
        let second_key = compute_data_key(message.id, 1, "second");
        let mut fields = HashMap::new();
        fields.insert(second_key, DataValue::I16(-2));
        fields.insert(first_key, DataValue::U8(1));
        let mut command = Command {
            key: MessageKey(message.id as u64),
            timestamp: SystemTime::now(),
            target: SourceKey(7),
            fields,
        };

        assert_eq!(
            command_to_mav_message(&command, &message).unwrap(),
            MavMessage {
                id: message.id,
                fields: vec![MsgField::UInt8(1), MsgField::Int16(-2)],
            }
        );

        command.fields.remove(&first_key);
        assert!(command_to_mav_message(&command, &message).is_err());
    }

    #[test]
    fn command_conversion_populates_managed_tc_timestamp() {
        let mut message = segs_mavlink::MessageInfo {
            id: 43,
            name: "TIMED_TC".into(),
            ..Default::default()
        };
        message.fields.push(Default::default());
        message.fields[0].name = TC_TIMESTAMP_FIELD.into();
        message.fields[0].mavtype = MavType::UInt64;
        message.fields.push(Default::default());
        message.fields[1].name = "value".into();
        message.fields[1].mavtype = MavType::UInt8;

        let before = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let command = Command {
            key: MessageKey(message.id as u64),
            timestamp: SystemTime::now(),
            target: SourceKey(7),
            fields: [(compute_data_key(message.id, 1, "value"), DataValue::U8(9))]
                .into_iter()
                .collect(),
        };
        let converted = command_to_mav_message(&command, &message).unwrap();
        let after = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        assert!(matches!(
            converted.fields.as_slice(),
            [MsgField::UInt64(timestamp), MsgField::UInt8(9)]
                if (before..=after).contains(timestamp)
        ));
        assert_eq!(
            system_time_to_mav_field(UNIX_EPOCH + std::time::Duration::from_secs(257), &MavType::UInt8),
            MsgField::UInt8(1)
        );
    }

    #[test]
    fn correlated_response_conversion_preserves_command_value_types() {
        let mut message_info = segs_mavlink::MessageInfo {
            id: 24,
            name: "RESPONSE_TM".into(),
            ..Default::default()
        };
        message_info.fields.push(Default::default());
        message_info.fields[0].name = "count".into();
        message_info.fields[0].mavtype = MavType::UInt16;
        message_info.fields.push(Default::default());
        message_info.fields[1].name = "value".into();
        message_info.fields[1].mavtype = MavType::Float;
        message_info.fields.push(Default::default());
        message_info.fields[2].name = "sensor_name".into();
        message_info.fields[2].mavtype = MavType::CharArray(24);

        let response = command_from_mav_message(
            MavMessage {
                id: 24,
                fields: vec![
                    MsgField::UInt16(12),
                    MsgField::Float(3.5),
                    MsgField::CharArray("AS5047D_LEFT\0\0\0f\u{14}\u{d0}".into()),
                ],
            },
            &message_info,
            SourceKey(7),
        )
        .unwrap();

        assert_eq!(response.target, SourceKey(7));
        assert!(matches!(
            response.fields.get(&compute_data_key(24, 0, "count")),
            Some(DataValue::U16(12))
        ));
        assert!(matches!(
            response.fields.get(&compute_data_key(24, 1, "value")),
            Some(DataValue::F32(value)) if *value == 3.5
        ));
        assert!(matches!(
            response.fields.get(&compute_data_key(24, 2, "sensor_name")),
            Some(DataValue::String(value)) if value == "AS5047D_LEFT"
        ));
    }

    #[test]
    fn char_array_stream_values_use_c_string_termination() {
        let mut stream = DataStream::String(Vec::new());

        assert!(insert_field_to_stream(
            MsgField::CharArray("AS5047D_LEFT\0\0garbage".into()),
            &mut stream,
            1.,
        ));
        assert!(insert_field_to_stream(
            MsgField::CharArray("FULL_LENGTH".into()),
            &mut stream,
            2.,
        ));

        let DataStream::String(points) = stream else {
            unreachable!()
        };
        assert_eq!(points[0].value, "AS5047D_LEFT");
        assert_eq!(points[1].value, "FULL_LENGTH");
    }
}
