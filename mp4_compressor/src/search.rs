use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::encode::{self, Encoder};
use crate::probe::VideoInfo;
use crate::vmaf;

const SAMPLE_LEN_SEC: f64 = 20.0;
const SAMPLE_COUNT: usize = 3;

/// 目標VMAFを満たす範囲で最も圧縮が効く品質設定を、サンプル区間の二分探索で決める。
///
/// エンコーダごとの品質パラメータを「大きいほど圧縮が効く(品質が下がる)」向きの
/// 整数pに正規化して扱う。VMAFはpに対して単調に下がるので、目標以上を満たす
/// 最大のpを二分探索すればよい。判定には各サンプルのスコアのうち最悪値を使い、
/// 「静かな区間だけ見て本編の激しい区間で品質が足りない」事故を避ける。
pub fn find_encoder(
    input: &Path,
    target: f64,
    base: &Encoder,
    info: &VideoInfo,
) -> Result<Encoder> {
    let workdir = std::env::temp_dir().join(format!("mp4c_search_{}", std::process::id()));
    fs::create_dir_all(&workdir).context("探索用の一時ディレクトリを作成できません")?;
    let result = run(input, target, base, info, &workdir);
    let _ = fs::remove_dir_all(&workdir);
    result
}

/// 正規化パラメータpの探索範囲。この外側は「明らかにやりすぎ/意味がない」領域
fn param_range(base: &Encoder) -> (u8, u8) {
    match base {
        Encoder::X265 { .. } => (18, 30),        // crf 18〜30
        Encoder::VideoToolbox { .. } => (15, 55), // qv 85〜45 (下記の反転を参照)
        Encoder::Nvenc { .. } => (20, 38),        // cq 20〜38
    }
}

/// 正規化パラメータp → 実際のエンコーダ設定
fn encoder_at(base: &Encoder, p: u8) -> Encoder {
    match base {
        Encoder::X265 { preset, .. } => Encoder::X265 {
            crf: p,
            preset: preset.clone(),
        },
        // qvだけ「大きいほど高品質」で向きが逆なので反転する (p=15..=55 ↔ qv=85..=45)
        Encoder::VideoToolbox { .. } => Encoder::VideoToolbox { quality: 100 - p },
        Encoder::Nvenc { .. } => Encoder::Nvenc { cq: p },
    }
}

fn run(
    input: &Path,
    target: f64,
    base: &Encoder,
    info: &VideoInfo,
    workdir: &Path,
) -> Result<Encoder> {
    let samples = extract_samples(input, info, workdir)?;
    if samples.len() == 1 && samples[0] == input {
        println!("短い動画のため全体を使って品質設定を探索します (目標VMAF {target})");
    } else {
        println!(
            "サンプル{}本 (各{}秒) で品質設定を探索します (目標VMAF {target})",
            samples.len(),
            SAMPLE_LEN_SEC
        );
    }

    let (mut lo, mut hi) = param_range(base);
    let mut best: Option<(Encoder, f64)> = None;
    while lo <= hi {
        let p = (lo + hi) / 2;
        let candidate = encoder_at(base, p);
        let score = score_at(&samples, &candidate, p, workdir)?;
        println!("  {}: VMAF {score:.2} (最悪サンプル)", candidate.describe());
        if score >= target {
            best = Some((candidate, score));
            lo = p + 1; // 品質に余裕あり → もっと圧縮が効く側を試す
        } else {
            hi = p - 1;
        }
    }

    match best {
        Some((encoder, score)) => {
            println!("→ {} に決定 (サンプルでのVMAF {score:.2})", encoder.describe());
            Ok(encoder)
        }
        None => bail!(
            "最も高品質側の設定でも目標VMAF {target} に届きません。圧縮が難しい映像です。\
             --target-vmaf を下げるか、--crf / --qv / --cq で直接指定してください"
        ),
    }
}

/// 動画から冒頭・末尾を避けて等間隔にサンプル区間を切り出す(再エンコードなし)。
/// サンプリングする意味がないほど短い動画は、入力自体を1本のサンプルとして返す。
fn extract_samples(input: &Path, info: &VideoInfo, workdir: &Path) -> Result<Vec<PathBuf>> {
    if info.duration_sec <= SAMPLE_LEN_SEC * SAMPLE_COUNT as f64 * 2.0 {
        return Ok(vec![input.to_path_buf()]);
    }

    let mut samples = Vec::new();
    for i in 0..SAMPLE_COUNT {
        // サンプル中心を 1/6, 3/6, 5/6 の位置に置く
        let center = info.duration_sec * (2 * i + 1) as f64 / (SAMPLE_COUNT * 2) as f64;
        let start = center - SAMPLE_LEN_SEC / 2.0;
        let out = workdir.join(format!("sample{i}.mp4"));

        let status = Command::new("ffmpeg")
            .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"])
            .args(["-ss", &format!("{start:.2}")])
            .arg("-i")
            .arg(input)
            .args(["-t", &SAMPLE_LEN_SEC.to_string()])
            // ストリームコピーなので高速。キーフレーム境界に丸められるが、
            // サンプル自身を基準にVMAFを測るためズレは問題にならない
            .args(["-map", "0:v:0", "-c", "copy"])
            .arg(&out)
            .status()
            .context("ffmpegの起動に失敗しました")?;
        if !status.success() {
            bail!("サンプル区間の抽出に失敗しました");
        }
        samples.push(out);
    }
    Ok(samples)
}

/// 候補設定で全サンプルをエンコードし、VMAF(mean)の最悪値を返す
fn score_at(samples: &[PathBuf], candidate: &Encoder, p: u8, workdir: &Path) -> Result<f64> {
    let mut worst = f64::MAX;
    for (i, sample) in samples.iter().enumerate() {
        let out = workdir.join(format!("s{i}_p{p}.mp4"));
        encode::encode(sample, &out, candidate)?;
        let score = vmaf::measure(sample, &out)?;
        worst = worst.min(score.mean);
    }
    Ok(worst)
}
