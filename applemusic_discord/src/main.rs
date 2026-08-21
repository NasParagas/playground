use std::collections::HashMap;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use discord_presence::models::{ActivityType, DisplayType};
use discord_presence::{Client, Event};
use serde::Deserialize;

const DISCORD_APP_ID: u64 = 1525712821453717685;
const POLL_INTERVAL: Duration = Duration::from_secs(5);
const NOW_PLAYING_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/now_playing.js");
/// この秒数以上再生位置が飛んでいたらシークとみなして表示を更新する
const SEEK_THRESHOLD_SECS: u64 = 3;

/// now_playing.js の出力。停止中は {"playing": false} だけが返る
#[derive(Debug, Deserialize)]
struct NowPlaying {
    name: Option<String>,
    artist: Option<String>,
    album: Option<String>,
    position: Option<f64>,
    duration: Option<f64>,
}

#[derive(Debug)]
struct Track {
    name: String,
    artist: String,
    album: String,
    position: f64,
    duration: f64,
}

impl Track {
    fn key(&self) -> String {
        format!("{}\u{1}{}\u{1}{}", self.name, self.artist, self.album)
    }
}

/// Music.app から再生中の曲を JXA 経由で取得する。停止中・取得失敗は None
fn fetch_now_playing() -> Option<Track> {
    let output = Command::new("osascript")
        .args(["-l", "JavaScript", NOW_PLAYING_SCRIPT])
        .output()
        .ok()?;
    if !output.status.success() {
        eprintln!(
            "osascript failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        return None;
    }
    let np: NowPlaying = serde_json::from_slice(&output.stdout).ok()?;
    Some(Track {
        name: np.name?,
        artist: np.artist.unwrap_or_default(),
        album: np.album.unwrap_or_default(),
        position: np.position.unwrap_or(0.0),
        duration: np.duration?,
    })
}

#[derive(Deserialize)]
struct SearchResponse {
    results: Vec<SearchResult>,
}

#[derive(Deserialize)]
struct SearchResult {
    #[serde(rename = "artworkUrl100")]
    artwork_url: Option<String>,
}

/// iTunes Search API でアートワーク URL を引く。アルバム単位でキャッシュする
fn fetch_artwork(cache: &mut HashMap<String, Option<String>>, track: &Track) -> Option<String> {
    let key = format!("{}\u{1}{}", track.artist, track.album);
    if let Some(cached) = cache.get(&key) {
        return cached.clone();
    }
    let url = search_artwork(track);
    cache.insert(key, url.clone());
    url
}

fn search_artwork(track: &Track) -> Option<String> {
    let (term, entity) = if track.album.is_empty() {
        (format!("{} {}", track.name, track.artist), "song")
    } else {
        (format!("{} {}", track.artist, track.album), "album")
    };
    let response: SearchResponse = ureq::get("https://itunes.apple.com/search")
        .query("media", "music")
        .query("entity", entity)
        .query("limit", "1")
        .query("term", &term)
        .call()
        .ok()?
        .into_json()
        .ok()?;
    response
        .results
        .into_iter()
        .next()?
        .artwork_url
        .map(|url| url.replace("100x100bb", "512x512bb"))
}

fn main() {
    let mut drpc = Client::new(DISCORD_APP_ID);
    drpc.start();
    drpc.block_until_event(Event::Ready)
        .expect("Failed to connect to Discord");
    println!("Connected to Discord");

    let mut artwork_cache: HashMap<String, Option<String>> = HashMap::new();
    // Discord に表示中の曲 (key, 再生開始時刻)。シーク検出のため開始時刻も持つ
    let mut shown: Option<(String, u64)> = None;

    loop {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System clock before Unix epoch")
            .as_secs();

        match fetch_now_playing() {
            Some(track) => {
                let start = now.saturating_sub(track.position as u64);
                let end = start + track.duration as u64;
                let needs_update = match &shown {
                    Some((key, shown_start)) => {
                        *key != track.key() || shown_start.abs_diff(start) > SEEK_THRESHOLD_SECS
                    }
                    None => true,
                };
                if needs_update {
                    let artwork = fetch_artwork(&mut artwork_cache, &track);
                    // details はステータスの一行にもそのまま出るので「曲名 - アーティスト」にする
                    let details = format!("{} - {}", track.artist, track.name);
                    let state = if track.album.is_empty() {
                        &track.artist
                    } else {
                        &track.album
                    };
                    let result = drpc.set_activity(|act| {
                        act.activity_type(ActivityType::Listening)
                            .status_display(DisplayType::Details)
                            .details(&details)
                            .state(state)
                            .timestamps(|t| t.start(start).end(end))
                            .assets(|a| {
                                let a = if track.album.is_empty() {
                                    a
                                } else {
                                    a.large_text(&track.album)
                                };
                                match &artwork {
                                    Some(url) => a.large_image(url),
                                    None => a,
                                }
                            })
                    });
                    match result {
                        Ok(_) => {
                            println!("Now playing: {} — {}", track.name, track.artist);
                            shown = Some((track.key(), start));
                        }
                        Err(e) => eprintln!("Failed to set activity: {e}"),
                    }
                }
            }
            None => {
                if shown.is_some() {
                    match drpc.clear_activity() {
                        Ok(_) => println!("Cleared (paused / stopped)"),
                        Err(e) => eprintln!("Failed to clear activity: {e}"),
                    }
                    shown = None;
                }
            }
        }

        thread::sleep(POLL_INTERVAL);
    }
}
