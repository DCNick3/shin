#import types::{MovieVertex, MovieUniformParams}

@group(0) @binding(0)
var<uniform> params: MovieUniformParams;

// this will be detected as out build script as a single texture bind group
// we can only bind these in an isolated bind group!
@group(0) @binding(1)
var luma_texture: texture_2d<f32>;
@group(0) @binding(2)
var luma_sampler: sampler;
@group(0) @binding(3)
var chroma_texture: texture_2d<f32>;
@group(0) @binding(4)
var chroma_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
}

@vertex
fn vertex_main(input: MovieVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.coords.xy, 0.0, 1.0);
    output.texture_position = input.coords.zw;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let luma = textureSample(luma_texture, luma_sampler, input.texture_position).x;
    let chroma = textureSample(chroma_texture, chroma_sampler, input.texture_position).xy;

    let biased_yuv = vec3<f32>(luma, chroma.x - 0.5, chroma.y - 0.5) - params.color_bias.xyz;

    let r = dot(biased_yuv, params.color_transform[0].xyz);
    let g = dot(biased_yuv, params.color_transform[1].xyz);
    let b = dot(biased_yuv, params.color_transform[2].xyz);

    return vec4<f32>(r, g, b, 1.0);
}
