use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    transcend::tui::run().await
}
