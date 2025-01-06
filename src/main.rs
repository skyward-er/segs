#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]
#![warn(clippy::panic)]

mod error;
mod mavlink;
mod ui;

use std::{
    num::NonZeroUsize,
    sync::{LazyLock, OnceLock},
};

use parking_lot::Mutex;
use tokio::runtime::Runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

use error::ErrInstrument;
use mavlink::{MessageBroker, ReflectionContext};
use ui::ComposableView;

/// MessageBroker singleton, used to fetch & filter Mavlink messages collected
static MSG_MANAGER: OnceLock<Mutex<MessageBroker>> = OnceLock::new();
/// ReflectionContext singleton, used to get access to the Mavlink message definitions
static MAVLINK_PROFILE: LazyLock<ReflectionContext> = LazyLock::new(ReflectionContext::new);

static APP_NAME: &str = "segs";

#[macro_export]
macro_rules! msg_broker {
    () => {
        $crate::MSG_MANAGER
            .get()
            .log_expect("Unable to get MessageBroker")
            .lock()
    };
}

fn main() -> Result<(), eframe::Error> {
    // Set up logging (USE RUST_LOG=debug to see logs)
    let env_filter = EnvFilter::builder().from_env_lossy();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
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
        Box::new(|ctx| {
            // First we initialize the MSGManager, as a global singleton available to all the panes
            MSG_MANAGER
                .set(Mutex::new(MessageBroker::new(
                    // FIXME: Choose where to put the channel size of the MessageBroker
                    NonZeroUsize::new(50).log_unwrap(),
                    ctx.egui_ctx.clone(),
                )))
                .log_expect("Unable to set MessageManager");
            egui_extras::install_image_loaders(&ctx.egui_ctx);
            let app = ctx
                .storage
                .map(|storage| ComposableView::new(APP_NAME, storage))
                .unwrap_or_default();
            Ok(Box::new(app))
        }),
    )
}
