use enum_map::{enum_map, EnumMap};
use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec4};
use shin_core::{
    time::{Ticks, Tweener},
    vm::command::types::{LayerId, LayerProperty},
};
use shin_render::{shaders::types::vertices::FloatColor4, LayerBlendType, LayerFragmentShader};

use crate::{
    layer::{
        render_params::{
            ComposeFlags, DrawableClipMode, DrawableClipParams, DrawableParams, TransformParams,
        },
        wobbler::Wobbler,
    },
    update::{Updatable, UpdateContext},
};

fn initial_values() -> EnumMap<LayerProperty, i32> {
    enum_map! {
        v => v.initial_value()
    }
}

#[derive(Debug, Clone)]
pub struct LayerProperties {
    layer_load_counter1: u32,
    layer_id: LayerId,
    properties: EnumMap<LayerProperty, Tweener>,
    wobbler_x: Wobbler,
    wobbler_y: Wobbler,
    wobbler_alpha: Wobbler,
    wobbler_rotation: Wobbler,
    wobbler_scale_x: Wobbler,
    wobbler_scale_y: Wobbler,
}

impl LayerProperties {
    pub fn new() -> Self {
        Self {
            layer_load_counter1: 0,
            layer_id: LayerId::new(0),
            properties: initial_values().map(|_, v| Tweener::new(v as f32)),
            wobbler_x: Wobbler::new(),
            wobbler_y: Wobbler::new(),
            wobbler_alpha: Wobbler::new(),
            wobbler_rotation: Wobbler::new(),
            wobbler_scale_x: Wobbler::new(),
            wobbler_scale_y: Wobbler::new(),
        }
    }

    pub fn get_value(&self, property: LayerProperty) -> f32 {
        self.properties[property].value()
    }
    #[allow(unused)]
    pub fn property_tweener(&self, property: LayerProperty) -> &Tweener {
        &self.properties[property]
    }

    pub fn property_tweener_mut(&mut self, property: LayerProperty) -> &mut Tweener {
        &mut self.properties[property]
    }

    pub fn init(&mut self) {
        for (prop, val) in initial_values() {
            self.properties[prop].fast_forward_to(val as f32);
        }
    }

    pub fn get_layer_id(&self) -> LayerId {
        self.layer_id
    }

    pub fn set_layer_id(&mut self, layer_id: LayerId) {
        self.layer_id = layer_id;
    }

    pub fn set_layerload_counter1(&mut self, layer_load_counter1: u32) {
        self.layer_load_counter1 = layer_load_counter1;
    }

    fn evaluate_wobbler(
        &self,
        wobbler: &Wobbler,
        amplitude: LayerProperty,
        bias: LayerProperty,
        scale: f32,
        fallback: f32,
    ) -> f32 {
        if let Some(value) = wobbler.value_opt() {
            (value * self.get_value(amplitude) + self.get_value(bias)) * scale
        } else {
            fallback
        }
    }

    fn get_effective_alpha(&self) -> f32 {
        use LayerProperty::*;

        let base = self.get_value(MulColorAlpha) * 0.001;

        base * self.evaluate_wobbler(
            &self.wobbler_alpha,
            WobbleAlphaAmplitude,
            WobbleAlphaBias,
            0.001,
            1.0,
        )
    }

    pub fn is_visible(&self) -> bool {
        use LayerProperty::*;

        if self.get_value(ShowLayer) as i32 == 0
            || self.get_value(ScaleX) as i32 == 0
            || self.get_value(ScaleY) as i32 == 0
        {
            return false;
        }

        self.get_effective_alpha() > 0.0
    }

    pub fn get_color_multiplier(&self) -> FloatColor4 {
        use LayerProperty::*;

        // NB: we initially multiply the color channels by 0.0005 instead of 0.001
        // this is to allow values between 0 and 2000 to be passed through the clamp below
        // the range is later restored, so in the end our color channel multipliers are [0.0; 2.0]
        let r = self.get_value(MulColorRed) * 0.0005;
        let g = self.get_value(MulColorGreen) * 0.0005;
        let b = self.get_value(MulColorBlue) * 0.0005;
        let a = self.get_effective_alpha();

        let mut color = vec4(r, g, b, a);
        color = color.clamp(Vec4::splat(0.0), Vec4::splat(1.0));
        // restore the original range
        color *= vec4(2.0, 2.0, 2.0, 1.0);

        FloatColor4::from_vec4(color)
    }

    pub fn get_blend_type(&self) -> LayerBlendType {
        use LayerProperty::*;

        let value = self.get_value(BlendType) as i32;
        match value {
            0 => LayerBlendType::Type1,
            1 => LayerBlendType::Type2,
            2 => LayerBlendType::Type3,
            _ => LayerBlendType::Type1,
        }
    }

    pub fn get_fragment_shader(&self) -> LayerFragmentShader {
        use LayerProperty::*;

        let value = self.get_value(FragmentShader) as i32;
        match value {
            0 => LayerFragmentShader::Default,
            1 => LayerFragmentShader::Mono,
            2 => LayerFragmentShader::Fill,
            3 => LayerFragmentShader::Fill2,
            4 => LayerFragmentShader::Negative,
            5 => LayerFragmentShader::Gamma,
            _ => LayerFragmentShader::Default,
        }
    }

    pub fn get_fragment_shader_param(&self) -> Vec4 {
        use LayerProperty::*;

        vec4(
            self.get_value(ShaderParamX) * 0.001,
            self.get_value(ShaderParamY) * 0.001,
            self.get_value(ShaderParamZ) * 0.001,
            self.get_value(ShaderParamW) * 0.001,
        )
    }

    pub fn is_fragment_shader_nontrivial(&self) -> bool {
        if self.get_color_multiplier() != FloatColor4::WHITE {
            return true;
        }

        let fragment_shader = self.get_fragment_shader();
        let shader_param = self.get_fragment_shader_param();

        !fragment_shader.is_equivalent_to_default(shader_param)
    }

    pub fn is_blending_nontrivial(&self) -> bool {
        use LayerProperty::*;

        // NB: not using get_effective_alpha here, wobbler alpha is not taken into account
        // this might be a bug in the original code
        self.get_value(MulColorAlpha) * 0.001 < 1.0
            || self.get_blend_type() != LayerBlendType::Type1
    }

    pub fn get_drawable_params(&self) -> DrawableParams {
        use LayerProperty::*;

        let color_multiplier = self.get_color_multiplier();
        let blend_type = self.get_blend_type();
        let fragment_shader = self.get_fragment_shader();

        let shader_param = self.get_fragment_shader_param();

        DrawableParams {
            color_multiplier,
            blend_type,
            fragment_shader,
            shader_param,
        }
    }

    pub fn get_clip_mode(&self) -> DrawableClipMode {
        use LayerProperty::*;

        let value = self.get_value(ClipMode) as i32;
        match value {
            0 => DrawableClipMode::None,
            1 => DrawableClipMode::Clip,
            2 => DrawableClipMode::ClipIgnoreTransform,
            _ => DrawableClipMode::None,
        }
    }

    pub fn get_clip_params(&self) -> DrawableClipParams {
        use LayerProperty::*;

        let mode = self.get_clip_mode();
        let from_x = self.get_value(ClipFromX);
        let to_x = self.get_value(ClipToX);
        let from_y = self.get_value(ClipFromY);
        let to_y = self.get_value(ClipToY);

        let (from_x, to_x) = if from_x < to_x {
            (from_x, to_x)
        } else {
            (to_x, from_x)
        };
        let (from_y, to_y) = if from_y < to_y {
            (from_y, to_y)
        } else {
            (to_y, from_y)
        };

        let width = to_x - from_x;
        let height = to_y - from_y;

        DrawableClipParams {
            mode,
            area: vec4(from_x, from_y, width, height),
        }
    }

    fn get_transform(&self) -> Mat4 {
        use LayerProperty::*;

        let mut result = Mat4::IDENTITY;

        bitflags::bitflags! {
            #[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Clone, Copy)]
            struct FlipFlags: i32 {
                const FLIP_HORIZONTAL = 0b01;
                const FLIP_VERTICAL = 0b10;
            }
        }
        let flip = FlipFlags::from_bits_truncate(self.get_value(Flip) as i32);
        let flip_scale = vec2(
            if flip.contains(FlipFlags::FLIP_HORIZONTAL) {
                -1.0
            } else {
                1.0
            },
            if flip.contains(FlipFlags::FLIP_VERTICAL) {
                -1.0
            } else {
                1.0
            },
        );

        let scale_origin = vec3(
            self.get_value(ScaleOriginX),
            self.get_value(ScaleOriginY),
            0.0,
        );

        let total_scale = vec2(
            self.get_value(ScaleX)
                * 0.001
                * self.get_value(ScaleX2)
                * 0.001
                * self.evaluate_wobbler(
                    &self.wobbler_scale_x,
                    WobbleScaleXAmplitude,
                    WobbleScaleXBias,
                    0.001,
                    1.0,
                ),
            self.get_value(ScaleY)
                * 0.001
                * self.get_value(ScaleY2)
                * 0.001
                * self.evaluate_wobbler(
                    &self.wobbler_scale_y,
                    WobbleScaleYAmplitude,
                    WobbleScaleYBias,
                    0.001,
                    1.0,
                ),
        ) * flip_scale;

        let rotation_origin = vec3(
            self.get_value(RotationOriginX),
            self.get_value(RotationOriginY),
            0.0,
        );

        let total_rotation = (self.get_value(Rotation) * 0.001
            + self.get_value(Rotation2) * 0.001
            + self.evaluate_wobbler(
                &self.wobbler_rotation,
                WobbleRotationAmplitude,
                WobbleRotationBias,
                0.001,
                0.0,
            ))
            * std::f32::consts::TAU;

        let total_translation = vec3(
            self.get_value(TranslateX) + self.get_value(TranslateX2)
                - self.get_value(NegatedTranslationX),
            self.get_value(TranslateY) + self.get_value(TranslateY2)
                - self.get_value(NegatedTranslationY),
            self.get_value(TranslateZ),
        );

        result = Mat4::from_translation(-scale_origin) * result;
        result = Mat4::from_scale(total_scale.extend(1.0)) * result;
        result = Mat4::from_translation(scale_origin - rotation_origin) * result;
        result = Mat4::from_rotation_z(total_rotation) * result;
        result = Mat4::from_translation(total_translation + rotation_origin) * result;

        result
    }

    fn get_wobble_translation(&self) -> Vec2 {
        use LayerProperty::*;

        vec2(
            self.evaluate_wobbler(&self.wobbler_x, WobbleXAmplitude, WobbleXBias, 1.0, 0.0),
            self.evaluate_wobbler(&self.wobbler_y, WobbleYAmplitude, WobbleYBias, 1.0, 0.0),
        )
    }

    pub fn get_transform_params(&self) -> TransformParams {
        use LayerProperty::*;

        let transform = self.get_transform();

        let camera_position = vec3(
            self.get_value(CameraPositionX),
            self.get_value(CameraPositionY),
            self.get_value(CameraPositionZ),
        );
        let unconditionally_inherited_translation = vec2(
            self.get_value(UnconditionallyInheritedTranslationX),
            self.get_value(UnconditionallyInheritedTranslationY),
        );
        let wobble_translation = self.get_wobble_translation();

        TransformParams {
            transform,
            camera_position,
            unconditionally_inherited_translation,
            wobble_translation,
        }
    }

    pub fn get_compose_flags(&self) -> ComposeFlags {
        let flags = self.get_value(LayerProperty::ComposeFlags) as i32;
        ComposeFlags::from_bits_truncate(flags)
    }
}

impl Updatable for LayerProperties {
    fn update(&mut self, context: &UpdateContext) {
        let dt = context.delta_time;

        for property in self.properties.values_mut() {
            property.update(dt);
        }

        macro_rules! get {
            ($property:ident) => {
                self.get_value(LayerProperty::$property)
            };
        }

        macro_rules! get_ticks {
            ($property:ident) => {
                Ticks::from_f32(get!($property))
            };
        }

        macro_rules! wobble {
            ($wobbler_name:ident, $wobble_mode:ident, $wobble_period:ident) => {
                self.$wobbler_name
                    .update(dt, get!($wobble_mode), get_ticks!($wobble_period));
            };
        }

        wobble!(wobbler_x, WobbleXMode, WobbleXPeriod);
        wobble!(wobbler_y, WobbleYMode, WobbleYPeriod);
        wobble!(wobbler_alpha, WobbleAlphaMode, WobbleAlphaPeriod);
        wobble!(wobbler_rotation, WobbleRotationMode, WobbleRotationPeriod);
        wobble!(wobbler_scale_x, WobbleScaleXMode, WobbleScaleXPeriod);
        wobble!(wobbler_scale_y, WobbleScaleYMode, WobbleScaleYPeriod);
    }
}

/// Stores only target property values.
/// Used to implement save/load (to quickly restore the state of the scene).
#[derive(Debug, Clone)]
pub struct LayerPropertiesSnapshot {
    // The game can actually only set integer values
    // hence the the use of i32 instead of f32
    properties: EnumMap<LayerProperty, i32>,
}

impl LayerPropertiesSnapshot {
    pub fn new() -> Self {
        Self {
            properties: initial_values(),
        }
    }

    pub fn init(&mut self) {
        self.properties = initial_values();
    }

    #[allow(unused)]
    pub fn get_property(&self, property: LayerProperty) -> i32 {
        self.properties[property]
    }

    pub fn set_property(&mut self, property: LayerProperty, value: i32) {
        self.properties[property] = value;
    }
}
