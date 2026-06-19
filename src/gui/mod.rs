pub mod app;
pub mod theme;
pub mod operations;

use crate::errors::WIKEv2ConnectError;

pub fn launch() -> Result<(), WIKEv2ConnectError> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("WIKEv2 Connect — VPN Manager")
        .with_inner_size([860.0, 580.0])
        .with_min_inner_size([640.0, 420.0])
        .with_active(true);  // richiede attivazione immediata su Wayland (fix "primo click ignorato")

    // Load app icon (embedded at compile time)
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(std::sync::Arc::new(icon));
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    // Il primo argomento diventa il Wayland app_id: deve coincidere con il nome
    // del file .desktop (senza estensione) affinché KDE abbini icona e titlebar.
    eframe::run_native(
        "wikev2connect",
        native_options,
        Box::new(|cc| Ok(Box::new(app::Wikev2App::new(cc)))),
    )
    .map_err(|e| WIKEv2ConnectError::GuiError(e.to_string()))
}

fn load_icon() -> Option<egui::IconData> {
    let bytes = include_bytes!("../../resources/icona.png");
    let img = image::load_from_memory(bytes).ok()?.into_rgba8();
    let (width, height) = img.dimensions();
    Some(egui::IconData { rgba: img.into_raw(), width, height })
}
