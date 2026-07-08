// AppKit + metal-cppでウィンドウに三角形を描画するサンプル。
//
// このファイルは本物のCocoa/QuartzCore/Metal(Objective-C)ヘッダーだけを使い、
// metal-cppのヘッダーは一切includeしない。実際のMetal描画コマンドの構築はRenderer.cpp
// (metal-cpp専用の翻訳単位)に分離してあり、境界はRenderer.hppのvoid*インターフェース経由。
// 分離している理由はRenderer.hppのコメント参照
// (本物のFoundationヘッダーとmetal-cppのFoundation.hppを同じ翻訳単位でincludeすると、
//  NSBundleDidLoadNotification等のグローバル定数の型がぶつかってコンパイルエラーになるため)。

#import <Cocoa/Cocoa.h>
#import <Metal/Metal.h>
#import <QuartzCore/QuartzCore.h>

#include "Renderer.hpp"

namespace {
constexpr CGFloat kWindowWidth = 800;
constexpr CGFloat kWindowHeight = 600;
} // namespace

@interface MetalView : NSView
@end

@implementation MetalView {
    Renderer* _pRenderer;
    NSTimer* _timer;
}

+ (Class)layerClass {
    return [CAMetalLayer class];
}

- (CALayer*)makeBackingLayer {
    return [CAMetalLayer layer];
}

- (instancetype)initWithFrame:(NSRect)frameRect {
    // [debug] NSLog(@"MetalView initWithFrame開始");
    self = [super initWithFrame:frameRect];
    if (self) {
        self.wantsLayer = YES;
        // +layerClass/makeBackingLayerのオーバーライドだけでは反映されない環境があったため、念のため直接代入する
        if (![self.layer isKindOfClass:[CAMetalLayer class]]) {
            self.layer = [CAMetalLayer layer];
        }
        // [debug] NSLog(@"wantsLayer設定後: self.layer=%@", self.layer);

        // [debug] NSLog(@"RendererCreate前");
        _pRenderer = RendererCreate();
        // [debug] NSLog(@"RendererCreate後: pRenderer=%p", (void*)_pRenderer);

        CAMetalLayer* layer = (CAMetalLayer*)self.layer;
        layer.device = (__bridge id<MTLDevice>)RendererGetDevice(_pRenderer);
        layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
        layer.framebufferOnly = YES;
        // [debug] NSLog(@"layer設定後: device=%@", layer.device);

        // CVDisplayLinkの代わりに、シンプルさ優先で約60fpsのNSTimerで再描画する
        _timer = [NSTimer scheduledTimerWithTimeInterval:1.0 / 60.0
                                                   target:self
                                                 selector:@selector(render)
                                                 userInfo:nil
                                                  repeats:YES];
        // [debug] NSLog(@"MetalView initWithFrame完了");
    }
    return self;
}

- (void)viewDidMoveToWindow {
    [super viewDidMoveToWindow];
    [self resizeDrawable];
}

- (void)setFrameSize:(NSSize)newSize {
    [super setFrameSize:newSize];
    [self resizeDrawable];
}

- (void)resizeDrawable {
    CGFloat scale = self.window ? self.window.backingScaleFactor : 1.0;
    CAMetalLayer* layer = (CAMetalLayer*)self.layer;
    layer.contentsScale = scale;

    CGSize drawableSize = self.bounds.size;
    drawableSize.width *= scale;
    drawableSize.height *= scale;
    layer.drawableSize = drawableSize;
}

- (void)render {
    RendererDraw(_pRenderer, (__bridge void*)self.layer);
}

- (void)dealloc {
    [_timer invalidate];
    RendererDestroy(_pRenderer);
}

@end

@interface AppDelegate : NSObject <NSApplicationDelegate>
@end

@implementation AppDelegate {
    NSWindow* _window;
    MetalView* _view;
}

- (void)applicationDidFinishLaunching:(NSNotification*)notification {
    // [debug] NSLog(@"applicationDidFinishLaunching開始");

    NSRect frame = NSMakeRect(0, 0, kWindowWidth, kWindowHeight);
    _window = [[NSWindow alloc]
            initWithContentRect:frame
                      styleMask:(NSWindowStyleMaskTitled | NSWindowStyleMaskClosable | NSWindowStyleMaskResizable)
                        backing:NSBackingStoreBuffered
                          defer:NO];
    _window.releasedWhenClosed = NO;
    [_window setTitle:@"metal-cpp: window triangle"];
    [_window center];

    _view = [[MetalView alloc] initWithFrame:frame];
    [_window setContentView:_view];

    // 念のため定番の「フロントに出す」呼び出しを重ねがけする
    [_window makeKeyAndOrderFront:nil];
    [_window orderFrontRegardless];
    [NSApp activateIgnoringOtherApps:YES];
    [[NSRunningApplication currentApplication] activateWithOptions:NSApplicationActivateIgnoringOtherApps];

    // [debug] NSLog(@"window作成完了: frame=%@ isVisible=%d isKeyWindow=%d isMiniaturized=%d",
    // [debug]       NSStringFromRect(_window.frame), _window.isVisible, _window.isKeyWindow, _window.isMiniaturized);
}

- (BOOL)applicationShouldTerminateAfterLastWindowClosed:(NSApplication*)sender {
    return YES;
}

@end

int main(int argc, const char* argv[]) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyRegular];
        // [debug] NSLog(@"setActivationPolicy成功=%d", policyOK);

        AppDelegate* pDelegate = [[AppDelegate alloc] init];
        [NSApp setDelegate:pDelegate];

        // [debug] NSLog(@"NSApp run開始");
        [NSApp run];
        // [debug] NSLog(@"NSApp run終了");
    }
    return 0;
}
