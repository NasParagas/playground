use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

#[derive(Debug)]
pub struct VideoInfo {
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub duration_sec: f64,
    pub size_bytes: u64,
}

#[derive(Deserialize)]
struct FfprobeOutput {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Deserialize)]
struct Stream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Deserialize)]
struct Format {
    duration: Option<String>,
    size: Option<String>,
}

pub fn probe(input: &Path) -> Result<VideoInfo> {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-print_format", "json", "-show_streams", "-show_format"])
        .arg(input)
        .output()
        .context("ffprobeの実行に失敗しました。インストールされていますか?")?;

    if !output.status.success() {
        bail!(
            "ffprobeがエラーを返しました:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let parsed: FfprobeOutput =
        serde_json::from_slice(&output.stdout).context("ffprobe出力のパースに失敗しました")?;

    let video = parsed
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .context("映像ストリームが見つかりません")?;

    Ok(VideoInfo {
        codec: video.codec_name.clone().unwrap_or_default(),
        width: video.width.context("解像度(width)を取得できません")?,
        height: video.height.context("解像度(height)を取得できません")?,
        duration_sec: parsed
            .format
            .duration
            .as_deref()
            .and_then(|d| d.parse().ok())
            .context("再生時間を取得できません")?,
        size_bytes: parsed
            .format
            .size
            .as_deref()
            .and_then(|s| s.parse().ok())
            .context("ファイルサイズを取得できません")?,
    })
}
