#import types::{TextVertex, FontBorderUniformParams}

@group(0) @binding(0)
var<uniform> params: FontBorderUniformParams;

@group(0) @binding(1)
var glyph_texture: texture_2d<f32>;
@group(0) @binding(2)
var glyph_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // can't use an array here because
    // > WGSL Spec, 12.3.1.2. User-defined Inputs and Outputs (https://www.w3.org/TR/WGSL/#user-defined-inputs-outputs)
    // > Each user-defined input datum and user-defined output datum must:
    // >  - be of numeric scalar type or numeric vector type.
    //
    @location(0) texture_pos0: vec2<f32>,
    @location(1) texture_pos1: vec2<f32>,
    @location(2) texture_pos2: vec2<f32>,
    @location(3) texture_pos3: vec2<f32>,
    @location(4) texture_pos4: vec2<f32>,
    @location(5) texture_pos5: vec2<f32>,
    @location(6) texture_pos6: vec2<f32>,
    @location(7) texture_pos7: vec2<f32>,
}

@vertex
fn vertex_main(input: TextVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.position.xy, 0.0, 1.0);

    let base_tex_position = input.position.zw;
    // programming GPUs is very fun
    output.texture_pos0 = base_tex_position + params.dist[0].xy;
    output.texture_pos1 = base_tex_position + params.dist[0].zw;
    output.texture_pos2 = base_tex_position + params.dist[1].xy;
    output.texture_pos3 = base_tex_position + params.dist[1].zw;
    output.texture_pos4 = base_tex_position + params.dist[2].xy;
    output.texture_pos5 = base_tex_position + params.dist[2].zw;
    output.texture_pos6 = base_tex_position + params.dist[3].xy;
    output.texture_pos7 = base_tex_position + params.dist[3].zw;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let tint = params.color;

    // programming GPUs is extremely fun
    let max_sampled = max(
        max(
            max(
                textureSample(glyph_texture, glyph_sampler, input.texture_pos0).x,
                textureSample(glyph_texture, glyph_sampler, input.texture_pos1).x,
            ),
            max(
                textureSample(glyph_texture, glyph_sampler, input.texture_pos2).x,
                textureSample(glyph_texture, glyph_sampler, input.texture_pos3).x,
            ),
        ),
        max(
            max(
                textureSample(glyph_texture, glyph_sampler, input.texture_pos4).x,
                textureSample(glyph_texture, glyph_sampler, input.texture_pos5).x,
            ),
            max(
                textureSample(glyph_texture, glyph_sampler, input.texture_pos6).x,
                textureSample(glyph_texture, glyph_sampler, input.texture_pos7).x,
            ),
        ),
    );

    let sampled = min(1.0, 1.5 * max_sampled);

    return tint * vec4(1.0, 1.0, 1.0, sampled);
}
