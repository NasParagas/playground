// レンダーパイプラインの最小サンプル: ウィンドウなしで三角形をオフスクリーンテクスチャに描画し、
// getBytes()でCPU側にピクセルを読み出してPPM画像として書き出す。
//
// triangle.metalは事前に `xcrun -sdk macosx metal` でtriangle.metallibにコンパイルしておく必要がある
// (justfileのレシピを参照)。実行時にこのmain.cppと同じディレクトリのtriangle.metallibを読み込む。

#define NS_PRIVATE_IMPLEMENTATION
#define MTL_PRIVATE_IMPLEMENTATION

#include <cstdint>
#include <cstdio>
#include <vector>

#include <Metal/Metal.hpp>

namespace {

constexpr uint32_t kWidth = 512;
constexpr uint32_t kHeight = 512;

// triangle.metalのVertex(packed_float2 + packed_float3)とバイト単位で一致させる
struct Vertex {
    float position[2];
    float color[3];
};

void writePPM(const char* path, const uint8_t* bgra, uint32_t width, uint32_t height) {
    FILE* pFile = fopen(path, "wb");
    if (pFile == nullptr) {
        fprintf(stderr, "%sを開けませんでした\n", path);
        return;
    }
    fprintf(pFile, "P6\n%u %u\n255\n", width, height);
    for (uint32_t i = 0; i < width * height; ++i) {
        const uint8_t* pPixel = bgra + i * 4; // B, G, R, A
        const uint8_t rgb[3] = {pPixel[2], pPixel[1], pPixel[0]};
        fwrite(rgb, 1, 3, pFile);
    }
    fclose(pFile);
}

} // namespace

int main() {
    MTL::Device* pDevice = MTL::CreateSystemDefaultDevice();

    NS::Error* pError = nullptr;
    NS::String* pLibraryPath = NS::String::string("./triangle.metallib", NS::UTF8StringEncoding);
    MTL::Library* pLibrary = pDevice->newLibrary(NS::URL::fileURLWithPath(pLibraryPath), &pError);
    if (pLibrary == nullptr) {
        fprintf(stderr, "triangle.metallibの読み込みに失敗しました: %s\n", pError->localizedDescription()->utf8String());
        fprintf(stderr, "先に `just build-offscreen` でmetallibを生成してください\n");
        return 1;
    }

    MTL::Function* pVertexFn = pLibrary->newFunction(NS::String::string("vertex_main", NS::UTF8StringEncoding));
    MTL::Function* pFragmentFn = pLibrary->newFunction(NS::String::string("fragment_main", NS::UTF8StringEncoding));

    MTL::RenderPipelineDescriptor* pPipelineDesc = MTL::RenderPipelineDescriptor::alloc()->init();
    pPipelineDesc->setVertexFunction(pVertexFn);
    pPipelineDesc->setFragmentFunction(pFragmentFn);
    pPipelineDesc->colorAttachments()->object(0)->setPixelFormat(MTL::PixelFormatBGRA8Unorm);

    MTL::RenderPipelineState* pPSO = pDevice->newRenderPipelineState(pPipelineDesc, &pError);
    if (pPSO == nullptr) {
        fprintf(stderr, "パイプライン作成に失敗しました: %s\n", pError->localizedDescription()->utf8String());
        return 1;
    }
    pPipelineDesc->release();

    // レンダーターゲットになるオフスクリーンテクスチャ。
    // StorageModeSharedにしておくことでBlit経由の同期なしにgetBytes()で直接読める。
    MTL::TextureDescriptor* pTexDesc =
            MTL::TextureDescriptor::texture2DDescriptor(MTL::PixelFormatBGRA8Unorm, kWidth, kHeight, false);
    pTexDesc->setUsage(MTL::TextureUsageRenderTarget | MTL::TextureUsageShaderRead);
    pTexDesc->setStorageMode(MTL::StorageModeShared);
    MTL::Texture* pTargetTexture = pDevice->newTexture(pTexDesc);
    pTexDesc->release();

    const Vertex vertices[] = {
            {{0.0f, 0.5f}, {1.0f, 0.0f, 0.0f}},
            {{-0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
            {{0.5f, -0.5f}, {0.0f, 0.0f, 1.0f}},
    };
    MTL::Buffer* pVertexBuffer = pDevice->newBuffer(vertices, sizeof(vertices), MTL::ResourceStorageModeShared);

    MTL::RenderPassDescriptor* pRenderPass = MTL::RenderPassDescriptor::alloc()->init();
    MTL::RenderPassColorAttachmentDescriptor* pColorAttachment = pRenderPass->colorAttachments()->object(0);
    pColorAttachment->setTexture(pTargetTexture);
    pColorAttachment->setLoadAction(MTL::LoadActionClear);
    pColorAttachment->setClearColor(MTL::ClearColor(0.05, 0.05, 0.08, 1.0));
    pColorAttachment->setStoreAction(MTL::StoreActionStore);

    MTL::CommandQueue* pQueue = pDevice->newCommandQueue();
    MTL::CommandBuffer* pCmd = pQueue->commandBuffer();
    MTL::RenderCommandEncoder* pEnc = pCmd->renderCommandEncoder(pRenderPass);

    pEnc->setRenderPipelineState(pPSO);
    pEnc->setVertexBuffer(pVertexBuffer, 0, 0);
    pEnc->drawPrimitives(MTL::PrimitiveTypeTriangle, NS::UInteger(0), NS::UInteger(3));
    pEnc->endEncoding();

    pCmd->commit();
    pCmd->waitUntilCompleted();

    std::vector<uint8_t> pixels(kWidth * kHeight * 4);
    pTargetTexture->getBytes(pixels.data(), kWidth * 4, MTL::Region::Make2D(0, 0, kWidth, kHeight), 0);

    const char* pOutPath = "./triangle.ppm";
    writePPM(pOutPath, pixels.data(), kWidth, kHeight);
    printf("%sに書き出しました (%ux%u)\n", pOutPath, kWidth, kHeight);

    pVertexBuffer->release();
    pRenderPass->release();
    pTargetTexture->release();
    pPSO->release();
    pVertexFn->release();
    pFragmentFn->release();
    pLibrary->release();
    pQueue->release();
    pDevice->release();

    return 0;
}
