use std::error::Error;
use std::iter::zip;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, mpsc::Receiver};
use std::thread;
use std::time::{Duration, Instant};

use segs_mavlink::connection::{Connection, MavConnection};
use segs_mavlink::{MavFrame, MavProfile, MsgField};

use crate::dataflow::adapter::DataAdapter;
use crate::dataflow::mapping::{DataMapping, MappingDescriptor, MappingType};
use crate::dataflow::transport::DataTransport;
use crate::dataflow::{DataKey, DataPoint, DataStore, DataStream};

/// Adapter implementation for MAVLink protocol.
/// Uses a local XML file mapping source that defines the MAVLink message formats to be processed
pub struct MavlinkAdapter {
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
        let profile = match mapping {
            DataMapping::LocalFile(path) => {
                let mav_profile = segs_mavlink::parse_profile(&path)?;
                Arc::new(segs_mavlink::MavProfile::from_profile_info(&mav_profile))
            }
            _ => return Err("Unsupported definition source method".into()),
        };

        let (tx, rx) = mpsc::channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let thread_stop_flag = stop_flag.clone();

        let connection = match transport {
            DataTransport::Ethernet {
                recv_socket,
                send_socket,
            } => Connection::udp(recv_socket, send_socket, profile.clone())?,
            DataTransport::Serial { tty, baud_rate } => Connection::serial(tty, baud_rate, profile.clone())?,
        };

        thread::spawn(move || {
            while !thread_stop_flag.load(Ordering::Relaxed) {
                match connection.recv_frame() {
                    Ok(frame) => {
                        let Ok(_) = tx.send(frame) else {
                            break; // Receiver has been dropped, exit the thread
                        };
                    }
                    Err(e) => {
                        eprintln!("Error receiving MAVLink frame: {:?}", e);
                    }
                }
            }
        });

        Ok(Self {
            stop_flag,
            incoming: rx,
            profile,
            created_at: Instant::now(),
        })
    }

    fn process_incoming(&mut self, data_store: &mut DataStore) -> bool {
        // Limit processing time to avoid UI lag
        const MAX_PROCESSING_TIME: Duration = Duration::from_millis(5);
        let start_time = Instant::now();
        let mut i = 0;

        for MavFrame { header, message, .. } in self.incoming.try_iter() {
            let timestamp = Instant::now().duration_since(self.created_at).as_secs_f64();
            println!("[{:<10.3}] Received MAVLink message: {:?}", timestamp, message);

            let Some(message_info) = self.profile.messages.get(&message.id) else {
                eprintln!("Unknown message ID: {}", message.id);
                continue;
            };

            for (i, (field, field_info)) in zip(message.fields.into_iter(), &message_info.fields).enumerate() {
                let stream_key = DataKey {
                    source_id: header.system_id as u32,
                    message_id: message.id,
                    field_hash: i as u32,
                };

                // TODO: need a way to distinguish between stream and command
                match data_store.streams.get_mut(&stream_key) {
                    Some(stream) => {
                        if !insert_field_to_stream(field, stream, timestamp) {
                            eprintln!(
                                "Type mismatch for field {} in message ID {}",
                                field_info.name, message.id
                            );
                        }
                    }

                    None => {
                        // Create the new stream since it doesn't exist yet
                        let mut stream = match field {
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

                        // Insert as usual
                        if !insert_field_to_stream(field, &mut stream, timestamp) {
                            eprintln!(
                                "Type mismatch for field {} in message ID {}",
                                field_info.name, message.id
                            );
                        }

                        data_store.streams.insert(stream_key, stream);
                    }
                }
            }

            i += 1;
            if Instant::now().duration_since(start_time) > MAX_PROCESSING_TIME {
                break; // Stop processing if we've exceeded the time limit
            }
        }

        if i <= 0 {
            return false;
        }

        println!("Processed {} messages", i);
        return true;
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
