#import types::{PosVertex, ClearUniformParams}

@group(0) @binding(0)
var<uniform> params: ClearUniformParams;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@vertex
fn vertex_main(input: PosVertex) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = vec4<f32>(input.position, 1.0);
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return params.color;
}
