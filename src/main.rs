#![forbid(unsafe_code)]

mod app;
mod solver;

use std::time::Duration;

use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().expect("failed to make new Tokio Runtime");

    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    // Run the GUI in the main thread.
    eframe::run_native(
        "Laser Mazer",
        eframe::NativeOptions {
            initial_window_size: Some(eframe::egui::vec2(1600., 900.)),
            ..Default::default()
        },
        Box::new(|_cc| Box::<app::MyApp>::default()),
    )
    .expect("Failed to launch app");
}
