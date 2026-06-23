mod gui;
mod vpn;
mod config;
mod cert;
mod system;
mod utils;
mod errors;

use log::info;

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Starting WIKEv2 Connect");

    // Multi-thread Tokio runtime kept alive for the entire app lifetime.
    // rt.enter() makes tokio::spawn() work from inside eframe's event loop.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async {
        if let Err(e) = system::prerequisites::check_and_setup().await {
            log::warn!("Prerequisites check skipped: {}", e);
        }
    });

    let _guard = rt.enter();

    if let Err(e) = gui::launch() {
        eprintln!("GUI error: {e}");
        std::process::exit(1);
    }
}
