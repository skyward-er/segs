use ui::ComposableView;

mod ui;

fn main() -> Result<(), eframe::Error> {
    // set up logging (USE RUST_LOG=debug to see logs)
    env_logger::init();

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
        "segs", // This is the app id, used for example by Wayland
        native_options,
        Box::new(|_| Ok(Box::<ComposableView>::default())),
    )
}
