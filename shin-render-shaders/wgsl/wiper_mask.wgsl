#import types::{MaskVertex, WiperMaskUniformParams}
#import utils::map

@group(0) @binding(0)
var<uniform> params: WiperMaskUniformParams;

@group(0) @binding(1)
var source_texture: texture_2d<f32>;
@group(0) @binding(2)
var source_sampler: sampler;
@group(0) @binding(3)
var target_texture: texture_2d<f32>;
@group(0) @binding(4)
var target_sampler: sampler;
@group(0) @binding(5)
var mask_texture: texture_2d<f32>;
@group(0) @binding(6)
var mask_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
    @location(1) mask_position: vec2<f32>,
}

@vertex
fn vertex_main(input: MaskVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.texture_position = input.texture_position;
    output.mask_position = input.mask_position;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled_source = textureSample(source_texture, source_sampler, input.texture_position);
    let sampled_target = textureSample(target_texture, target_sampler, input.texture_position);

    let sampled_mask = textureSample(mask_texture, mask_sampler, input.mask_position).x;

    let remapped_mask = 1.0 - clamp(map(sampled_mask, params.minmax.xy), 0.0, 1.0);

    let mixed = mix(sampled_source, sampled_target, remapped_mask);

    return mixed;
}
