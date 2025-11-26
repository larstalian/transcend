use anyhow::Result;

fn main() -> Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Transcend")
            .with_inner_size(egui::vec2(1000.0, 600.0)),
        ..Default::default()
    };

    if let Err(err) = eframe::run_native(
        "Transcend",
        native_options,
        Box::new(|cc| Ok(Box::new(transcend::gui::App::new(cc)))),
    ) {
        eprintln!("Failed to launch Transcend: {err}");
    }

    Ok(())
}
