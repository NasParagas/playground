# m2w — 動画から音声を抽出してwavに変換するCLI

mp4/mov/mkvなどの動画ファイルから、ffmpegを使って音声トラックをwav(PCM)として
取り出すツール。文字起こしや音声解析の前処理を想定。

- 既定では出力先が既存なら中断(`--force` で上書き)
- サンプリングレート・チャンネル数・ビット深度を指定可能(省略時は元のまま / 16bit)

## 必要なもの

- ffmpeg
- Rustツールチェーン

```bash
cargo build --release
# バイナリは target/release/m2w
```

## 使い方

```bash
m2w input.mp4                       # input.wav を出力 (16bit PCM)
m2w input.mp4 -o voice.wav          # 出力先を指定
m2w input.mp4 -r 16000 -c 1         # 16kHz モノラル (Whisper等の文字起こし向け)
m2w input.mp4 --bits 24             # 24bit PCM で出力
m2w input.mp4 --force               # 既存のwavを上書き

# バッチ処理
m2w a.mp4 b.mov c.mkv               # 複数ファイルを順に処理
m2w videos/                         # ディレクトリ直下の動画を一括処理
m2w videos/ --out-dir wav/          # 出力先ディレクトリを指定
```

対象拡張子(ディレクトリ指定時): mp4, mov, mkv, avi, webm, m4v, ts, mts, flv
