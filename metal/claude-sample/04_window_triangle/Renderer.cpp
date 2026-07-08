// metal-cppを使う側の翻訳単位。本物のCocoa/QuartzCoreヘッダーは絶対にincludeしない
// (理由はRenderer.hppのコメント参照)。NS_PRIVATE_IMPLEMENTATION等の定義もこのファイルだけに置く。

#define NS_PRIVATE_IMPLEMENTATION
#define CA_PRIVATE_IMPLEMENTATION
#define MTL_PRIVATE_IMPLEMENTATION

#include "Renderer.hpp"

#include <cstdio>
#include <cstdlib>

#include <Metal/Metal.hpp>
#include <QuartzCore/QuartzCore.hpp>

namespace {

// triangle.metalのVertex(packed_float2 + packed_float3)とバイト単位で一致させる
struct Vertex {
    float position[2];
    float color[3];
};

} // namespace

class Renderer {
public:
    Renderer() {
        // [debug] fprintf(stderr, "[Renderer] CreateSystemDefaultDevice前\n");
        // [debug] fflush(stderr);
        _pDevice = MTL::CreateSystemDefaultDevice();
        // [debug] fprintf(stderr, "[Renderer] CreateSystemDefaultDevice後: pDevice=%p name=%s\n", (void*)_pDevice,
        // [debug]         _pDevice ? _pDevice->name()->utf8String() : "(null)");
        // [debug] fflush(stderr);

        _pCommandQueue = _pDevice->newCommandQueue();
        // [debug] fprintf(stderr, "[Renderer] newCommandQueue後: pQueue=%p\n", (void*)_pCommandQueue);
        // [debug] fflush(stderr);

        buildPipeline();
        // [debug] fprintf(stderr, "[Renderer] buildPipeline完了\n");
        // [debug] fflush(stderr);

        buildBuffers();
        // [debug] fprintf(stderr, "[Renderer] buildBuffers完了\n");
        // [debug] fflush(stderr);
    }

    ~Renderer() {
        _pVertexBuffer->release();
        _pPSO->release();
        _pCommandQueue->release();
        _pDevice->release();
    }

    MTL::Device* device() const {
        return _pDevice;
    }

    void draw(CA::MetalLayer* pLayer) {
        NS::AutoreleasePool* pPool = NS::AutoreleasePool::alloc()->init();

        CA::MetalDrawable* pDrawable = pLayer->nextDrawable();
        if (pDrawable == nullptr) {
            pPool->release();
            return;
        }

        MTL::RenderPassDescriptor* pRenderPass = MTL::RenderPassDescriptor::alloc()->init();
        MTL::RenderPassColorAttachmentDescriptor* pColorAttachment = pRenderPass->colorAttachments()->object(0);
        pColorAttachment->setTexture(pDrawable->texture());
        pColorAttachment->setLoadAction(MTL::LoadActionClear);
        pColorAttachment->setClearColor(MTL::ClearColor(0.05, 0.05, 0.08, 1.0));
        pColorAttachment->setStoreAction(MTL::StoreActionStore);

        MTL::CommandBuffer* pCmd = _pCommandQueue->commandBuffer();
        MTL::RenderCommandEncoder* pEnc = pCmd->renderCommandEncoder(pRenderPass);
        pEnc->setRenderPipelineState(_pPSO);
        pEnc->setVertexBuffer(_pVertexBuffer, 0, 0);
        pEnc->drawPrimitives(MTL::PrimitiveTypeTriangle, NS::UInteger(0), NS::UInteger(3));
        pEnc->endEncoding();

        pCmd->presentDrawable(pDrawable);
        pCmd->commit();

        pRenderPass->release();
        pPool->release();
    }

private:
    void buildPipeline() {
        // [debug] fprintf(stderr, "[Renderer]   newLibrary前\n");
        // [debug] fflush(stderr);
        NS::Error* pError = nullptr;
        NS::String* pLibraryPath = NS::String::string("./triangle.metallib", NS::UTF8StringEncoding);
        MTL::Library* pLibrary = _pDevice->newLibrary(NS::URL::fileURLWithPath(pLibraryPath), &pError);
        if (pLibrary == nullptr) {
            fprintf(stderr, "triangle.metallibの読み込みに失敗しました: %s\n",
                    pError->localizedDescription()->utf8String());
            fprintf(stderr, "先に `just build-window` でmetallibを生成してください\n");
            exit(1);
        }
        // [debug] fprintf(stderr, "[Renderer]   newLibrary後\n");
        // [debug] fflush(stderr);

        MTL::Function* pVertexFn = pLibrary->newFunction(NS::String::string("vertex_main", NS::UTF8StringEncoding));
        MTL::Function* pFragmentFn =
                pLibrary->newFunction(NS::String::string("fragment_main", NS::UTF8StringEncoding));
        // [debug] fprintf(stderr, "[Renderer]   newFunction後: vertexFn=%p fragmentFn=%p\n", (void*)pVertexFn,
        // [debug]         (void*)pFragmentFn);
        // [debug] fflush(stderr);

        MTL::RenderPipelineDescriptor* pDesc = MTL::RenderPipelineDescriptor::alloc()->init();
        pDesc->setVertexFunction(pVertexFn);
        pDesc->setFragmentFunction(pFragmentFn);
        pDesc->colorAttachments()->object(0)->setPixelFormat(MTL::PixelFormatBGRA8Unorm);

        // [debug] fprintf(stderr, "[Renderer]   newRenderPipelineState前\n");
        // [debug] fflush(stderr);
        _pPSO = _pDevice->newRenderPipelineState(pDesc, &pError);
        if (_pPSO == nullptr) {
            fprintf(stderr, "パイプライン作成に失敗しました: %s\n", pError->localizedDescription()->utf8String());
            exit(1);
        }
        // [debug] fprintf(stderr, "[Renderer]   newRenderPipelineState後\n");
        // [debug] fflush(stderr);

        pDesc->release();
        pVertexFn->release();
        pFragmentFn->release();
        pLibrary->release();
    }

    void buildBuffers() {
        const Vertex vertices[] = {
                {{0.0f, 0.5f}, {1.0f, 0.0f, 0.0f}},
                {{-0.5f, -0.5f}, {0.0f, 1.0f, 0.0f}},
                {{0.5f, -0.5f}, {0.0f, 0.0f, 1.0f}},
        };
        _pVertexBuffer = _pDevice->newBuffer(vertices, sizeof(vertices), MTL::ResourceStorageModeShared);
    }

    MTL::Device* _pDevice;
    MTL::CommandQueue* _pCommandQueue;
    MTL::RenderPipelineState* _pPSO;
    MTL::Buffer* _pVertexBuffer;
};

Renderer* RendererCreate() {
    return new Renderer();
}

void RendererDestroy(Renderer* pRenderer) {
    delete pRenderer;
}

void* RendererGetDevice(Renderer* pRenderer) {
    return reinterpret_cast<void*>(pRenderer->device());
}

void RendererDraw(Renderer* pRenderer, void* pMetalLayer) {
    pRenderer->draw(reinterpret_cast<CA::MetalLayer*>(pMetalLayer));
}
