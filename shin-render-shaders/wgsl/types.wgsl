struct PosVertex {
    @location(0) position: vec3<f32>,
}

struct PosColVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

struct PosColTexVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_position: vec2<f32>,
}

struct TextVertex {
    @location(0) position: vec4<f32>,
    @location(1) color: f32,
}

struct BlendVertex {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_position: vec4<f32>,
}

struct WindowVertex {
    @location(0) position: vec4<f32>,
    @location(1) texture_position: vec4<f32>,
}

struct LayerVertex {
    @location(0) position: vec4<f32>,
}

struct MaskVertex {
    @location(0) position: vec2<f32>,
    @location(1) texture_position: vec4<f32>,
}

struct ClearUniformParams {
    color: vec4<f32>,
}

struct FillUniformParams {
    transform: mat4x4<f32>,
}

struct SpriteUniformParams {
    transform: mat4x4<f32>,
}

struct FontUniformParams {
    transform: mat4x4<f32>,
    color1_: vec4<f32>,
    color2_: vec4<f32>,
}

