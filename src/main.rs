mod mavlink;
mod ui;

use std::{
    num::NonZeroUsize,
    sync::{LazyLock, OnceLock},
};

use mavlink::{MessageBroker, ReflectionContext};
use parking_lot::Mutex;
use tokio::runtime::Runtime;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use ui::ComposableView;

static MSG_MANAGER: OnceLock<Mutex<MessageBroker>> = OnceLock::new();
static MAVLINK_PROFILE: LazyLock<ReflectionContext> = LazyLock::new(ReflectionContext::new);

static APP_NAME: &str = "segs";

fn main() -> Result<(), eframe::Error> {
    // set up logging (USE RUST_LOG=debug to see logs)
    let env_filter = EnvFilter::builder().from_env_lossy();
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(env_filter))
        .init();

    let rt = Runtime::new().expect("Unable to create Runtime");
    let _enter = rt.enter();

    let native_options = eframe::NativeOptions {
        // By modifying the viewport, we can change things like the windows size
        viewport: egui::ViewportBuilder::default()
            .with_min_inner_size((1000.0, 600.0))
            .with_title("Skyward Enhanced Ground Software"),
        ..Default::default()
    };

    // To create an app, eframe wants an `AppCreator`, which is a
    // Box<dyn FnOnce(&CreationContext<'_>) -> Result<Box<dyn App + 'app>, ...>
    //
    // CreationContext constains information useful to initilize our app, like storage.
    // Storage allows to store custom data in a way that persist whan you restart the app.
    eframe::run_native(
        APP_NAME, // This is the app id, used for example by Wayland
        native_options,
        Box::new(|ctx| {
            MSG_MANAGER
                .set(Mutex::new(MessageBroker::new(
                    NonZeroUsize::new(50).unwrap(),
                    ctx.egui_ctx.clone(),
                )))
                .expect("Unable to set MessageManager");
            let app = ctx
                .storage
                .map(|storage| ComposableView::new(APP_NAME, storage))
                .unwrap_or_default();
            Ok(Box::new(app))
        }),
    )
}
