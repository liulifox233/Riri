mod lyrics;
mod models;
mod riri;

use anyhow::Result;
use system_status_bar_macos::*;
use tokio::*;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    let config_dir = dirs::config_dir().unwrap().join("Riri");

    if !config_dir.exists() {
        std::fs::create_dir(&config_dir)?
    }

    spawn(async_infinite_event_loop(time::sleep));

    let (tx, mut rx) = tokio::sync::mpsc::channel(8);

    tokio::task::spawn({
        let riri = riri::Riri::new(config_dir.join("config.yml")).await?;
        riri.run(tx)
    });

    info!("Riri is running...");

    let mut status_item = StatusItem::new(
        "🎵",
        Menu::new(vec![
            MenuItem::new(
                "Play/Pause",
                Some(Box::new(|| {
                    let _ = apple_music::AppleMusic::playpause();
                })),
                None,
            ),
            MenuItem::new("Quit", Some(Box::new(|| std::process::exit(0))), None),
        ]),
    );

    loop {
        if let Some(title) = rx.recv().await {
            status_item.set_title(&title);
        }
    }
}
