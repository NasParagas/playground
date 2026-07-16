use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

#[derive(Debug)]
pub struct VmafScore {
    pub mean: f64,
    pub min: f64,
}

/// 圧縮後の動画を元動画と比較してVMAFスコアを測定する。
/// libvmafフィルタは第1入力=劣化版、第2入力=リファレンスの順で受け取る。
pub fn measure(reference: &Path, distorted: &Path) -> Result<VmafScore> {
    let log_path = std::env::temp_dir().join(format!("mp4c_vmaf_{}.json", std::process::id()));
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());

    let filter = format!(
        "libvmaf=log_fmt=json:log_path={}:n_threads={threads}",
        log_path.display()
    );

    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "warning", "-stats", "-nostdin"])
        .arg("-i")
        .arg(distorted)
        .arg("-i")
        .arg(reference)
        .args(["-lavfi", &filter, "-f", "null", "-"])
        .output()
        .context("ffmpeg(libvmaf)の起動に失敗しました")?;

    if !output.status.success() {
        bail!(
            "VMAF測定が失敗しました:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let log = std::fs::read_to_string(&log_path).context("VMAFログの読み込みに失敗しました")?;
    let _ = std::fs::remove_file(&log_path);

    let json: Value = serde_json::from_str(&log).context("VMAFログのパースに失敗しました")?;
    let vmaf = &json["pooled_metrics"]["vmaf"];
    let (Some(mean), Some(min)) = (vmaf["mean"].as_f64(), vmaf["min"].as_f64()) else {
        bail!("VMAFログにスコアが見つかりません");
    };

    Ok(VmafScore { mean, min })
}
