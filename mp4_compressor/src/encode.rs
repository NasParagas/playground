use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

#[derive(Clone)]
pub enum Encoder {
    /// ソフトウェアエンコード。遅いが同品質で最小サイズ
    X265 { crf: u8, preset: String },
    /// Apple Siliconのメディアエンジンによるハードウェアエンコード。
    /// 高速・低発熱だが同品質でサイズは2〜5割大きくなる
    VideoToolbox { quality: u8 },
    /// NVIDIA GPUのハードウェアエンコーダ。位置づけはVideoToolboxと同様
    Nvenc { cq: u8 },
}

impl Encoder {
    pub fn describe(&self) -> String {
        match self {
            Self::X265 { crf, preset } => format!("libx265 crf={crf} preset={preset}"),
            Self::VideoToolbox { quality } => format!("hevc_videotoolbox q={quality}"),
            Self::Nvenc { cq } => format!("hevc_nvenc cq={cq}"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum HwKind {
    VideoToolbox,
    Nvenc,
}

/// このffmpegで使えるHEVCハードウェアエンコーダを実行時に検出する。
/// コンパイル時のOS分岐ではなく実行時検出にしているのは、
/// 「LinuxだがffmpegにNVENCが入っていない」等のケースを正しく弾くため。
pub fn detect_hw_encoder() -> Result<Option<HwKind>> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .context("ffmpegの起動に失敗しました")?;
    let encoders = String::from_utf8_lossy(&output.stdout);

    if encoders.contains("hevc_videotoolbox") {
        Ok(Some(HwKind::VideoToolbox))
    } else if encoders.contains("hevc_nvenc") {
        Ok(Some(HwKind::Nvenc))
    } else {
        Ok(None)
    }
}

/// HEVCへの再エンコードを行う。音声・メタデータは無劣化でコピー。
pub fn encode(input: &Path, output: &Path, encoder: &Encoder) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-loglevel", "warning", "-stats", "-nostdin"])
        .arg("-i")
        .arg(input)
        // 映像1本 + 音声全部(あれば)。mp4に入らないデータ系ストリームは拾わない
        .args(["-map", "0:v:0", "-map", "0:a?"]);

    match encoder {
        Encoder::X265 { crf, preset } => {
            cmd.args(["-c:v", "libx265", "-preset", preset])
                .args(["-crf", &crf.to_string()])
                .args(["-x265-params", "log-level=error"]);
        }
        Encoder::VideoToolbox { quality } => {
            cmd.args(["-c:v", "hevc_videotoolbox"])
                .args(["-q:v", &quality.to_string()]);
        }
        Encoder::Nvenc { cq } => {
            // -rc vbr + -cq + -b:v 0 で品質一定モード(CRF相当)になる
            cmd.args(["-c:v", "hevc_nvenc", "-preset", "p5"])
                .args(["-rc", "vbr", "-cq", &cq.to_string(), "-b:v", "0"]);
        }
    }

    let status = cmd
        // hvc1タグがないとQuickTime/iPhoneで再生できない
        .args(["-tag:v", "hvc1"])
        .args(["-c:a", "copy"])
        .args(["-map_metadata", "0", "-movflags", "+faststart"])
        .arg(output)
        .status()
        .context("ffmpegの起動に失敗しました")?;

    if !status.success() {
        bail!("ffmpegのエンコードが失敗しました (exit: {status})");
    }
    Ok(())
}
