#import types::{PosTexVertex, LayerUniformParams}
#import utils::evaluate_fragment_shader

@group(0) @binding(0)
var<uniform> params: LayerUniformParams;

@group(0) @binding(1)
var texture_texture: texture_2d<f32>;
@group(0) @binding(2)
var texture_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_position: vec2<f32>,
}

@vertex
fn vertex_main(input: PosTexVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.texture_position = input.texture_position;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(texture_texture, texture_sampler, input.texture_position);

    if params.output_type == 2 {
        // discard
        if sampled.w * params.color.w - 0.00100000005 < 0.0 {
            discard;
        }
    }

    let tinted = sampled * params.color;
    // the fragment shader won't change the alpha channel
    let value = evaluate_fragment_shader(tinted.xyz, params.fragment_operation, params.fragment_param);
    let processed = vec4<f32>(value, tinted.w);

    if params.output_type == 0 || params.output_type == 2 {
        // normal & discard
        return processed;
    } else if params.output_type == 1 {
        // premultiply
        return processed * vec4<f32>(processed.www, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
