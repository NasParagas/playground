# Metal開発のエディタ環境構築 (2026-07-08, Claudeによる作業メモ)

## 経緯

もともと「`compile_flags.txt`にフラグを追加してね」と言われていたのを、
`.clangd`(YAML形式でより柔軟)で代用することにした。
ところが`src/add.metal`を開くと以下のエラーが出た:

```
src/add.metal|1 col 1 error| Unable to handle compilation, expected exactly one compiler job in ''
```

## 原因調査の結果

**clangdはMetal Shading Language(`.metal`)を原理的に解析できない。**

- neovimはデフォルト設定(init.luaの`vim.filetype.add`)で`*.metal`をfiletype `cpp`として扱っていた
- そのためclangdが`.metal`バッファに自動アタッチし、clangは`.metal`という言語を知らないので上記エラーになっていた
- `.clangd`の`CompileFlags: Compiler:`に本物の`metal`コンパイラを指定しても無駄。
  clangdは診断・補完を**自前に内蔵したclangフロントエンド**で行うため、外部コンパイラは使われない
- vanilla clangd(mason版 22.1.6)にもXcode同梱のApple clangd(21.0.0)にも
  `-x metal`という言語は存在しない(`language not recognized: 'metal'`)
- Metal Toolchain(`xcodebuild -downloadComponent metalToolchain`で入れたやつ)の中身も確認したが、
  コンパイラ・リンカ類のみでLSPサーバーは同梱されていない
- つまり**Metal用のLSPはAppleから提供されておらず、意味論的な補完・定義ジャンプはXcodeエディタ専用機能**

## 採用した構成

「本物のLSP」は無理だが、実用ラインとして以下を構築した:

| 機能 | 手段 |
|---|---|
| シンタックスハイライト | treesitterのcppパーサを流用(MSLはC++ベースなのでほぼ違和感なし) |
| エラー・警告の診断表示 | ファイルを開いた時と保存時に本物の`metal`コンパイラを走らせる |
| 補完・定義ジャンプ | シェーダー側は諦め(ホスト側C++はclangdでフルに効く) |

### neovim側の変更 (dotfiles)

1. **`init.lua`**: `vim.filetype.add`のマッピングを`metal = "cpp"` → `metal = "metal"`に変更。
   専用filetypeにすることでclangdがアタッチしなくなる(lspconfigのclangdは
   c/cpp/objc等のfiletypeにしか反応しない)。これがエラー解消の本丸
2. **`lua/plugins/nvim_treesitter.lua`**:
   `vim.treesitter.language.register("cpp", "metal")`でmetal filetypeにcppパーサを紐付け、
   ハイライト起動のFileType autocmdのpatternに`"metal"`を追加
3. **`after/ftplugin/metal.lua`** (新規): cindent等の設定に加え、
   バッファを開いた時とBufWritePostで
   `xcrun -sdk macosx metal -fsyntax-only -Wall <file>`を非同期実行(`vim.system`)し、
   出力(`file:line:col: severity: message`形式)をパースして
   `vim.diagnostic.set`でバッファに表示する自前リンター。
   インクルード先ファイルのエラーはファイル先頭(1行目)にアンカーして表示

### プロジェクト側の変更

- **`.clangd`**: ホスト側C++専用の設定に整理:

  ```yaml
  CompileFlags:
    Add:
      - -std=c++17
      - -I/Users/niiyama/ws/playground/metal-cpp
      - -isysroot
      - /Applications/Xcode-beta.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX27.0.sdk
  ```

  注意点:
  - `-isysroot`のように引数が別トークンのフラグは、YAMLでも2要素に分けて書く必要がある
  - **`-Imetal-cpp`(相対パス)は`-I/Users/niiyama/ws/playground/metal-cpp`(絶対パス)に修正した**。
    metal-cppはこのプロジェクトの中ではなく隣(`playground/metal-cpp`)にクローンされており、
    さらにclangdのフォールバックコマンドはソースファイルのあるディレクトリ基準で相対パスを解決するため、
    相対パスだとどう転んでも`Metal/Metal.hpp`が見つからなかった
- 一時的に作った`tools/metal-cc`(metalコンパイラのラッパー)は、上記の通り無意味と判明したので削除

## 検証済みの動作

- `src/add.metal`を開く → filetype=metal / treesitterハイライト有効 / LSPクライアント0(エラー消滅) /
  書きかけの`kernel`行に本物のコンパイラ由来の診断「expected unqualified-id」が表示される
- 正しく書いた`kernel void add_arrays(...)`シェーダー → 診断0件
- `MTL::CreateSystemDefaultDevice()`を含むホスト側C++で`clangd --check` → エラー0件
  (`Metal/Metal.hpp`のインクルード解決を確認)

## 豆知識

- `clangd --check=<file>`で、エディタを介さずにclangdがそのファイルをどう処理するか
  (フォールバックコンパイルコマンド、cc1引数、診断)をCLIから確認できる。デバッグに便利
- `compile_flags.txt`と`.clangd`が両方あると`compile_flags.txt`が優先されるので、どちらか片方だけ置く
