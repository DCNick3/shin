use crate::format::scenario::instructions::NumberSpec;
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
    Prop3 = 3,
    Prop4 = 4,
    Prop5 = 5,
    Prop6 = 6,
    Prop7 = 7,
    Prop8 = 8,
    Prop9 = 9,
    Prop10 = 10,
    Prop11 = 11,
    Prop12 = 12,
    Prop13 = 13,
    Prop14 = 14,
    Prop15 = 15,
    Prop16 = 16,
    Prop17 = 17,
    /// Rotation of the layer in degrees
    Rotation = 18,
    Prop19 = 19,
    Prop20 = 20,
    Prop21 = 21,
    Prop22 = 22,
    Prop23 = 23,
    Prop24 = 24,
    Prop25 = 25,
    Prop26 = 26,
    Prop27 = 27,
    Prop28 = 28,
    Prop29 = 29,
    Prop30 = 30,
    Prop31 = 31,
    Prop32 = 32,
    Prop33 = 33,
    Prop34 = 34,
    Prop35 = 35,
    Prop36 = 36,
    Prop37 = 37,
    Prop38 = 38,
    Prop39 = 39,
    Prop40 = 40,
    Prop41 = 41,
    Prop42 = 42,
    Prop43 = 43,
    Prop44 = 44,
    Prop45 = 45,
    Prop46 = 46,
    Prop47 = 47,
    Prop48 = 48,
    Prop49 = 49,
    Prop50 = 50,
    Prop51 = 51,
    Prop52 = 52,
    Prop53 = 53,
    Prop54 = 54,
    Prop55 = 55,
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
    Prop69 = 69,
    Prop70 = 70,
    Prop71 = 71,
    Prop72 = 72,
    Prop73 = 73,
    Prop74 = 74,
    Prop75 = 75,
    Prop76 = 76,
    Prop77 = 77,
    Prop78 = 78,
    Prop79 = 79,
    Prop80 = 80,
    Prop81 = 81,
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
    pub const COUNT: usize = <LayerProperty as Enum>::LENGTH;

    pub fn initial_value(self) -> i32 {
        match self {
            LayerProperty::TranslateZ => 1000,
            LayerProperty::Prop5 => 1000,
            LayerProperty::Prop6 => 1000,
            LayerProperty::Prop7 => 1000,
            LayerProperty::Prop8 => 1000,
            LayerProperty::Prop9 => 1000,
            LayerProperty::Prop12 => 1000,
            LayerProperty::Prop13 => 1000,
            LayerProperty::Prop14 => 1000,
            LayerProperty::Prop15 => 1000,
            LayerProperty::Prop22 => 1,
            LayerProperty::Prop27 => 1,
            LayerProperty::Prop28 => 1000,
            LayerProperty::Prop29 => 1000,
            LayerProperty::Prop30 => 1000,
            LayerProperty::Prop31 => 1000,
            LayerProperty::Prop43 => 1000,
            LayerProperty::Prop51 => 1000,
            LayerProperty::Prop55 => 1000,
            LayerProperty::Prop57 => 1000,
            LayerProperty::Prop73 => 1000,
            LayerProperty::Prop75 => 1000,
            _ => 0,
        }
    }

    // TODO: add a function to check whether the property is supported
    // we can print a warning or smth if it's not
}

impl FromVmCtx<NumberSpec> for LayerProperty {
    fn from_vm_ctx(ctx: &VmCtx, input: NumberSpec) -> Self {
        let num = ctx.get_number(input);
        num_traits::FromPrimitive::from_i32(num)
            .unwrap_or_else(|| panic!("LayerProperty::from_vm_ctx: invalid layer type: {}", num))
    }
}
