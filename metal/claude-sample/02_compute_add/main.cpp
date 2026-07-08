// GPGPUの最小サンプル: 2つのfloat配列を足し算するcomputeカーネル(add.metal)を
// デバイス・コマンドキュー・バッファを用意してディスパッチし、結果をCPU側で検証する。
//
// add.metalは事前に `xcrun -sdk macosx metal` でadd.metallibにコンパイルしておく必要がある
// (justfileのレシピを参照)。実行時にこのmain.cppと同じディレクトリのadd.metallibを読み込む。

#define NS_PRIVATE_IMPLEMENTATION
#define MTL_PRIVATE_IMPLEMENTATION

#include <cmath>
#include <cstdio>
#include <cstdlib>

#include <Metal/Metal.hpp>

namespace {

constexpr size_t kArrayLength = 1 << 20; // 約100万要素

MTL::Buffer* newBufferFilledWithRandom(MTL::Device* pDevice) {
    MTL::Buffer* pBuffer = pDevice->newBuffer(kArrayLength * sizeof(float), MTL::ResourceStorageModeShared);
    float* pContents = static_cast<float*>(pBuffer->contents());
    for (size_t i = 0; i < kArrayLength; ++i) {
        pContents[i] = static_cast<float>(rand()) / static_cast<float>(RAND_MAX);
    }
    return pBuffer;
}

} // namespace

int main() {
    MTL::Device* pDevice = MTL::CreateSystemDefaultDevice();

    NS::Error* pError = nullptr;
    NS::String* pLibraryPath = NS::String::string("./add.metallib", NS::UTF8StringEncoding);
    MTL::Library* pLibrary = pDevice->newLibrary(NS::URL::fileURLWithPath(pLibraryPath), &pError);
    if (pLibrary == nullptr) {
        fprintf(stderr, "add.metallibの読み込みに失敗しました: %s\n", pError->localizedDescription()->utf8String());
        fprintf(stderr, "先に `just build-compute` でmetallibを生成してください\n");
        return 1;
    }

    MTL::Function* pAddFunction = pLibrary->newFunction(NS::String::string("add_arrays", NS::UTF8StringEncoding));
    MTL::ComputePipelineState* pPSO = pDevice->newComputePipelineState(pAddFunction, &pError);
    if (pPSO == nullptr) {
        fprintf(stderr, "パイプライン作成に失敗しました: %s\n", pError->localizedDescription()->utf8String());
        return 1;
    }

    MTL::CommandQueue* pQueue = pDevice->newCommandQueue();

    MTL::Buffer* pBufferA = newBufferFilledWithRandom(pDevice);
    MTL::Buffer* pBufferB = newBufferFilledWithRandom(pDevice);
    MTL::Buffer* pBufferResult = pDevice->newBuffer(kArrayLength * sizeof(float), MTL::ResourceStorageModeShared);

    MTL::CommandBuffer* pCmd = pQueue->commandBuffer();
    MTL::ComputeCommandEncoder* pEnc = pCmd->computeCommandEncoder();

    pEnc->setComputePipelineState(pPSO);
    pEnc->setBuffer(pBufferA, 0, 0);
    pEnc->setBuffer(pBufferB, 0, 1);
    pEnc->setBuffer(pBufferResult, 0, 2);

    NS::UInteger threadGroupSize = pPSO->maxTotalThreadsPerThreadgroup();
    if (threadGroupSize > kArrayLength) {
        threadGroupSize = kArrayLength;
    }
    pEnc->dispatchThreads(MTL::Size(kArrayLength, 1, 1), MTL::Size(threadGroupSize, 1, 1));
    pEnc->endEncoding();

    pCmd->commit();
    pCmd->waitUntilCompleted();

    const float* a = static_cast<const float*>(pBufferA->contents());
    const float* b = static_cast<const float*>(pBufferB->contents());
    const float* result = static_cast<const float*>(pBufferResult->contents());

    bool ok = true;
    for (size_t i = 0; i < kArrayLength; ++i) {
        float expected = a[i] + b[i];
        if (std::fabs(expected - result[i]) > 1e-5f) {
            fprintf(stderr, "不一致 at %zu: got %f, expected %f\n", i, result[i], expected);
            ok = false;
            break;
        }
    }
    if (ok) {
        printf("OK: %zu要素の加算結果がすべて一致しました\n", kArrayLength);
    }

    pBufferA->release();
    pBufferB->release();
    pBufferResult->release();
    pPSO->release();
    pAddFunction->release();
    pLibrary->release();
    pQueue->release();
    pDevice->release();

    return ok ? 0 : 1;
}
