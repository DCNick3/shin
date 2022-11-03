use crate::format::scenario::instructions::NumberSpec;
use crate::vm::{FromVmCtx, VmCtx};
use num_derive::FromPrimitive;

pub const LAYERBANKS_COUNT: u8 = 0x30;
pub const LAYERS_COUNT: u32 = 0x100;
pub const PLANES_COUNT: u32 = 4;

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

pub type LayerbankId = Id<u8, { LAYERBANKS_COUNT as u32 }>;
pub type LayerbankIdOpt = IdOpt<u8, { LAYERBANKS_COUNT as u32 }>;

/// Layer id, allowing for the special values -1, -2, -3, -4, -5
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VLayerId(i32);
#[derive(Debug)]
pub enum VLayerIdRepr {
    // TODO: give these meaningful names
    Neg1,
    Neg2,
    Neg3,
    Neg4,
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
                -1 => VLayerIdRepr::Neg1,
                -2 => VLayerIdRepr::Neg2,
                -3 => VLayerIdRepr::Neg3,
                -4 => VLayerIdRepr::Neg4,
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

#[derive(FromPrimitive, Debug, Copy, Clone)]
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
