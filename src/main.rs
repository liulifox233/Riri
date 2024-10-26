mod lyrics;
mod models;
mod riri;

use anyhow::Result;
use system_status_bar_macos::*;
use tokio::*;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config_dir = dirs::config_dir().unwrap().join("Riri");
    if !config_dir.exists() { std::fs::create_dir(&config_dir)? }

    let riri = riri::Riri::new(config_dir.join("config.yml")).await?;

    spawn(async_infinite_event_loop(time::sleep));

    riri.run().await
}
