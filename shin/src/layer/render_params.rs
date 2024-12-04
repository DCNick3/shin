use bitflags::bitflags;
use glam::{vec3, Mat4, Vec2, Vec3, Vec4};
use shin_render::{shaders::types::vertices::FloatColor4, LayerBlendType, LayerFragmentShader};

bitflags! {
    #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
    pub struct ComposeFlags: i32 {
        const IGNORE_CAMERA_POSITION = 0b00000001;
        const DONT_INHERIT_TRANSFORM = 0b00000010;
        const DONT_INHERIT_WOBBLE = 0b00000100;
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct TransformParams {
    pub transform: Mat4,
    /// Some kind of origin. Maybe camera position?
    pub camera_position: Vec3,
    /// This translation is always inherited by children, but not applied to the layer itself
    pub unconditionally_inherited_translation: Vec2,
    /// This is a translation coming from the parent wobbler and its inheritance can be controlled with FLAG_2
    pub wobble_translation: Vec2,
}

impl TransformParams {
    pub fn compose_with(&mut self, composed_with: &Self, flags: ComposeFlags) {
        let some_origin = if flags.contains(ComposeFlags::IGNORE_CAMERA_POSITION) {
            Vec3::ZERO
        } else {
            self.camera_position
        };

        // extract distance from the camera and apply scaling based on it
        let z_distance = (self.transform.w_axis.z - some_origin.z) * 0.001;

        let distance_scale = if z_distance <= 0.0 {
            0.0
        } else {
            self.transform = Mat4::from_translation((-some_origin).with_z(1.0)) * self.transform;

            1.0 / z_distance
        };

        self.transform =
            Mat4::from_scale(vec3(distance_scale, distance_scale, 1.0)) * self.transform;
        // the z translation has served its purpose, zero it out
        self.transform.w_axis.z = 0.0;

        if !flags.contains(ComposeFlags::DONT_INHERIT_TRANSFORM) {
            self.transform = composed_with.transform * self.transform;
        }
        if !flags.contains(ComposeFlags::DONT_INHERIT_WOBBLE) {
            self.transform = Mat4::from_translation(composed_with.wobble_translation.extend(0.0))
                * self.transform;
        }

        self.transform =
            Mat4::from_translation(self.unconditionally_inherited_translation.extend(0.0))
                * self.transform;
    }

    pub fn compute_final_transform(&self) -> Mat4 {
        // flip the Y coordinate
        // the game uses the D3D coordinate system (Y goes down), while wgpu uses the GL coordinate system (Y goes up)
        Mat4::from_scale(vec3(1.0, -1.0, 1.0))
            * Mat4::orthographic_rh_gl(-960.0, 960.0, 540.0, -540.0, -1.0, 1.0)
            * Mat4::from_translation(self.wobble_translation.extend(0.0))
            * self.transform
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct DrawableParams {
    pub color_multiplier: FloatColor4,
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
