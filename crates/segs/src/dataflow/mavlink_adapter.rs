use std::collections::hash_map::{DefaultHasher, Entry};
use std::error::Error;
use std::hash::{Hash, Hasher};
use std::io::ErrorKind::{Interrupted, TimedOut, WouldBlock};
use std::iter::zip;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc, mpsc::Receiver};
use std::thread;
use std::time::Instant;

use segs_mavlink::connection::{Connection, MavConnection};
use segs_mavlink::{MavFrame, MavProfile, MavType, MessageReadError, MsgField};

use crate::dataflow::adapter::{DataAdapter, Status};
use crate::dataflow::mapping::{DataMapping, MappingDescriptor, MappingType};
use crate::dataflow::protocol::{FieldDescriptor, ProtocolDescriptor, SourceDescriptor};
use crate::dataflow::transport::DataTransport;
use crate::dataflow::{DataKey, DataPoint, DataStore, DataStream, DataType, SourceKey, StreamKey};

/// Adapter implementation for MAVLink protocol.
/// Uses a local XML file mapping source that defines the MAVLink message formats to be processed
pub struct MavlinkAdapter {
    transport: DataTransport,
    mapping: DataMapping,
    stop_flag: Arc<AtomicBool>,
    incoming: Receiver<MavFrame>,
    profile: Arc<MavProfile>,
    created_at: Instant,
}

impl DataAdapter for MavlinkAdapter {
    fn get_mapping_sources() -> Vec<MappingDescriptor>
    where
        Self: Sized,
    {
        vec![MappingDescriptor {
            method: MappingType::LocalFile,
            description: "MAVLink XML message definition file".into(),
        }]
    }

    fn new(transport: DataTransport, mapping: DataMapping) -> Result<Self, Box<dyn Error>> {
        let profile = match &mapping {
            DataMapping::LocalFile(path) => {
                let mav_profile = segs_mavlink::parse_profile(&path)?;
                Arc::new(segs_mavlink::MavProfile::from_profile_info(&mav_profile))
            }
            _ => return Err("Unsupported definition source method".into()),
        };

        let (tx, rx) = mpsc::channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();

        let connection = match &transport {
            DataTransport::Ethernet {
                recv_socket,
                send_socket,
            } => Connection::udp(recv_socket, send_socket.clone(), profile.clone())?,
            DataTransport::Serial { tty, baud_rate } => Connection::serial(tty.clone(), *baud_rate, profile.clone())?,
        };

        thread::spawn(move || {
            while !thread_stop_flag.load(Ordering::Relaxed) {
                match connection.recv_frame() {
                    Ok(frame) => {
                        let Ok(_) = tx.send(frame) else {
                            break; // Receiver has been dropped, exit the thread
                        };
                    }
                    Err(MessageReadError::Io(e)) => match e.kind() {
                        WouldBlock | TimedOut | Interrupted => continue, // retry
                        _ => eprintln!("Failed to read MAVLink message: {e}"),
                    },
                    Err(MessageReadError::Parse(e)) => {
                        eprintln!("Failed to parse MAVLink message: {e}");
                    }
                }
            }
        });

        Ok(Self {
            transport,
            mapping,
            stop_flag,
            incoming: rx,
            profile,
            created_at: Instant::now(),
        })
    }

    fn describe_protocol(&self) -> ProtocolDescriptor {
        let messages = self
            .profile
            .messages
            .values()
            .map(|message| {
                let fields = message
                    .fields
                    .iter()
                    .enumerate()
                    .map(|(i, field)| FieldDescriptor::Field {
                        name: field.name.clone(),
                        data_key: compute_data_key(message.id, i as u32, &field.name),
                        field_type: mavtype_to_datatype(field.mavtype.clone()),
                    })
                    .collect();

                FieldDescriptor::Structure {
                    name: message.name.clone(),
                    fields,
                }
            })
            .collect();

        let sources = self
            .profile
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

        ProtocolDescriptor { messages, sources }
    }

    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool {
        let mut i = 0;

        for MavFrame { header, message, .. } in self.incoming.try_iter() {
            let timestamp = Instant::now().duration_since(self.created_at).as_secs_f64();
            println!("[{:<10.3}] Received MAVLink message: {:?}", timestamp, message);

            let Some(message_info) = self.profile.messages.get(&message.id) else {
                eprintln!("Unknown message ID: {}", message.id);
                continue;
            };

            for (i, (field, field_info)) in zip(message.fields.into_iter(), &message_info.fields).enumerate() {
                let stream_key = StreamKey {
                    source_key: SourceKey(header.system_id as u32),
                    data_key: compute_data_key(message.id, i as u32, &field_info.name),
                };

                // TODO: need a way to distinguish between stream and command
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

            i += 1;
        }

        i > 0
    }

    fn status(&self) -> Status {
        Status {
            transport: self.transport.clone(),
            mapping: self.mapping.clone(),
            rx: Default::default(),
            tx: Default::default(),
        }
    }
}

impl Drop for MavlinkAdapter {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
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
            inner_stream.push(DataPoint {
                timestamp,
                value: value as i64,
            });
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
            inner_stream.push(DataPoint {
                timestamp,
                value: value,
            });
        }
        (MsgField::CharArray(value), DataStream::String(inner_stream)) => {
            inner_stream.push(DataPoint {
                timestamp,
                value: value,
            });
        }
        _ => return false,
    }
    true
}

fn mavtype_to_datatype(mav: MavType) -> DataType {
    match mav {
        MavType::Char
        | MavType::UInt8
        | MavType::UInt16
        | MavType::UInt32
        | MavType::UInt64
        | MavType::Int8
        | MavType::Int16
        | MavType::Int32
        | MavType::Int64 => DataType::I64,
        MavType::Float | MavType::Double => DataType::F64,
        MavType::CharArray(_) => DataType::String,
        _ => unimplemented!("Non-primitive MavTypes are not supported"),
    }
}

fn compute_data_key(message_id: u32, field_id: u32, field_name: &str) -> DataKey {
    let mut hasher = DefaultHasher::new();
    message_id.hash(&mut hasher);
    field_id.hash(&mut hasher);
    field_name.hash(&mut hasher);

    DataKey(hasher.finish())
}
