use glam::{Mat4, Vec2, Vec3, Vec4};
use shin_render::{LayerBlendType, LayerFragmentShader};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TransformParams {
    pub transform: Mat4,
    pub some_vector: Vec3,
    pub another_vector: Vec2,
    pub wobble_translation: Vec2,
}

impl TransformParams {
    pub fn compute_final_transform(&self) -> Mat4 {
        // flip the Y coordinate
        // the game uses the D3D coordinate system (Y goes down), while wgpu uses the GL coordinate system (Y goes up)
        Mat4::from_scale(Vec3::new(1.0, -1.0, 1.0))
            * Mat4::orthographic_rh_gl(-960.0, 960.0, 540.0, -540.0, -1.0, 1.0)
            * Mat4::from_translation(self.wobble_translation.extend(0.0))
            * self.transform
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawableParams {
    pub color_multiplier: Vec4,
    pub blend_type: LayerBlendType,
    pub fragment_shader: LayerFragmentShader,
    pub shader_param: Vec4,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DrawableClipMode {
    /// Don't clip anything
    None,
    /// Clip, applying the [`TransformParams`] to the coordinates
    Clip,
    /// Clip, ignoring the [`TransformParams`] and using the screen coordinates directly
    ClipIgnoreTransform,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawableClipParams {
    pub mode: DrawableClipMode,
    /// xy - top left, zw - width height
    pub area: Vec4,
}
