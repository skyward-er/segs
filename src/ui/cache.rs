use std::time::{Duration, Instant};

use egui::Context;
use serialport::SerialPortInfo;

use crate::{communication, error::ErrInstrument};

const SERIAL_PORT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

fn call<T, F>(ctx: &egui::Context, id: egui::Id, fun: F, expiration_duration: Duration) -> T
where
    F: Fn() -> T,
    T: Clone + Send + Sync + 'static,
{
    ctx.memory_mut(|m| {
        match m.data.get_temp::<(T, Instant)>(id) {
            None => {
                m.data.insert_temp(id, (fun(), Instant::now()));
            }
            Some((_, i)) if i.elapsed() >= expiration_duration => {
                m.data.insert_temp(id, (fun(), Instant::now()));
            }
            _ => {}
        }
        m.data.get_temp::<(T, Instant)>(id).log_unwrap().0
    })
}

pub trait CacheCall {
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static;
}

impl CacheCall for egui::Context {
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static,
    {
        call(&self, id, fun, expiration_duration)
    }
}

pub fn cached_list_all_usb_ports(ctx: &Context) -> Result<Vec<SerialPortInfo>, serialport::Error> {
    ctx.call_cached(
        egui::Id::new("list_usb_ports"),
        communication::serial::list_all_usb_ports,
        SERIAL_PORT_REFRESH_INTERVAL,
    )
}

pub fn cached_first_stm32_port(ctx: &Context) -> Result<Option<SerialPortInfo>, serialport::Error> {
    ctx.call_cached(
        egui::Id::new("list_usb_ports"),
        communication::serial::find_first_stm32_port,
        SERIAL_PORT_REFRESH_INTERVAL,
    )
}
