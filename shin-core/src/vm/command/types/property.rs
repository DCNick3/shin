use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use crate::format::scenario::instruction_elements::FromNumber;

#[derive(FromPrimitive, Enum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerProperty {
    TranslateX = 0,
    TranslateY = 1,
    /// Unused by the game (TODO: machine-readable annotations for this)
    TranslateZ = 2,
    TranslateX2 = 3,
    TranslateY2 = 4,
    RenderPosition = 5,

    MulColorRed = 6,
    MulColorGreen = 7,
    MulColorBlue = 8,
    MulColorAlpha = 9,

    ScaleOriginX = 10,
    ScaleOriginY = 11,
    ScaleX = 12,
    ScaleY = 13,
    ScaleX2 = 14,
    ScaleY2 = 15,

    RotationOriginX = 16,
    RotationOriginY = 17,
    /// Rotation of the layer in milliturns (1000mt = 1 turn)
    Rotation = 18,
    Rotation2 = 19,

    NegatedTranslationX = 20,
    NegatedTranslationY = 21,

    ShowLayer = 22,
    BlendType = 23,
    FragmentShader = 24,
    ComposeFlags = 25,
    Flip = 26,
    Prop27 = 27,
    ShaderParamX = 28,
    ShaderParamY = 29,
    ShaderParamZ = 30,
    ShaderParamW = 31,

    WobbleXMode = 32,
    WobbleXPeriod = 33,
    WobbleXAmplitude = 34,
    WobbleXBias = 35,

    WobbleYMode = 36,
    WobbleYPeriod = 37,
    WobbleYAmplitude = 38,
    WobbleYBias = 39,

    WobbleAlphaMode = 40,
    WobbleAlphaPeriod = 41,
    WobbleAlphaAmplitude = 42,
    WobbleAlphaBias = 43,

    WobbleScaleXMode = 44,
    WobbleScaleXPeriod = 45,
    WobbleScaleXAmplitude = 46,
    WobbleScaleXBias = 47,

    WobbleScaleYMode = 48,
    WobbleScaleYPeriod = 49,
    WobbleScaleYAmplitude = 50,
    WobbleScaleYBias = 51,

    WobbleRotationMode = 52,
    WobbleRotationPeriod = 53,
    WobbleRotationAmplitude = 54,
    WobbleRotationBias = 55,

    // "Ghosting" effect
    GhostingAlpha = 56,
    GhostingZoom = 57,
    GhostingRotation = 58,
    GhostingTransformOriginX = 59,
    GhostingTransformOriginY = 60,

    // Clipping, makes the layer render only within a certain rectangle
    ClipMode = 61,
    ClipFromX = 62,
    ClipToX = 63,
    ClipFromY = 64,
    ClipToY = 65,

    // "Blur" effect?
    BlurRadius = 66,
    // "pixelizes" the image
    MosaicSize = 67,

    // "Dissolve" Effect, used by the witch
    DissolveIntensity = 68,
    DissolveMode = 69,

    // zoomblur?
    Prop70 = 70,
    Prop71 = 71,
    Prop72 = 72,

    // "Rain" effect, only applicable to the RainLayer (I think)
    RainIntensity = 73,
    RainDirection = 74,
    RainSpeed = 75,

    // "Raster" effect (wavey)
    // TODO: which one is horizontal and which one is vertical again? Not sure it's named correctly
    RasterHorizontalAmplitude = 76,
    RasterHorizontalLPeriod = 77,
    RasterHorizontalTPeriod = 78,
    RasterVerticalAmplitude = 79,
    RasterVerticalLPeriod = 80,
    RasterVerticalTPeriod = 81,

    // "Ripple" effect
    RippleAmplitude = 82,
    RippleLPeriod = 83,
    RippleTPeriod = 84,

    CameraPositionX = 85,
    CameraPositionY = 86,
    CameraPositionZ = 87,

    UnconditionallyInheritedTranslationX = 88,
    UnconditionallyInheritedTranslationY = 89,
}

impl LayerProperty {
    // pub const COUNT: usize = <LayerProperty as Enum>::Array::LENGTH;

    pub fn initial_value(self) -> i32 {
        use LayerProperty::*;
        match self {
            TranslateZ => 1000,
            RenderPosition => 1000,
            MulColorRed => 1000,
            MulColorGreen => 1000,
            MulColorBlue => 1000,
            MulColorAlpha => 1000,
            ScaleX => 1000,
            ScaleY => 1000,
            ScaleX2 => 1000,
            ScaleY2 => 1000,
            ShowLayer => 1,
            Prop27 => 1,
            ShaderParamX => 1000,
            ShaderParamY => 1000,
            ShaderParamZ => 1000,
            ShaderParamW => 1000,
            WobbleAlphaBias => 1000,
            WobbleScaleXBias => 1000,
            WobbleScaleYBias => 1000,
            GhostingZoom => 1000,
            RainIntensity => 1000,
            RainSpeed => 1000,
            _ => 0,
        }
    }
}

impl FromNumber for LayerProperty {
    fn from_number(number: i32) -> Self {
        FromPrimitive::from_i32(number)
            .unwrap_or_else(|| panic!("LayerProperty::from_vm_ctx: invalid layer type: {}", number))
    }
}
