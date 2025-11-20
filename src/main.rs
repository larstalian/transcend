use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    transcend::tui::run().await
}

// #[cfg(feature = "gui")]
// fn main() -> eframe::Result<()> {
//     // later: transcend::gui::run();
//     unimplemented!()
// }
