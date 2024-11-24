#import types::{TextVertex, FontUniformParams}

@group(0) @binding(0)
var<uniform> params: FontUniformParams;

@group(0) @binding(1)
var glyph_texture: texture_2d<f32>;
@group(0) @binding(2)
var glyph_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: f32,
    @location(1) texture_position: vec2<f32>,
}

@vertex
fn vertex_main(input: TextVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.position.xy, 0.0, 1.0);
    output.color = input.color;
    output.texture_position = input.position.zw;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let color1 = params.color1_;
    let color2 = params.color2_;

    let tint = input.color * (color2 - color1) + color1;

    let sampled = textureSample(glyph_texture, glyph_sampler, input.texture_position).x;

    return tint * vec4(sampled, sampled, sampled, 1.0);
}
