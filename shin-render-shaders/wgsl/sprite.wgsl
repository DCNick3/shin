#import types::{PosColTexVertex, SpriteUniformParams}

@group(0) @binding(0)
var<uniform> params: SpriteUniformParams;

// this will be detected as out build script as a single texture
// the binding names should have the same prefix and end with _texture and _sampler
@group(0) @binding(1)
var sprite_texture: texture_2d<f32>;
@group(0) @binding(2)
var sprite_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_position: vec2<f32>,
}

@vertex
fn vertex_main(input: PosColTexVertex) -> VertexOutput {
    var output: VertexOutput;

    output.clip_position = params.transform * vec4<f32>(input.position, 1.0);
    output.color = input.color.zyxw; // funny swizzing...
    output.texture_position = input.texture_position;

    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sampled = textureSample(sprite_texture, sprite_sampler, input.texture_position);

    return sampled * input.color;
}
