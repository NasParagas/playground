// metal-cppで一番小さく書けるサンプル: デフォルトのMTL::Deviceを取得し、
// 名前やGPUファミリーのサポート状況を表示するだけ。
// ウィンドウもシェーダーコンパイルも不要なので、まずはこれでビルド環境の疎通確認をする。

#define NS_PRIVATE_IMPLEMENTATION
#define MTL_PRIVATE_IMPLEMENTATION

#include <cstdio>

#include <Metal/Metal.hpp>

namespace {

struct FamilyCheck {
    MTL::GPUFamily family;
    const char* label;
};

} // namespace

int main() {
    MTL::Device* pDevice = MTL::CreateSystemDefaultDevice();
    if (pDevice == nullptr) {
        fprintf(stderr, "Metalデバイスが見つかりませんでした\n");
        return 1;
    }

    printf("device name                 : %s\n", pDevice->name()->utf8String());
    printf("registryID                  : %llu\n", (unsigned long long)pDevice->registryID());
    printf("low power                   : %s\n", pDevice->isLowPower() ? "yes" : "no");
    printf("removable                   : %s\n", pDevice->isRemovable() ? "yes" : "no");
    printf("unified memory              : %s\n", pDevice->hasUnifiedMemory() ? "yes" : "no");
    printf("max buffer length           : %.2f MiB\n", pDevice->maxBufferLength() / (1024.0 * 1024.0));

    MTL::Size maxThreads = pDevice->maxThreadsPerThreadgroup();
    printf("max threads per threadgroup : %llu x %llu x %llu\n",
           (unsigned long long)maxThreads.width,
           (unsigned long long)maxThreads.height,
           (unsigned long long)maxThreads.depth);
    printf("recommended max working set : %.2f MiB\n",
           pDevice->recommendedMaxWorkingSetSize() / (1024.0 * 1024.0));

    const FamilyCheck families[] = {
        {MTL::GPUFamilyApple7, "Apple7"},
        {MTL::GPUFamilyApple8, "Apple8"},
        {MTL::GPUFamilyApple9, "Apple9"},
        {MTL::GPUFamilyApple10, "Apple10"},
        {MTL::GPUFamilyMac2, "Mac2"},
        {MTL::GPUFamilyMetal3, "Metal3"},
    };

    printf("--- GPU family support ---\n");
    for (const auto& f : families) {
        printf("  %-8s: %s\n", f.label, pDevice->supportsFamily(f.family) ? "yes" : "no");
    }

    pDevice->release();
    return 0;
}
