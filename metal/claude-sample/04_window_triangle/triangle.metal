#include <metal_stdlib>
using namespace metal;

struct Vertex {
    packed_float2 position;
    packed_float3 color;
};

struct RasterizerData {
    float4 position [[position]];
    float3 color;
};

vertex RasterizerData vertex_main(
        device const Vertex* vertices [[buffer(0)]],
        uint vertexID [[vertex_id]])
{
    RasterizerData out;
    out.position = float4(vertices[vertexID].position, 0.0, 1.0);
    out.color = vertices[vertexID].color;
    return out;
}

fragment float4 fragment_main(RasterizerData in [[stage_in]])
{
    return float4(in.color, 1.0);
}
