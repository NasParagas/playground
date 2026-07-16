use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::Parser;

/// 動画ファイル(mp4/mov/mkvなど)から音声を抽出してwavに変換する
#[derive(Parser)]
#[command(version, about)]
struct Args {
    /// 入力の動画ファイル、またはディレクトリ(直下の動画を一括処理)。複数指定可
    #[arg(required = true)]
    inputs: Vec<PathBuf>,

    /// 出力パス。入力が1ファイルのときのみ指定可 (既定: <input>.wav)
    #[arg(short, long, conflicts_with = "out_dir")]
    output: Option<PathBuf>,

    /// 出力先ディレクトリ。元と同じファイル名(拡張子だけwav)で出力する
    #[arg(long)]
    out_dir: Option<PathBuf>,

    /// サンプリングレート[Hz] (例: 16000, 44100)。省略時は元のまま
    #[arg(short = 'r', long)]
    rate: Option<u32>,

    /// チャンネル数 (1=モノラル, 2=ステレオ)。省略時は元のまま
    #[arg(short = 'c', long)]
    channels: Option<u8>,

    /// ビット深度 (16, 24, 32)
    #[arg(long, default_value_t = 16, value_parser = parse_bits)]
    bits: u8,

    /// 出力先が既に存在しても上書きする
    #[arg(long)]
    force: bool,
}

fn parse_bits(s: &str) -> Result<u8, String> {
    match s {
        "16" => Ok(16),
        "24" => Ok(24),
        "32" => Ok(32),
        _ => Err("16, 24, 32 のいずれかを指定してください".into()),
    }
}

/// ディレクトリ入力のとき対象とみなす動画の拡張子
const MOVIE_EXTS: &[&str] = &["mp4", "mov", "mkv", "avi", "webm", "m4v", "ts", "mts", "flv"];

fn main() -> Result<()> {
    let args = Args::parse();

    check_ffmpeg()?;

    if let Some(ch) = args.channels
        && ch == 0
    {
        bail!("--channels は1以上を指定してください");
    }

    let inputs = collect_inputs(&args.inputs)?;
    if inputs.len() > 1 && args.output.is_some() {
        bail!("-o/--output は入力が1ファイルのときだけ使えます。複数入力には --out-dir を");
    }
    if let Some(dir) = &args.out_dir {
        fs::create_dir_all(dir)
            .with_context(|| format!("出力ディレクトリを作成できません: {}", dir.display()))?;
    }

    let total = inputs.len();
    let mut failed = 0usize;
    for (i, input) in inputs.iter().enumerate() {
        if total > 1 {
            println!("[{}/{}] {}", i + 1, total, input.display());
        }
        let output = resolve_output(input, &args);
        // バッチでは1件の失敗で全体を止めず、記録して次へ進む
        if let Err(e) = convert(input, &output, &args) {
            eprintln!("エラー: {e:#}");
            failed += 1;
        }
    }

    if failed > 0 {
        bail!("{failed}/{total} 件失敗しました");
    }
    Ok(())
}

/// 入力引数を処理対象ファイルの一覧に展開する。
/// ディレクトリは直下の動画ファイル(MOVIE_EXTS)に展開する。
fn collect_inputs(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let is_movie = |p: &Path| {
        p.extension()
            .is_some_and(|e| MOVIE_EXTS.iter().any(|m| e.eq_ignore_ascii_case(m)))
    };

    let mut files = Vec::new();
    for path in paths {
        if path.is_dir() {
            let mut found: Vec<PathBuf> = fs::read_dir(path)
                .with_context(|| format!("ディレクトリを読めません: {}", path.display()))?
                .filter_map(|entry| entry.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && is_movie(p))
                .collect();
            found.sort();
            if found.is_empty() {
                println!("注意: {} の直下に動画ファイルがありません", path.display());
            }
            files.extend(found);
        } else if path.is_file() {
            files.push(path.clone());
        } else {
            bail!("入力が見つかりません: {}", path.display());
        }
    }

    if files.is_empty() {
        bail!("処理対象の動画がありません");
    }
    Ok(files)
}

fn resolve_output(input: &Path, args: &Args) -> PathBuf {
    if let Some(output) = &args.output {
        output.clone()
    } else if let Some(dir) = &args.out_dir {
        dir.join(input.with_extension("wav").file_name().unwrap_or_default())
    } else {
        input.with_extension("wav")
    }
}

fn convert(input: &Path, output: &Path, args: &Args) -> Result<()> {
    if output.exists() && !args.force {
        bail!(
            "出力先が既に存在します(上書きするには --force): {}",
            output.display()
        );
    }

    let codec = match args.bits {
        16 => "pcm_s16le",
        24 => "pcm_s24le",
        32 => "pcm_s32le",
        _ => unreachable!(),
    };

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(input)
        // -vn: 映像を捨てて音声だけを取り出す
        .args(["-vn", "-acodec", codec]);
    if let Some(rate) = args.rate {
        cmd.args(["-ar", &rate.to_string()]);
    }
    if let Some(ch) = args.channels {
        cmd.args(["-ac", &ch.to_string()]);
    }
    cmd.arg(output);

    let status = cmd.status().context("ffmpegの実行に失敗しました")?;
    if !status.success() {
        // 音声ストリームなし等でffmpegが失敗すると空ファイルが残ることがあるため掃除する
        let _ = fs::remove_file(output);
        bail!("ffmpegが失敗しました: {}", input.display());
    }

    let size = fs::metadata(output)
        .context("出力ファイルを確認できません")?
        .len();
    println!("出力: {} ({})", output.display(), human_size(size));
    Ok(())
}

/// ffmpegの存在を起動時に確認する
fn check_ffmpeg() -> Result<()> {
    Command::new("ffmpeg")
        .args(["-hide_banner", "-version"])
        .output()
        .context("ffmpegが見つかりません。`brew install ffmpeg` 等でインストールしてください")?;
    Ok(())
}

fn human_size(bytes: u64) -> String {
    let mb = bytes as f64 / 1024.0 / 1024.0;
    if mb >= 1024.0 {
        format!("{:.2} GB", mb / 1024.0)
    } else {
        format!("{mb:.1} MB")
    }
}
