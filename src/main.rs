mod lyrics;
mod models;
mod player;
mod riri;
mod utils;

use anyhow::Result;
use media_remote::{
    add_observer, get_now_playing_info, register_for_now_playing_notifications, InfoTypes, Number,
};
use std::time::SystemTime;
use system_status_bar_macos::*;
use tokio::time::sleep;
use tokio::*;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();
    let config_dir = dirs::config_dir().unwrap().join("Riri");
    let data_dir = dirs::data_local_dir().unwrap().join("Riri").join("Data");
    if !config_dir.exists() {
        std::fs::create_dir(&config_dir)?;
    }
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)?;
    }

    spawn(async_infinite_event_loop(sleep));

    let (lyrics_tx, mut lyrics_rx) = tokio::sync::mpsc::channel(8);
    let media_event_rx = register_observer().unwrap();
    let mut riri = riri::Riri::new(config_dir.join("config.yml")).await?;

    tokio::spawn(async move {
        riri.run(lyrics_tx, media_event_rx).await.unwrap();
    });

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

    while let Some(title) = lyrics_rx.recv().await {
        status_item.set_title(&title);
    }

    Ok(())
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
enum MediaEvent {
    Paused,
    Playing(PlayInfo),
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
struct PlayInfo {
    name: String,
    artist: String,
    id: Option<i64>,
    timestamp: SystemTime,
    elapsed_time: f64,
}

fn register_observer() -> Result<tokio::sync::mpsc::Receiver<MediaEvent>> {
    let (media_event_tx, media_event_rx) = tokio::sync::mpsc::channel(100);
    register_for_now_playing_notifications();
    let observer = Box::new(add_observer(
        media_remote::Notification::NowPlayingInfoDidChange,
        move || {
            if let Some(play_info) = get_now_playing_info() {
                let title = play_info
                    .get("kMRMediaRemoteNowPlayingInfoTitle")
                    .unwrap_or(&InfoTypes::String("Unknown".to_string()))
                    .to_string();
                let artist = play_info
                    .get("kMRMediaRemoteNowPlayingInfoArtist")
                    .unwrap_or(&InfoTypes::String("Unknown".to_string()))
                    .to_string();
                let id = play_info
                    .get("kMRMediaRemoteNowPlayingInfoiTunesStoreIdentifier")
                    .and_then(|id| {
                        if let InfoTypes::Number(Number::Signed(id)) = id {
                            Some(*id)
                        } else {
                            None
                        }
                    });
                let timestamp = play_info
                    .get("kMRMediaRemoteNowPlayingInfoTimestamp")
                    .and_then(|timestamp| {
                        if let InfoTypes::SystemTime(timestamp) = timestamp {
                            Some(*timestamp)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(SystemTime::now());
                let elapsed_time = play_info
                    .get("kMRMediaRemoteNowPlayingInfoElapsedTime")
                    .and_then(|elapsed_time| {
                        if let InfoTypes::Number(Number::Floating(elapsed_time)) = elapsed_time {
                            Some(*elapsed_time)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0.0);
                match play_info
                    .get("kMRMediaRemoteNowPlayingInfoPlaybackRate")
                    .and_then(|rate| {
                        if let InfoTypes::Number(Number::Floating(rate)) = rate {
                            Some(*rate)
                        } else {
                            None
                        }
                    }) {
                    Some(rate) if rate > 0.0 => media_event_tx
                        .try_send(MediaEvent::Playing(PlayInfo {
                            name: title.to_string(),
                            artist: artist.to_string(),
                            id,
                            timestamp,
                            elapsed_time,
                        }))
                        .unwrap(),
                    _ => {
                        media_event_tx.try_send(MediaEvent::Paused).unwrap();
                    }
                }
            };
        },
    ));
    Box::leak(observer);
    Ok(media_event_rx)
}
