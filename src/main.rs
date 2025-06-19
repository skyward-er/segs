#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]

mod communication;
mod error;
mod mavlink;
mod message_broker;
mod ui;
mod utils;

use std::{fs::create_dir_all, sync::LazyLock, time::Instant};

use error::ErrInstrument;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use mavlink::reflection::ReflectionContext;
use ui::App;

static APP_START_TIMESTAMP_ORIGIN: LazyLock<Instant> = LazyLock::new(Instant::now);

static APP_NAME: &str = "segs";

fn main() -> Result<(), eframe::Error> {
    // Create the logs directory if it doesn't exist and add to the registry
    let mut _guard = None;
    let file_layer = if let Some(base_dirs) = directories::BaseDirs::new() {
        let proj_dir = base_dirs.data_local_dir().join(APP_NAME);
        let logs_dir = proj_dir.join("logs");
        create_dir_all(&logs_dir).log_expect("Failed to create logs directory");

        let file_appender = tracing_appender::rolling::daily(&logs_dir, "segs.log");
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        _guard = Some(guard); // Keep guard alive to flush logs
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_writer(non_blocking)
                .with_filter(LevelFilter::DEBUG),
        )
    } else {
        None
    };

    // Set up logging (USE RUST_LOG=debug to see logs)
    let env_filter = EnvFilter::builder().from_env_lossy();

    // Initialize the logger
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(file_layer)
        .init();

    let native_options = eframe::NativeOptions {
        // By modifying the viewport, we can change things like the windows size
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((1000.0, 600.0))
            .with_title("Skyward Enhanced Ground Software"),
        ..Default::default()
    };

    // Initialize the starting timestamp
    let starting_time = &APP_START_TIMESTAMP_ORIGIN;
    tracing::info!("Starting {} at {:?}", APP_NAME, starting_time);

    // CreationContext constains information useful to initilize our app, like storage.
    // Storage allows to store custom data in a way that persist whan you restart the app.
    eframe::run_native(
        APP_NAME, // This is the app id, used for example by Wayland
        native_options,
        Box::new(|ctx| Ok(Box::new(App::new(ctx)))),
    )
}
