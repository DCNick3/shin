#import types::{LayerVertex, LayerUniformParams}

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
fn vertex_main(input: LayerVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.coords.xy, 0.0, 1.0);
    output.texture_position = input.coords.zw;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var value = textureSample(texture_texture, texture_sampler, input.texture_position);

    if params.output_type == 1 {
        if value.w * params.color.w - 0.00100000005 < 0.0 {
            discard;
        }
    }

    value = value * params.color;

    if params.fragment_operation == 0 {
        // default
    } else if params.fragment_operation == 1 {
        // mono
        let wet = params.fragment_param.w;
        let dry = 1.0 - wet;

        let luma = dot(value.xyz, vec3(0.298999995, 0.587000012, 0.114));
        let mix = vec3(luma) * wet + value.xyz * dry;
        value = vec4<f32>(mix * params.fragment_param.xyz, value.w);
    } else if params.fragment_operation == 2 {
        // fill
        let wet = params.fragment_param.w;
        let dry = 1.0 - wet;

        let mix = params.fragment_param.xyz * wet + value.xyz * dry;
        value = vec4<f32>(mix, value.w);
    } else if params.fragment_operation == 3 {
        // fill2
        // I have no idea how it is different from default
    } else if params.fragment_operation == 4 {
        // negative
        let wet = params.fragment_param.w;
        let dry = 1.0 - wet;

        let negated = 1 - value.xyz;
        let mix = negated * wet + value.xyz * dry;
        value = vec4<f32>(mix, value.w);
    } else if params.fragment_operation == 5 {
        // gamma
        let corrected = exp2(log2(value.xyz) * 1 / params.fragment_param.xyz);
        value = vec4<f32>(corrected, value.w);
    }

    if params.output_type == 0 || params.output_type == 2 {
        // normal & discard
        return value;
    } else if params.output_type == 1 {
        // premultiply
        return value * vec4<f32>(value.www, 1.0);
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}
