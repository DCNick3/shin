#import types::{LayerVertex, WiperDefaultUniformParams}

@group(0) @binding(0)
var<uniform> params: WiperDefaultUniformParams;

@group(0) @binding(1)
var source_texture: texture_2d<f32>;
@group(0) @binding(2)
var source_sampler: sampler;
@group(0) @binding(3)
var target_texture: texture_2d<f32>;
@group(0) @binding(4)
var target_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
}

@vertex
fn vertex_main(input: LayerVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.coords.xy, 0.0, 1.0);
    output.texture_position = input.coords.zw;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled_source = textureSample(source_texture, source_sampler, input.texture_position);
    let sampled_target = textureSample(target_texture, target_sampler, input.texture_position);

    let alpha = params.alpha.x;

    let mixed = mix(sampled_source, sampled_target, alpha);

    return mixed;
}
