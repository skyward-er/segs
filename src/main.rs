#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]

mod communication;
mod error;
mod mavlink;
mod message_broker;
mod ui;
mod utils;

use std::sync::LazyLock;

use tokio::runtime::Runtime;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

use error::ErrInstrument;
use mavlink::ReflectionContext;
use ui::App;

/// ReflectionContext singleton, used to get access to the Mavlink message definitions
static MAVLINK_PROFILE: LazyLock<ReflectionContext> = LazyLock::new(ReflectionContext::new);

static APP_NAME: &str = "segs";

fn main() -> Result<(), eframe::Error> {
    // Set up logging (USE RUST_LOG=debug to see logs)
    let env_filter = EnvFilter::builder().from_env_lossy();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .with(tracing_tracy::TracyLayer::default())
        .init();

    // Start Tokio runtime (TODO: decide whether to use Tokio or a simpler thread-based approach)
    let rt = Runtime::new().log_expect("Unable to create Tokio Runtime");
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        // By modifying the viewport, we can change things like the windows size
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((1000.0, 600.0))
            .with_title("Skyward Enhanced Ground Software"),
        ..Default::default()
    };

    // CreationContext constains information useful to initilize our app, like storage.
    // Storage allows to store custom data in a way that persist whan you restart the app.
    eframe::run_native(
        APP_NAME, // This is the app id, used for example by Wayland
        native_options,
        Box::new(|ctx| Ok(Box::new(App::new(APP_NAME, ctx)))),
    )
}
