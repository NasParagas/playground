# Metalプログラミングの基礎知識 (概念編)

`claude-sample/` のコードを読むために最低限必要な前提知識をまとめたもの。
ビルド方法や個別サンプルの解説は `claude-sample/README.claude.md` を、
環境構築のトラブルシュートは同じ `claude-memo/` の他のファイルを参照。

## 1. Metalとは何か・設計思想

- **CPUとGPUは非同期**
- CPU側はコマンドを積んで(encode)GPUに投げる(commit)。基本的にブロックしない
- **3つの言語バインディングが存在する**
  - Objective-C(`Metal.h`、ネイティブ)
  - Swift
  - `metal-cpp`
    - Appleが公式に配布するヘッダオンリーのC++ラッパー。ARCが効かないので手動retain/releaseが必要 → 詳細は6章)
  - このプロジェクトでは主に`metal-cpp`、
    - ウィンドウ管理部分だけ本物のObjective-C(AppKit)を使っている(04参照)。

## 2. オブジェクトの相関図

```
MTL::Device ──creates──> MTL::CommandQueue
    │                          │
    ├──creates──> MTL::Library ──newFunction──> MTL::Function
    │                                                 │
    ├──creates(Function経由)──> RenderPipelineState / ComputePipelineState
    │
    └──creates──> MTL::Buffer, MTL::Texture

MTL::CommandQueue ──commandBuffer()──> MTL::CommandBuffer   (だいたい1フレーム/1タスクにつき1個)
                                             │
                                             ├─ renderCommandEncoder(pass) ─┐
                                             ├─ computeCommandEncoder()    ─┼─ setXXX(...)の列 → endEncoding()
                                             └─ blitCommandEncoder()      ─┘
                                             │
                                     commit() → (GPUが実行) → waitUntilCompleted() / completion handler
```

- **MTLDevice**
  - HWとしてのGPUを抽象化したもの(TODO: もの？)。ほぼ全てのMetalオブジェクトはDeviceから作る
  - `MTLDevice`オブジェクトへの参照を取得するには、`MTL::CreateSystemDefaultDevice()`を利用する
- **MTLCommandQueue**
  - コマンドバッファの実行順を管理するqueue
  - 基本はアプリ全体で1個作って使い回すらしい
- **CommandBuffer**
  - GPUへ投げるcommandを格納するbuffer。
  - 中にEncoderで命令を積んで、`commit()`することでGPUがそのbufferを実行する
  - 一旦commitしたbufferは、スケジューリングハンドラや処理完了のハンドラ実行の待機、実行statusの確認などしか行えず、**command bufferオブジェクトの再利用はできない**
- **CommandEncoder**: 実際にGPU命令(`setBuffer`, `drawPrimitives`, `dispatchThreads`等)を積むオブジェクト。
  用途別に3種類あり(Render/Compute/Blit)、使い終わったら必ず`endEncoding()`する。
  1つのCommandBufferの中に複数のEncoderを順番に(同時にではなく)作れる。

## 3. 主要クラス早見表

| クラス | 役割 |
|---|---|
| `MTL::Device` | GPUデバイスそのもの。全ての起点 |
| `MTL::CommandQueue` | CommandBufferの投入先。使い回す |
| `MTL::CommandBuffer` | 1回分のGPU命令列の入れ物。`commit()`で提出 |
| `MTL::RenderCommandEncoder` | 描画命令(vertex/fragment実行)を積む |
| `MTL::ComputeCommandEncoder` | GPGPU命令(kernel実行)を積む |
| `MTL::BlitCommandEncoder` | メモリコピー等の単純転送命令を積む(今回のサンプルでは未使用) |
| `MTL::Library` | コンパイル済み`.metal`シェーダー群(`.metallib`)へのハンドル |
| `MTL::Function` | Library内の1関数(vertex/fragment/kernel)への参照 |
| `MTL::RenderPipelineDescriptor` → `MTL::RenderPipelineState` | vertex/fragment関数の組み合わせ・出力フォーマット等を固めた「描画設定一式」。作成コストが高いので事前に1回作って使い回す |
| `MTL::ComputePipelineState` | kernel関数を固めた「計算設定一式」 |
| `MTL::Buffer` | GPUから見える生メモリ(構造体配列など任意) |
| `MTL::Texture` | 画像専用リソース(pixelFormat・サイズ等のメタデータ持ち) |
| `MTL::RenderPassDescriptor` | 「今回の描画はどのテクスチャに、どうロード/ストアするか」の指定(色・深度アタッチメント) |
| `CA::MetalLayer` | 画面(ウィンドウ)に出すためのCore Animationレイヤー。drawableの供給元 |
| `CA::MetalDrawable` | `MetalLayer`から借りる「今フレーム描画して画面に出す用」のテクスチャ |

## 4. 2つのGPUプログラム: レンダー vs コンピュート

Metalで書けるシェーダー(GPU上で動く関数)は大きく2系統ある。

- **レンダーパイプライン**(03・04で使用): `vertex`関数 → (固定機能の)ラスタライズ → `fragment`関数、
  という固定の3段構成。頂点1個につき`vertex`関数が1回、画面上のピクセル1個につき`fragment`関数が1回
  呼ばれ、`drawPrimitives`で起動する。三角形を描くような用途はこちら。
- **コンピュートパイプライン**(02で使用): `kernel`関数を大量のスレッドで単純に並列実行するだけ
  (ラスタライズ等の固定機能は一切絡まない)。汎用計算(GPGPU)用で、`dispatchThreads`で起動する。
  スレッドは`grid`(全体の要素数)を`threadgroup`(GPU上の実行単位のかたまり)に分割して実行される
  — 02の`dispatchThreads(MTL::Size(kArrayLength,1,1), MTL::Size(threadGroupSize,1,1))`がこれ。

## 5. Metal Shading Language (MSL) の基本

- C++14ベースの独自言語。`#include <metal_stdlib>` / `using namespace metal;` から始めるのが定型。
- 関数には修飾子(`vertex` / `fragment` / `kernel`)を付け、GPU上のどの段で動くかを明示する。
- **`[[attribute]]`構文**でGPU側の暗黙情報とホスト側の設定を結びつける。よく出てくるもの:
  - `[[buffer(n)]]` — 引数がバインドされるバッファ番号。ホスト側の`setBuffer(buf, offset, n)`の
    `n`と対応(02・03・04で頻出)
  - `[[vertex_id]]` — 今処理している頂点のインデックス
  - `[[thread_position_in_grid]]` — コンピュートシェーダーで今のスレッドの位置(02)
  - `[[position]]` — vertex関数の戻り値のうち、クリップ空間(NDC、およそ-1〜1)の座標であることを示す
    必須フィールド
  - `[[stage_in]]` — ラスタライザが頂点間を補間した結果を、fragment関数の引数として受け取る指定(03・04)
- vertex関数は最低限`float4 position [[position]]`を返す必要がある。fragment関数は補間済みの値を受け取り、
  最終的なピクセルカラー(`float4`、各成分0.0〜1.0のRGBA)を返す。

## 6. リソース: Buffer / Texture とStorage Mode

- `MTL::Buffer`は「型のない生メモリ」で、構造体配列など何でも入れられる。`MTL::Texture`は
  画像専用で、`pixelFormat`やミップマップ枚数などのメタデータを持つ。
- **Storage Mode**(CPU/GPU間でどうメモリを共有するか):
  - `StorageModeShared` — CPU/GPUが同じメモリを見る。Apple SiliconはUnified Memoryなので、
    これだけで`contents()`/`getBytes()`から直接読み書きでき、明示的な同期が要らない
    (02・03・04は全部これ)。
  - `StorageModePrivate` — GPU専用。CPUからは触れないが最速。CPU⇔GPU間のやり取りが不要な
    中間リソース(レンダーターゲット等)向け。
  - `StorageModeManaged` — CPU/GPUそれぞれにコピーを持ち明示的に同期する方式。Intel Mac(Discrete GPU)
    向けで、Apple SiliconにはShared/Privateしか実質出てこない。
- ホスト側(C++)の構造体とMSL側の構造体は**メモリレイアウトを一致させる必要がある**。
  MSLの`float3`はSIMD都合でデフォルト16バイトアラインされ`float4`と同サイズになってしまうため、
  ホスト側にパディングを入れたくない場合はMSL側で`packed_float2`/`packed_float3`を使う
  (03・04の`Vertex`構造体参照)。

## 7. metal-cppのメモリ管理(ARCがない)

`metal-cpp`はC++なのでARC(自動参照カウント)が効かない。Cocoa/CoreFoundation由来の
明示的な参照カウントルールに従う必要がある:

1. `alloc` / `new` / `copy` / `mutableCopy` / `Create` で始まるメソッドが返すオブジェクトは
   **呼び出し側が所有**する(retainCount=1で返る)。使い終わったら`release()`が必要。
2. それ以外の便利コンストラクタ(例: `MTL::RenderPassDescriptor::alloc()->init()`ではなく
   `RenderPassDescriptor::renderPassDescriptor()`のような形)が返すオブジェクトは
   **autorelease pool行き**。明示的な`release()`は不要(ただし所有権を伸ばしたいなら`retain()`する)。
3. `NS_PRIVATE_IMPLEMENTATION` / `CA_PRIVATE_IMPLEMENTATION` / `MTL_PRIVATE_IMPLEMENTATION`は
   実装コード本体を注入するマクロで、リンクする実行バイナリ全体でちょうど1つの翻訳単位にのみ定義する
   (01〜03は単一ファイルなのでそのファイル、04は`Renderer.cpp`に集約)。

## 8. ウィンドウ表示の仕組み: CAMetalLayerとdrawable

オフスクリーン描画(03)は自前の`Texture`に描いて`getBytes()`で読むだけで完結するが、
実際に画面(ウィンドウ)に出す(04)にはOSのウィンドウ合成システム(Core Animation)との連携が要る。

- `CA::MetalLayer`(Objective-C側では`CAMetalLayer`)は「Metalで描画できるレイヤー」。
  `NSView`/`UIView`にこのレイヤーを持たせることで、そのビューがMetal描画の出力先になる。
- 毎フレームの流れ:
  1. `layer->nextDrawable()`で`CA::MetalDrawable`(中身はテクスチャ)を1枚借りる
  2. それを`RenderPassDescriptor`のcolorAttachmentに設定して通常通り描画
  3. `commandBuffer->presentDrawable(drawable)` → `commit()`
  4. OS側が適切なタイミング(vsync等)で実際に画面に反映する
- 04で`+layerClass`/`-makeBackingLayer`をオーバーライドしているのはこのレイヤーを
  `CAMetalLayer`にするため(実際にmacOS 27 betaでは効かず`self.layer`への直接代入が必要だった、
  という顛末は`claude-sample/README.claude.md`の04節参照)。

## 9. サンプルとの対応関係

| # | 新しく出てくる概念 |
|---|---|
| 01 | `MTL::Device`単体(2〜8章の前提となる「起点」を触るだけ) |
| 02 | 3章・4章(コンピュートパイプライン)・7章 |
| 03 | 3章・4章(レンダーパイプライン)・5章(MSL)・6章(Storage Mode) |
| 04 | 8章(CAMetalLayer/drawable)に加え、03までの全部の集大成 |

読む順番に迷ったら、まず1〜3章で全体像を掴んでから`01_device_info`のコードを読み、
4章以降を必要に応じて参照しながら02→03→04と進むのがおすすめ。
