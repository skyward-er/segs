use std::net::SocketAddrV4;
use std::path::PathBuf;

use crate::dataflow::{
    adapter::AdapterType,
    mapping::DataMapping,
    transport::{DataTransport, TransportType},
};
use argh::{ArgsInfo, FromArgs};

#[derive(FromArgs, ArgsInfo)]
/// Skyward Enhanced Ground Software (SEGS) - A telemetry analysis and visualization tool.
#[argh(example = "\
    {command_name} --transport serial --tty /dev/ttyUSB0 --baudrate 115200\n\
    {command_name} --transport ethernet --recv-socket 0.0.0.0:42069 --send-socket 169.254.255.255:42070")]
struct CliArgs {
    /// transport type to be used at startup (serial, ethernet)
    #[argh(option)]
    transport: Option<TransportType>,

    /// serial port to connect to (for serial transport)
    #[argh(option)]
    tty: Option<String>,

    /// baud rate for serial connection (for serial transport)
    #[argh(option)]
    baudrate: Option<u32>,

    /// ip:port to listen on (for ethernet transport)
    #[argh(option)]
    recv_socket: Option<SocketAddrV4>,

    /// ip:port to send data to (for ethernet transport)
    #[argh(option)]
    send_socket: Option<SocketAddrV4>,

    /// data adapter used for processing incoming data (mavlink)
    #[argh(option)]
    adapter: Option<AdapterType>,

    /// data mapping file passed to the adapter (when adapter is specified)
    #[argh(option)]
    mapping_file: Option<PathBuf>,
}

#[derive(Default)]
pub struct AppArgs {
    pub transport: Option<DataTransport>,
    pub adapter: Option<AdapterType>,
    pub mapping: Option<DataMapping>,
}

pub fn parse_args() -> Result<AppArgs, Box<dyn std::error::Error>> {
    let cli_args: CliArgs = argh::from_env();
    let mut args = AppArgs::default();

    // Validate transport arguments
    match cli_args.transport {
        Some(TransportType::Serial) => {
            let (tty, baudrate) = match (cli_args.tty, cli_args.baudrate) {
                (Some(tty), Some(baudrate)) => (tty, baudrate),
                _ => return Err("Error: Serial transport selected but tty or baudrate missing.".into()),
            };

            args.transport = Some(DataTransport::Serial {
                tty,
                baud_rate: baudrate,
            });
        }
        Some(TransportType::Ethernet) => {
            let (recv_socket, send_socket) = match (cli_args.recv_socket, cli_args.send_socket) {
                (Some(recv), Some(send)) => (recv, send),
                _ => return Err("Error: Ethernet transport selected but recv or send socket missing.".into()),
            };

            args.transport = Some(DataTransport::Ethernet {
                recv_socket,
                send_socket,
            });
        }
        None => {}
    };

    // Validate adapter arguments
    match cli_args.adapter {
        Some(AdapterType::Mavlink) => {
            args.adapter = Some(AdapterType::Mavlink);
            // Ensure a mapping file is provided if an adapter was specified
            let Some(mapping_file) = cli_args.mapping_file else {
                return Err("Error: MAVLink adapter selected but no mapping file provided.".into());
            };
            args.mapping = Some(DataMapping::LocalFile(mapping_file));
        }
        None => {}
    }

    Ok(args)
}
