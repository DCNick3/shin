use crate::format::scenario::instructions::NumberSpec;
use crate::time::Ticks;
use crate::vm::{FromVmCtx, VmCtx};
use enum_map::Enum;
use num_derive::FromPrimitive;

pub const LAYERBANKS_COUNT: u8 = 0x30;
pub const LAYERS_COUNT: u32 = 0x100;
pub const PLANES_COUNT: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T: num_traits::Unsigned + TryFrom<u32> + Into<u32> + Copy, const SENTINEL: u32>(T);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdOpt<T: num_traits::Unsigned + TryFrom<u32> + Into<u32> + Copy, const SENTINEL: u32>(T);

impl<T: num_traits::Unsigned + TryFrom<u32> + Into<u32> + Copy, const SENTINEL: u32>
    Id<T, SENTINEL>
{
    pub fn new(id: T) -> Self {
        assert!(
            (0..SENTINEL).contains(&id.into()),
            "Id::new: id out of range"
        );
        Self(id)
    }

    pub fn raw(self) -> T {
        self.0
    }

    pub fn next(self) -> Self {
        let id = self.0 + T::one();
        assert_ne!(id.into(), SENTINEL, "Id::next: id out of range");
        Self::new(id)
    }
}

impl<T: num_traits::Unsigned + TryFrom<u32> + Into<u32> + Copy, const SENTINEL: u32>
    IdOpt<T, SENTINEL>
{
    pub fn none() -> Self {
        Self(
            T::try_from(SENTINEL)
                .map_err(|_| "BUG: sentinel conversion failed")
                .unwrap(),
        )
    }

    pub fn some(id: Id<T, SENTINEL>) -> Self {
        Self(id.0)
    }

    pub fn opt(self) -> Option<Id<T, SENTINEL>> {
        if self.0.into() == SENTINEL {
            None
        } else {
            Some(Id(self.0))
        }
    }

    pub fn raw(self) -> T {
        self.0
    }
}

/// Layer id, but allowing only "real" layers
pub type LayerId = Id<u32, { LAYERS_COUNT as u32 }>;
/// Layer id, but allowing only "real" layers and a "none" value
pub type LayerIdOpt = IdOpt<u32, { LAYERS_COUNT as u32 }>;

/// Layer id, allowing for the special values -1, -2, -3, -4, -5
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VLayerId(i32);
#[derive(Debug)]
pub enum VLayerIdRepr {
    // TODO: give these meaningful names
    RootLayerGroup,
    ScreenLayer,
    PageLayer,
    PlaneLayerGroup,
    Selected,
    Layer(LayerId),
}

impl VLayerId {
    pub const MIN: i32 = -5;

    pub fn new(id: i32) -> Self {
        assert!(
            (Self::MIN..LAYERS_COUNT.try_into().unwrap()).contains(&id),
            "VLayerId::new: id out of range"
        );
        Self(id)
    }

    pub fn repr(self) -> VLayerIdRepr {
        if self.0 < 0 {
            match self.0 {
                -1 => VLayerIdRepr::RootLayerGroup,
                -2 => VLayerIdRepr::ScreenLayer,
                -3 => VLayerIdRepr::PageLayer,
                -4 => VLayerIdRepr::PlaneLayerGroup,
                -5 => VLayerIdRepr::Selected,
                _ => unreachable!(),
            }
        } else {
            VLayerIdRepr::Layer(LayerId::new(self.0.try_into().unwrap()))
        }
    }
}

impl FromVmCtx<NumberSpec> for VLayerId {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        VLayerId::new(ctx.get_number(input))
    }
}

impl FromVmCtx<NumberSpec> for LayerId {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        LayerId::new(ctx.get_number(input).try_into().unwrap())
    }
}

#[derive(FromPrimitive, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerType {
    Null = 0,
    Tile = 1,
    Picture = 2,
    Bustup = 3,
    Animation = 4,
    Effect = 5,
    Movie = 6,
    FocusLine = 7,
    Rain = 8,
    Quiz = 9,
}

impl FromVmCtx<NumberSpec> for LayerType {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        let num = ctx.get_number(input);
        num_traits::FromPrimitive::from_i32(num)
            .unwrap_or_else(|| panic!("LayerType::from_vm_ctx: invalid layer type: {}", num))
    }
}

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
        num_traits::FromPrimitive::from_i32(num)
            .unwrap_or_else(|| panic!("LayerProperty::from_vm_ctx: invalid layer type: {}", num))
    }
}

pub type LayerPropertySmallList = smallvec::SmallVec<[LayerProperty; 6]>;

#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum MessageboxType {
    Neutral = 0,
    WitchSpace = 1,
    Ushiromiya = 2,
    Transparent = 3,
    Novel = 4,
    NoText = 5,
}

#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum MessageTextLayout {
    Left = 0,
    /// I _think_ this is the same as Left
    Layout1 = 1,
    Center = 2,
    Right = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub struct MessageboxStyle {
    pub messagebox_type: MessageboxType,
    pub text_layout: MessageTextLayout,
}

impl Default for MessageboxStyle {
    fn default() -> Self {
        Self {
            messagebox_type: MessageboxType::Neutral,
            text_layout: MessageTextLayout::Left,
        }
    }
}

impl FromVmCtx<NumberSpec> for MessageboxStyle {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        let v = ctx.get_number(input);
        assert!(v >= 0);
        let msgbox_type = v & 0xf;
        let text_layout = (v >> 4) & 0xf;
        Self {
            messagebox_type: num_traits::FromPrimitive::from_i32(msgbox_type).unwrap_or_else(
                || panic!("MsgInit::from: unknown messagebox type: {}", msgbox_type),
            ),
            text_layout: num_traits::FromPrimitive::from_i32(text_layout)
                .unwrap_or_else(|| panic!("MsgInit::from: unknown text layout: {}", text_layout)),
        }
    }
}

impl FromVmCtx<NumberSpec> for Ticks {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        Ticks::from_i32(ctx.get_number(input))
    }
}
