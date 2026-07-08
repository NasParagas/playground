# claude-sample

`../metal-cpp` を使ったMetalプログラミングのサンプル集。難易度順に4つ並んでいる。
ビルドは全部プロジェクトルートの `justfile` 経由 (`just --list` でレシピ一覧)。

```sh
just run-device-info   # 01
just run-compute       # 02
just run-offscreen     # 03
just run-window        # 04
just build-all         # まとめてビルドだけ
just clean             # build/・metallib・出力画像を消す
```

## 一覧

| # | ディレクトリ | 内容 | 新しく出てくるAPI |
|---|---|---|---|
| 01 | `01_device_info` | `MTL::Device` を取得して情報を表示するだけ | `MTL::CreateSystemDefaultDevice`, `supportsFamily` |
| 02 | `02_compute_add` | computeシェーダーでfloat配列を加算(GPGPU) | `MTL::Library`, `ComputePipelineState`, `dispatchThreads` |
| 03 | `03_offscreen_triangle` | 三角形をオフスクリーンテクスチャに描き、PPMに書き出す | `RenderPipelineState`, `RenderPassDescriptor`, `Texture::getBytes` |
| 04 | `04_window_triangle` | AppKitでウィンドウを開いて三角形を描画 | `CA::MetalLayer`, `CA::MetalDrawable`, AppKit連携 |

各サンプルは基本的に単一ファイル完結(04だけ事情があって3ファイル、後述)。
`.metal` シェーダーは `xcrun -sdk macosx metal` で `.metallib` にコンパイルし、実行時にホスト側から
`newLibrary(NS::URL::fileURLWithPath(...), &pError)` で読み込む方式に統一している
(カレントディレクトリ相対の `./xxx.metallib` を見に行くので、`justfile` は必ず該当ディレクトリに `cd` してから実行する)。

## 01_device_info

一番小さいサンプル。`MTL::CreateSystemDefaultDevice()` で得たデバイスから名前・統合メモリの有無・
GPUファミリー(`GPUFamilyApple7`〜`Apple10`, `Mac2`, `Metal3`)対応状況などを表示するだけ。
ウィンドウもシェーダーコンパイルも不要なので、ビルド環境の疎通確認に丁度いい。

## 02_compute_add

`add.metal` の `add_arrays` カーネルで2つのfloat配列(約100万要素)を要素ごとに加算し、
CPU側で結果を検証する。流れは:

1. `newLibrary` → `newFunction("add_arrays")` → `newComputePipelineState`
2. `MTL::ResourceStorageModeShared` でバッファを3本確保(A, B, 結果)
3. `ComputeCommandEncoder` に `setBuffer` ×3 → `dispatchThreads`
4. `commit()` → `waitUntilCompleted()` → `contents()` で直接CPUから結果を読む

Apple SiliconはUnified Memoryなので `StorageModeShared` にしておくとBlit転送なしで
GPU書き込み結果をそのままCPU側ポインタから読める。

## 03_offscreen_triangle

ウィンドウを一切使わず、三角形をオフスクリーンの `MTL::Texture` に描画してPPM画像に書き出す。
レンダーパイプラインの最小構成を学ぶのに向いている:

1. `RenderPipelineDescriptor` に vertex/fragment関数とカラーアタッチメントのpixelFormatを設定
2. `TextureDescriptor::texture2DDescriptor` でレンダーターゲット用テクスチャを作成
   (`StorageModeShared` にしておくと `getBytes()` で直接読める)
3. `RenderPassDescriptor` のcolorAttachmentにそのテクスチャを紐付け、`LoadActionClear`/`StoreActionStore`
4. `RenderCommandEncoder` で `drawPrimitives(PrimitiveTypeTriangle, 0, 3)`
5. `Texture::getBytes()` でCPUにピクセルを読み出し、自前でPPM(P6)ヘッダを付けて書き出す

出力される `triangle.ppm` はmacOSのPreviewでは直接開けないことが多いので、見るなら
`sips -s format png triangle.ppm --out triangle.png && open triangle.png`。

## 04_window_triangle

一番手強かったサンプル。実際にウィンドウを開いて毎フレーム描画する。

### なぜ3ファイルに分かれているか

最初は `main.mm` 1ファイルに `#import <Cocoa/Cocoa.h>` と `#include <Metal/Metal.hpp>`(metal-cpp)を
両方書いていたが、以下のビルドエラーで動かなかった:

```
error: redeclaration of 'NSBundleDidLoadNotification' with a different type:
  'const NS::NotificationName' (aka 'NS::String *const') vs 'const NSNotificationName ...'
```

原因: metal-cppの `Foundation/Foundation.hpp` は `NSBundleDidLoadNotification` などの
Foundationのグローバル定数を、**実際のObjective-Cランタイムが export している同じシンボル名**を
`NS::String* const` として再宣言することで使えるようにしている(意図的な設計)。
ところが本物の `<Foundation/Foundation.h>`(`Cocoa.h` 経由で入ってくる)も同じシンボル名を
`NSString* const` として宣言しているため、**両方を同じ翻訳単位でincludeすると型の異なる二重宣言になり
コンパイルエラーになる**。これはmetal-cppの既知の制約で、ウィンドウ管理コードとmetal-cppコードは
別ファイルに分けるしかない。

そこで:
- `main.mm` — 本物のCocoa/QuartzCore/Metalヘッダーだけを使う(metal-cppは一切importしない)
- `Renderer.cpp` — metal-cppだけを使う(本物のCocoaヘッダーは一切importしない)。
  `NS_PRIVATE_IMPLEMENTATION` 等のマクロもここにしか書かない
- `Renderer.hpp` — 両者の橋渡し用。`Renderer*`/`void*` だけで構成された、どちらのヘッダーにも
  依存しないインターフェース

境界を越えるときは `void*` に reinterpret_cast / `__bridge` するだけでよい。これは
metal-cppの `NS::Object` が中身として `objc_object*` を継承しているだけ(追加メンバなし、仮想関数なし)で、
ABI的に本物の `id` と同一だから成立するトリック
(`Foundation/NSObject.hpp` の `class Object : public Referencing<Object, objc_object>` を参照)。

### ウィンドウが表示されなかった話

TU分離でビルドは通ったが、実行してもウィンドウが画面に出ず、Dockにアイコンだけ出て
Ctrl-Cしないと終了しないという状態になった。`NSLog`/`fprintf`で各ステップにログを仕込んで
原因を特定した(該当箇所は `// [debug]` コメントとして残してある):

```
wantsLayer設定後: self.layer=<NSViewBackingLayer: 0x...>
```

`+layerClass` を `CAMetalLayer` にオーバーライドしていたのに、実際に生成された `self.layer` は
`NSViewBackingLayer`(AppKitの汎用レイヤー)だった。この状態で `layer.device = ...` のような
`CAMetalLayer` 専用プロパティを叩いていたため、期待通りに動いていなかった
(このmacOS 27 betaでは `+layerClass` だけでは意図通りにならなかった、というのが実際に観測した事実)。

対策として `-makeBackingLayer` のオーバーライドを追加し、さらに念のため
`self.layer` が `CAMetalLayer` になっていなければ直接 `self.layer = [CAMetalLayer layer]` を
代入するようにしたら解決した:

```objc
- (CALayer*)makeBackingLayer {
    return [CAMetalLayer layer];
}
// ...
self.wantsLayer = YES;
if (![self.layer isKindOfClass:[CAMetalLayer class]]) {
    self.layer = [CAMetalLayer layer];
}
```

教訓: `+layerClass`/`-makeBackingLayer` に頼るより、`self.layer` を明示的に代入したほうが確実。
特にbeta OS上ではAppKit内部の挙動が変わっている可能性があるので、型を信用せず
`isKindOfClass:` で確認してから使うと安全。

## metal-cppを使う上で気づいたこと

- **所有権の命名規則**: `alloc`/`new`/`copy`/`mutableCopy`/`Create` で始まるメソッドが返すオブジェクトは
  呼び出し側が所有する(=いずれ `release()` が必要)。それ以外(例: `RenderPassDescriptor::renderPassDescriptor()`
  のような便利コンストラクタ)はautoreleaseされている。README(`metal-cpp/README.md`)に詳しい説明がある。
- **`NS_PRIVATE_IMPLEMENTATION`/`CA_PRIVATE_IMPLEMENTATION`/`MTL_PRIVATE_IMPLEMENTATION`** は
  リンク対象のバイナリ全体でちょうど1つの翻訳単位だけに定義する(重複定義でも未定義でもエラーになる)。
  04では `Renderer.cpp` だけに置いている。
- **`packed_float2`/`packed_float3` とホスト側構造体のアライメント**: MSLの `float3` は(SIMD命令の都合で)
  デフォルトで16バイトアラインされ `float4` と同じサイズになる。ホスト側でパディングなしの
  `struct { float[2]; float[3]; }` を渡すと丸ごとズレるので、シェーダー側で
  `packed_float2`/`packed_float3` を使ってホスト側のレイアウトに揃える必要がある
  (02・03・04すべてこれで揃えている)。
- **`StorageModeShared`**: Apple SiliconはUnified Memoryなので、CPU/GPU間で読み書きするバッファ/テクスチャは
  `StorageModeShared` にしておくとBlit転送や `synchronize` なしで `contents()`/`getBytes()` が直接使える。
  Intel Mac(Discrete GPU)だと事情が変わる。
- **本物のObjective-CヘッダーとMetal-cppヘッダーは同じ翻訳単位に混在させない**(04の顛末を参照)。
  AppKit/Cocoaを使うウィンドウ管理コードと、metal-cppを使う描画コードは常にファイルを分けるのが安全。
