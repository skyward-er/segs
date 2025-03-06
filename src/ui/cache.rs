//! Module for caching expensive UI calls using egui's temporary memory storage.
//! It provides utilities for caching the results of functions to avoid frequent recalculations.

use std::time::{Duration, Instant};

use egui::Context;
use serialport::SerialPortInfo;

use crate::{communication, error::ErrInstrument};

const SERIAL_PORT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

/// Internal helper function that caches the result of a given function call for a specified duration.
///
/// # Arguments
/// * `ctx` - The egui context used for caching.
/// * `id` - The unique identifier for the cached item.
/// * `fun` - The function whose return value is to be cached.
/// * `expiration_duration` - The duration after which the cache should be refreshed.
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

/// A trait to extend egui's Context with a caching function.
pub trait CacheCall {
    /// Calls the provided function and caches its result.
    ///
    /// # Arguments
    /// * `id` - A unique identifier for the cached value.
    /// * `fun` - The function to be cached.
    /// * `expiration_duration` - The cache expiration duration.
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static;
}

impl CacheCall for egui::Context {
    /// Implements the caching call using the internal `call` function.
    fn call_cached<F, T>(&self, id: egui::Id, fun: F, expiration_duration: Duration) -> T
    where
        F: Fn() -> T,
        T: Clone + Send + Sync + 'static,
    {
        call(self, id, fun, expiration_duration)
    }
}

/// Returns a cached list of all available USB ports.
///
/// # Arguments
/// * `ctx` - The egui context used for caching.
///
/// # Returns
/// * A Result containing a vector of `SerialPortInfo` or a `serialport::Error`.
pub fn cached_list_all_usb_ports(ctx: &Context) -> Result<Vec<SerialPortInfo>, serialport::Error> {
    ctx.call_cached(
        egui::Id::new("list_usb_ports"),
        communication::serial::list_all_usb_ports,
        SERIAL_PORT_REFRESH_INTERVAL,
    )
}

/// Returns the first cached STM32 port found, if any.
///
/// # Arguments
/// * `ctx` - The egui context used for caching.
///
/// # Returns
/// * A Result containing an Option of `SerialPortInfo` or a `serialport::Error`.
pub fn cached_first_stm32_port(ctx: &Context) -> Result<Option<SerialPortInfo>, serialport::Error> {
    ctx.call_cached(
        egui::Id::new("list_usb_ports"),
        communication::serial::find_first_stm32_port,
        SERIAL_PORT_REFRESH_INTERVAL,
    )
}
