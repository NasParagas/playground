# C/C++/Obj-C++の自動format整備 (2026-07-09, Claudeによる作業メモ)

## 経緯

「`:w`で`.mm`や`.cpp`を保存するとformatされるんだけど、誰のせい？」という質問から
調査を開始。最終的に「全filetypeで自動formatを有効にして、無効化したい時だけ明示的に切る」
という方針に転換し、`.clang-format`も新規に用意した。

## 原因調査の結果

`~/dotfiles/.config/nvim/lua/plugins/conform.lua`のkickstart.nvim由来のデフォルト設定が、
以下の2つの理由で「`.cpp`/`.c`は無効・`.mm`(objcpp)は有効」という中途半端な状態になっていた。

1. **`conform.lua`の`disable_filetypes = { c = true, cpp = true }`がobjcppを考慮していなかった**。
   `.mm`のfiletypeは`objcpp`なので、このリストに引っかからず`lsp_format = "fallback"`が有効なまま
   → clangdのLSP formatting(実体はclang-format)が保存時に走っていた
2. **`config/lsp/servers.lua`のclangd設定`filetypes = {"c", "cpp"}`は実は無効化として機能していなかった**。
   nvim 0.11+で`require("lspconfig").clangd.setup()`は互換レイヤー
   (`lspconfig/configs.lua`)を経由し、`vim.tbl_deep_extend("keep", user_config, default_config)`で
   マージされる。配列はインデックスごとにマージされるため、ユーザー指定の`{"c","cpp"}`(2要素)は
   デフォルトの`{"c","cpp","objc","objcpp","cuda"}`(5要素)の3〜5番目の要素をそのまま素通りさせてしまう。
   結果、clangdは常に`{c, cpp, objc, objcpp, cuda}`全部にattachしていた(`vim.lsp.get_clients()`で実測済み)

副次的な発見: `.mm`はneovim組み込みの`vim.filetype.detect.mm`によって、
**ファイル冒頭20行に`#include`/`#import`/`@import`/ブロックコメントが無いと`nroff`(troffマクロ)扱いになる**
(`#include`等が無い空ファイルなどで再現)。実プロジェクトの`.mm`はほぼ確実に`#import`があるので
通常は`objcpp`になる。

## 採用した構成

「無効化リストで例外を作る」のではなく、「全filetypeでデフォルトformat有効・止めたい時だけ明示的に無効化」
という方式に変更した。

### `conform.lua`の変更

- `format_on_save`の`disable_filetypes`分岐を削除。全filetypeで`{ timeout_ms = 500, lsp_format = "fallback" }`
- 代わりに`vim.g.disable_autoformat` / `vim.b[bufnr].disable_autoformat`を見るガードを追加
- `init`でユーザーコマンドを2つ追加(conform.nvim公式レシピ):
  - `:FormatDisable` — 全体で自動format無効化
  - `:FormatDisable!` (bang付き) — **今のバッファだけ**無効化
  - `:FormatEnable` — 両方とも再有効化

### スタイル設定は`.clang-format`側

**pluginの設定(conform.lua/servers.lua)は「いつ・どのツールを呼ぶか」だけを決める。
実際のインデント幅・波括弧位置などのスタイルは、フォーマッタ自身の設定ファイルで決まる。**
clangdのLSP formattingは中身がclang-formatなので、編集ファイルのディレクトリから上に向かって
`.clang-format`を探して使う。

- `metal/.clang-format`: `BasedOnStyle: Google` + `IndentWidth: 4`
  (Googleスタイルはデフォルト2スペースインデントだが、既存コード(`Renderer.cpp`等)が
  4スペースで書かれていたため上書き)
- `metal/metal-cpp/.clang-format`: `DisableFormat: true`
  (Apple製のベンダーコードなので、うっかり保存して大量差分になるのを防止)

補足: `.clang-tidy`はclang-tidyの**lintチェック**設定であり、フォーマットスタイルとは別物
(clangdの`--clang-tidy`フラグで既に有効)。フォーマットの見た目を変えたいなら`.clang-format`が正解。

### 行単位・範囲単位の無効化

clang-formatのコメントマーカーで囲むと、その範囲だけ元の書式を保持できる。
単独行専用のマーカーは無く、必ずoff/onのペアで囲む。

```cpp
// clang-format off
int   matrix[3]  =  {1,    2,     3};
// clang-format on
```

## 検証済みの動作

headless nvim(`nvim --headless -u ~/.config/nvim/init.lua -l <script>.lua`)で
プロジェクト直下に使い捨てファイルを作り、実際に`:w`させて確認(検証後は削除済み)。

- `.cpp`保存 → 修正前は`disable_filetypes`によりformat自体がスキップされ内容不変
- 修正後、`.cpp`保存 → clangd経由でGoogleスタイル・4スペースインデントに整形される
  (`int main(){` → `int main() {`、`if(x==1){` → `if (x == 1) {`等)
- `// clang-format off` / `on`で囲んだ行だけ元の不揃いなスペースが保持され、
  それ以外の行は通常通り整形される

## 豆知識

- `vim.tbl_deep_extend("keep", short_array, long_array)`は配列(数値キーのテーブル)に対して
  「短い方に無い添字はロング側の値がそのまま残る」ため、配列を**部分的に短くして絞り込む**目的では
  使えない。置き換えたいなら丸ごと同じ長さの配列を渡すか、別のロジックで絞る必要がある
- neovimでLSPクライアントの実際のfiletypes等を確認したい時は
  `vim.lsp.get_clients({ bufnr = 0 })`の`client.config.filetypes`が手軽
- `.mm`ファイルのfiletype判定は中身依存(`vim.filetype.detect.mm`)。空ファイルや
  コメントだけのファイルだと`nroff`になるので、LSPが付かない時はまずfiletypeを疑う
