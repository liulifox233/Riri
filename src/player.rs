use std::time::SystemTime;

use media_remote::{get_now_playing_info, InfoTypes, Number};

use crate::PlayInfo;

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Player {
    pub play_info: Option<PlayInfo>,
    pub playing: bool,
    pub offset: f64,
}

impl Default for Player {
    fn default() -> Self {
        let mut playing: bool = false;
        let play_info = get_now_playing_info().map(|info| {
            let name = info
                .get("kMRMediaRemoteNowPlayingInfoTitle")
                .unwrap_or(&InfoTypes::String("Unknown".to_string()))
                .to_string();
            let artist = info
                .get("kMRMediaRemoteNowPlayingInfoArtist")
                .unwrap_or(&InfoTypes::String("Unknown".to_string()))
                .to_string();
            let id = info
                .get("kMRMediaRemoteNowPlayingInfoiTunesStoreIdentifier")
                .and_then(|id| {
                    if let InfoTypes::Number(Number::Signed(id)) = id {
                        Some(*id)
                    } else {
                        None
                    }
                });
            let timestamp =
                info.get("kMRMediaRemoteNowPlayingInfoTimestamp")
                    .and_then(|timestamp| {
                        if let InfoTypes::SystemTime(timestamp) = timestamp {
                            Some(*timestamp)
                        } else {
                            None
                        }
                    });

            playing = matches!(info
                 .get("kMRMediaRemoteNowPlayingInfoPlaybackRate")
                 .and_then(|rate| {
                     if let InfoTypes::Number(Number::Floating(rate)) = rate {
                         Some(*rate)
                     } else {
                         None
                     }
                 }), Some(rate) if rate > 0.0);

            PlayInfo {
                name,
                artist,
                id,
                timestamp: timestamp.unwrap_or(SystemTime::now()),
                elapsed_time: 0.0,
            }
        });

        Self {
            play_info,
            playing,
            offset: 0.0,
        }
    }
}
