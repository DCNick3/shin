#import types::{PosColVertex, FillUniformParams}

@group(0) @binding(0)
var<uniform> params: FillUniformParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex_main(input: PosColVertex) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = params.transform * vec4<f32>(input.position, 1.0);
    output.color = input.color;
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
