struct VertexIn {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

// TODO: this should be an include...
struct CameraParams {
    projectionMatrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> params: CameraParams;

@fragment
var<push_constant> color: vec4<f32>;

@vertex
fn vertex_main(input: VertexIn) -> VertexOutput {
    var output: VertexOutput;
    output.clip_position = params.projectionMatrix * vec4<f32>(input.position, 1.0);
    return output;
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return color;
}