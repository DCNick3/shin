use crate::format::scenario::instruction_elements::NumberSpec;
use crate::format::scenario::types::U8SmallNumberList;
use crate::vm::{FromVmCtx, VmCtx};
use enum_map::Enum;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(FromPrimitive, Enum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerProperty {
    TranslateX = 0,
    TranslateY = 1,
    /// Unused by the game (TODO: machine-readable annotations for this)
    TranslateZ = 2,
    TranslateX2 = 3,
    TranslateY2 = 4,
    RenderPosition = 5,

    Prop6 = 6,
    Prop7 = 7,
    Prop8 = 8,
    Prop9 = 9,

    ScaleOriginX = 10,
    ScaleOriginY = 11,
    ScaleX = 12,
    ScaleY = 13,
    ScaleX2 = 14,
    ScaleY2 = 15,

    RotationOriginX = 16,
    RotationOriginY = 17,
    /// Rotation of the layer in 1000th of rotation
    Rotation = 18,
    Rotation2 = 19,

    Prop20 = 20,
    Prop21 = 21,

    ShowLayer = 22,
    Prop23 = 23,
    Prop24 = 24,
    Prop25 = 25,
    Prop26 = 26,
    Prop27 = 27,
    Prop28 = 28,
    Prop29 = 29,
    Prop30 = 30,
    Prop31 = 31,

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
    Prop56 = 56,
    Prop57 = 57,
    Prop58 = 58,
    Prop59 = 59,
    Prop60 = 60,

    Prop61 = 61,
    Prop62 = 62,
    Prop63 = 63,
    Prop64 = 64,
    Prop65 = 65,
    Prop66 = 66,
    Prop67 = 67,
    Prop68 = 68,

    // "Blur" effect
    Prop69 = 69,
    // "Pixelize" effect
    PixelizeSize = 70,

    // "Dissolve" Effect, used by the witch
    Prop71 = 71,
    Prop72 = 72,

    // "Rain" effect
    Prop73 = 73,
    Prop74 = 74,
    Prop75 = 75,

    // "Waves" effect
    Prop76 = 76,
    Prop77 = 77,
    Prop78 = 78,

    // dunno
    Prop79 = 79,
    Prop80 = 80,
    Prop81 = 81,

    // "Ripple" effect
    Prop82 = 82,
    Prop83 = 83,
    Prop84 = 84,

    Prop85 = 85,
    Prop86 = 86,
    Prop87 = 87,
    Prop88 = 88,
    Prop89 = 89,
    Prop90 = 90,
}

impl LayerProperty {
    // pub const COUNT: usize = <LayerProperty as Enum>::Array::LENGTH;

    pub fn initial_value(self) -> i32 {
        use LayerProperty::*;
        match self {
            TranslateZ => 1000,
            RenderPosition => 1000,
            Prop6 => 1000,
            Prop7 => 1000,
            Prop8 => 1000,
            Prop9 => 1000,
            ScaleX => 1000,
            ScaleY => 1000,
            ScaleX2 => 1000,
            ScaleY2 => 1000,
            ShowLayer => 1,
            Prop27 => 1,
            Prop28 => 1000,
            Prop29 => 1000,
            Prop30 => 1000,
            Prop31 => 1000,
            WobbleAlphaBias => 1000,
            WobbleScaleXBias => 1000,
            WobbleScaleYBias => 1000,
            Prop57 => 1000,
            Prop73 => 1000,
            Prop75 => 1000,
            _ => 0,
        }
    }

    pub fn is_implemented(&self) -> bool {
        use LayerProperty::*;
        matches!(
            self,
            ScaleOriginX | ScaleOriginY | ScaleX | ScaleY | ScaleX2 | ScaleY2 |
            WobbleScaleXMode | WobbleScaleXPeriod | WobbleScaleXAmplitude | WobbleScaleXBias |
            WobbleScaleYMode | WobbleScaleYPeriod | WobbleScaleYAmplitude | WobbleScaleYBias |
            RotationOriginX | RotationOriginY | Rotation | Rotation2 |
            WobbleRotationMode | WobbleRotationPeriod | WobbleRotationAmplitude | WobbleRotationBias |
            TranslateX | TranslateY | TranslateX2 | TranslateY2 |
            WobbleXMode | WobbleXPeriod | WobbleXAmplitude | WobbleXBias |
            WobbleYMode | WobbleYPeriod | WobbleYAmplitude | WobbleYBias |

            // this one is not, actually, implemented
            // everything seems to work fine, so ignoring it for now
            // (figuring out how it works is non-trivial tbh)
            RenderPosition
        )
    }
}

impl FromVmCtx<NumberSpec> for LayerProperty {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        let num = ctx.get_number(input);
        FromPrimitive::from_i32(num)
            .unwrap_or_else(|| panic!("LayerProperty::from_vm_ctx: invalid layer type: {}", num))
    }
}

pub type LayerPropertySmallList = smallvec::SmallVec<[LayerProperty; 6]>;

impl FromVmCtx<U8SmallNumberList> for LayerPropertySmallList {
    fn from_vm_ctx(ctx: &VmCtx, input: U8SmallNumberList) -> Self {
        input
            .0
            .into_iter()
            .map(|n| {
                let n = ctx.get_number(n);
                FromPrimitive::from_i32(n).unwrap_or_else(|| {
                    panic!(
                        "LayerPropertySmallList::from_vm_ctx: invalid layer type: {}",
                        n
                    )
                })
            })
            .collect()
    }
}
