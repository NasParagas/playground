mod encode;
mod probe;
mod search;
mod vmaf;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Parser;

/// mp4を品質(VMAF)を確認しながらHEVCに圧縮する
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// 入力mp4ファイル、またはディレクトリ(直下の*.mp4を一括処理)。複数指定可
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// 出力パス。入力が1ファイルのときのみ指定可 (既定: <input>.compressed.mp4)
    #[arg(short, long, conflicts_with = "out_dir")]
    output: Option<PathBuf>,

    /// 出力先ディレクトリ。元と同じファイル名で出力する
    #[arg(long)]
    out_dir: Option<PathBuf>,

    /// x265のCRF値。小さいほど高品質・大サイズ (目安: 20〜26)
    #[arg(long, default_value_t = 22)]
    crf: u8,

    /// 目標VMAF (例: 95)。サンプル区間で目標を満たす最も圧縮の効く品質設定を
    /// 二分探索して自動決定する。--fastとの併用可、--crf/--qv/--cqとは排他
    #[arg(long, conflicts_with_all = ["crf", "qv", "cq"])]
    target_vmaf: Option<f64>,

    /// x265のpreset (速度と圧縮効率のトレードオフ)
    #[arg(long, default_value = "slow")]
    preset: String,

    /// ハードウェアエンコーダを使う(VideoToolbox → NVENC の順で自動検出)。
    /// 数倍高速だが同品質でサイズは大きめ
    #[arg(long)]
    fast: bool,

    /// --fast(VideoToolbox)時の品質。0-100で大きいほど高品質・大サイズ (目安: 55〜75)
    #[arg(long, default_value_t = 65, value_parser = clap::value_parser!(u8).range(..=100))]
    qv: u8,

    /// --fast(NVENC)時の品質。0-51で小さいほど高品質・大サイズ (目安: 22〜28)
    #[arg(long, default_value_t = 24, value_parser = clap::value_parser!(u8).range(..=51))]
    cq: u8,

    /// エンコード後のVMAF測定をスキップする
    #[arg(long)]
    no_vmaf: bool,
}

enum Outcome {
    Compressed {
        in_size: u64,
        out_size: u64,
        vmaf: Option<f64>,
    },
    /// 元より大きくなったため出力を破棄した
    NoGain,
}

fn main() -> Result<()> {
    let args = Args::parse();

    check_ffmpeg()?;

    if let Some(target) = args.target_vmaf
        && !(0.0..=100.0).contains(&target)
    {
        bail!("--target-vmaf は0〜100で指定してください (推奨: 93〜97)");
    }

    let encoder = if args.fast {
        match encode::detect_hw_encoder()? {
            Some(encode::HwKind::VideoToolbox) => {
                encode::Encoder::VideoToolbox { quality: args.qv }
            }
            Some(encode::HwKind::Nvenc) => encode::Encoder::Nvenc { cq: args.cq },
            None => bail!(
                "このffmpegで使えるHEVCハードウェアエンコーダ(videotoolbox / nvenc)が\
                 見つかりません。--fast なしで実行してください"
            ),
        }
    } else {
        encode::Encoder::X265 {
            crf: args.crf,
            preset: args.preset.clone(),
        }
    };

    let inputs = collect_inputs(&args.inputs)?;
    if inputs.len() > 1 && args.output.is_some() {
        bail!("-o/--output は入力が1ファイルのときだけ使えます。複数入力には --out-dir を");
    }
    if let Some(dir) = &args.out_dir {
        fs::create_dir_all(dir)
            .with_context(|| format!("出力ディレクトリを作成できません: {}", dir.display()))?;
    }

    let total = inputs.len();
    let mut results: Vec<(PathBuf, Result<Outcome>)> = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        if total > 1 {
            println!("\n===== [{}/{}] {} =====", i + 1, total, input.display());
        }
        let output = resolve_output(input, &args);
        let result = process_one(input, &output, &encoder, &args);
        // バッチでは1件の失敗で全体を止めず、記録して次へ進む
        if let Err(e) = &result {
            eprintln!("エラー: {e:#}");
        }
        results.push((input.clone(), result));
    }

    if total > 1 {
        print_summary(&results);
    }

    if results.iter().any(|(_, r)| r.is_err()) {
        std::process::exit(1);
    }
    Ok(())
}

/// 入力引数を処理対象ファイルの一覧に展開する。
/// ディレクトリは直下の*.mp4に展開する(過去の出力 *.compressed.mp4 は除く)。
fn collect_inputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let is_mp4 = |p: &Path| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("mp4"));
    let is_own_output = |p: &Path| {
        p.file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".compressed.mp4"))
    };

    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let mut found: Vec<PathBuf> = fs::read_dir(path)
                .with_context(|| format!("ディレクトリを読めません: {}", path.display()))?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && is_mp4(p) && !is_own_output(p))
                .collect();
            found.sort();
            if found.is_empty() {
                println!("注意: {} の直下にmp4がありません", path.display());
            }
            files.extend(found);
        } else if path.is_file() {
            files.push(path.clone());
        } else {
            bail!("入力が見つかりません: {}", path.display());
        }
    }

    if files.is_empty() {
        bail!("処理対象のmp4がありません");
    }
    Ok(files)
}

fn resolve_output(input: &Path, args: &Args) -> PathBuf {
    if let Some(output) = &args.output {
        output.clone()
    } else if let Some(dir) = &args.out_dir {
        dir.join(input.file_name().unwrap_or_default())
    } else {
        input.with_extension("compressed.mp4")
    }
}

fn process_one(
    input: &Path,
    output: &Path,
    base_encoder: &encode::Encoder,
    args: &Args,
) -> Result<Outcome> {
    if output.exists() {
        bail!("出力先が既に存在します(上書きしません): {}", output.display());
    }

    let info = probe::probe(input)?;
    println!(
        "入力: {} ({}x{}, {}, {:.1}分, {})",
        input.display(),
        info.width,
        info.height,
        info.codec,
        info.duration_sec / 60.0,
        human_size(info.size_bytes),
    );

    if matches!(info.codec.as_str(), "hevc" | "av1") {
        println!(
            "注意: 既に{}でエンコード済みです。再圧縮しても縮まないか、劣化だけする可能性があります。",
            info.codec
        );
    }

    // --target-vmaf 指定時は、この動画に合った品質設定をサンプル探索で決める
    // (最適な設定は映像の内容次第でファイルごとに違うため、バッチでも毎回探索する)
    let encoder = if let Some(target) = args.target_vmaf {
        println!();
        search::find_encoder(input, target, base_encoder, &info)?
    } else {
        base_encoder.clone()
    };

    println!("\nエンコード開始: {}", encoder.describe());
    encode::encode(input, output, &encoder)?;

    let out_size = fs::metadata(output)
        .context("出力ファイルを確認できません")?
        .len();

    println!("\n--- 結果 ---");
    println!("元:     {}", human_size(info.size_bytes));
    println!(
        "圧縮後: {} ({:.1}%に削減)",
        human_size(out_size),
        out_size as f64 / info.size_bytes as f64 * 100.0
    );

    if out_size >= info.size_bytes {
        println!("\n元のファイルより大きくなりました。この動画は再圧縮に向いていません。");
        println!("出力を削除します: {}", output.display());
        fs::remove_file(output)?;
        return Ok(Outcome::NoGain);
    }

    let mut vmaf_mean = None;
    if !args.no_vmaf {
        println!("\nVMAF測定中(元動画との品質比較)...");
        let score = vmaf::measure(input, output)?;
        println!("VMAF: mean={:.2} min={:.2}", score.mean, score.min);
        if let Some(target) = args.target_vmaf {
            if score.mean >= target {
                println!("→ 全体でも目標VMAF {target} を満たしています。");
            } else {
                println!(
                    "→ 全体のVMAFが目標 {target} をわずかに下回りました。\
                     サンプル区間が本編より圧縮しやすかった可能性があります。"
                );
            }
        } else {
            // 「もっと縮める/品質を上げる」ときに次に試すオプションの提案。
            // 高品質の向きがCRF/cq(小さいほど)とqv(大きいほど)で逆な点に注意
            let (smaller, better) = match &encoder {
                encode::Encoder::X265 { crf, .. } => (
                    format!("--crf {}", crf + 2),
                    format!("--crf {}", crf.saturating_sub(2)),
                ),
                encode::Encoder::VideoToolbox { quality } => (
                    format!("--fast --qv {}", quality.saturating_sub(5)),
                    format!("--fast --qv {}", (quality + 5).min(100)),
                ),
                encode::Encoder::Nvenc { cq } => (
                    format!("--fast --cq {}", (cq + 2).min(51)),
                    format!("--fast --cq {}", cq.saturating_sub(2)),
                ),
            };
            println!("{}", interpret_vmaf(score.mean, &smaller, &better));
        }
        vmaf_mean = Some(score.mean);
    }

    println!("\n出力: {}", output.display());
    Ok(Outcome::Compressed {
        in_size: info.size_bytes,
        out_size,
        vmaf: vmaf_mean,
    })
}

fn print_summary(results: &[(PathBuf, Result<Outcome>)]) {
    println!("\n========== サマリー ==========");
    let (mut total_in, mut total_out, mut failed) = (0u64, 0u64, 0usize);

    for (path, result) in results {
        let name = path
            .file_name()
            .map_or_else(|| path.display().to_string(), |n| n.to_string_lossy().into_owned());
        match result {
            Ok(Outcome::Compressed { in_size, out_size, vmaf }) => {
                total_in += in_size;
                total_out += out_size;
                let vmaf_note = vmaf.map_or(String::new(), |v| format!(" VMAF {v:.1}"));
                println!(
                    "✓ {name}: {} → {} ({:.1}%){vmaf_note}",
                    human_size(*in_size),
                    human_size(*out_size),
                    *out_size as f64 / *in_size as f64 * 100.0,
                );
            }
            Ok(Outcome::NoGain) => {
                println!("- {name}: 縮まないため出力を破棄(元のまま)");
            }
            Err(e) => {
                failed += 1;
                println!("✗ {name}: {e:#}");
            }
        }
    }

    if total_in > 0 {
        println!(
            "\n合計: {} → {} ({} 削減)",
            human_size(total_in),
            human_size(total_out),
            human_size(total_in - total_out),
        );
    }
    if failed > 0 {
        println!("{failed}件失敗しました");
    }
}

/// ffmpeg/ffprobeの存在とlibvmaf対応を起動時に確認する
fn check_ffmpeg() -> Result<()> {
    let filters = Command::new("ffmpeg")
        .args(["-hide_banner", "-filters"])
        .output()
        .context("ffmpegが見つかりません。`brew install ffmpeg` でインストールしてください")?;
    if !String::from_utf8_lossy(&filters.stdout).contains("libvmaf") {
        bail!("このffmpegはlibvmaf非対応です。VMAF測定には対応ビルドが必要です");
    }
    Ok(())
}

fn interpret_vmaf(mean: f64, smaller: &str, better: &str) -> String {
    match mean {
        m if m >= 95.0 => format!(
            "→ 元動画とほぼ見分けがつかない品質です。まだ余裕があるので {smaller} でさらに縮むかもしれません。"
        ),
        m if m >= 90.0 => "→ 注視すれば違いが分かる程度。通常の視聴には十分な品質です。".into(),
        _ => format!("→ 劣化が知覚できる可能性があります。{better} を試してください。"),
    }
}

fn human_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{mb:.1} MB")
    }
}
