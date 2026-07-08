#pragma once

// metal-cppを知らないmain.mm側から橋渡しするための、あえてvoid*だけを使うインターフェース。
//
// 本物のCocoa/QuartzCore(Objective-C)ヘッダーと metal-cpp のヘッダーを同じ翻訳単位でincludeすると、
// Foundationのグローバル定数(NSBundleDidLoadNotificationなど)をmetal-cppと実SDKの両方が
// 同じシンボル名・別の型で再宣言してしまい、コンパイルエラーになる。
// そのため windowing(本物のCocoa, main.mm) と rendering(metal-cpp, Renderer.cpp) を
// 別の翻訳単位に分離し、境界はvoid*でやり取りする
// (metal-cppのNS::Objectは中身がobjc_object*そのものなので、id related型とvoid*は
// reinterpret_cast/__bridgeで自由に行き来できる)。

class Renderer;

Renderer* RendererCreate();
void RendererDestroy(Renderer* pRenderer);

// 戻り値の実体は id<MTLDevice>。CAMetalLayerのdeviceプロパティに設定する用
void* RendererGetDevice(Renderer* pRenderer);

// pMetalLayerの実体はCAMetalLayer*
void RendererDraw(Renderer* pRenderer, void* pMetalLayer);
