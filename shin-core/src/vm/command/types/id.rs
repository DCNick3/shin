use crate::format::scenario::instruction_elements::FromNumber;

pub const LAYERBANKS_COUNT: usize = 0x30;
pub const LAYERS_COUNT: usize = 0x100;
pub const PLANES_COUNT: usize = 4;

pub trait ThroughUsize {
    fn from_usize(value: usize) -> Self;
    fn into_usize(self) -> usize;
}

impl ThroughUsize for u8 {
    fn from_usize(value: usize) -> Self {
        value.try_into().unwrap()
    }

    fn into_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

impl ThroughUsize for u16 {
    fn from_usize(value: usize) -> Self {
        value.try_into().unwrap()
    }

    fn into_usize(self) -> usize {
        self.try_into().unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id<T: num_traits::Unsigned + ThroughUsize + Copy, const SENTINEL: usize>(T);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IdOpt<T: num_traits::Unsigned + ThroughUsize + Copy, const SENTINEL: usize>(T);

impl<T: num_traits::Unsigned + ThroughUsize + Copy, const SENTINEL: usize> Id<T, SENTINEL> {
    pub fn try_new(id: T) -> Option<Self> {
        if (0..SENTINEL).contains(&id.into_usize()) {
            Some(Self(id))
        } else {
            None
        }
    }

    pub fn new(id: T) -> Self {
        Self::try_new(id).expect("Id::new: id out of range")
    }

    /// Doesn't check that the `T` is in range (here be dragons), but is a const fn
    pub const fn new_unchecked(id: T) -> Self {
        Self(id)
    }

    pub fn raw(self) -> T {
        self.0
    }

    pub fn try_next(self) -> Option<Self> {
        let id = self.0 + T::one();
        if id.into_usize() == SENTINEL {
            None
        } else {
            Some(Self(id))
        }
    }

    pub fn next(self) -> Self {
        self.try_next().expect("Id::next: id out of range")
    }
}

impl<T: num_traits::Unsigned + ThroughUsize + Copy, const SENTINEL: usize> IdOpt<T, SENTINEL> {
    pub fn none() -> Self {
        Self(T::from_usize(SENTINEL))
    }

    pub fn some(id: Id<T, SENTINEL>) -> Self {
        Self(id.0)
    }

    pub fn into_option(self) -> Option<Id<T, SENTINEL>> {
        if self.0.into_usize() == SENTINEL {
            None
        } else {
            Some(Id(self.0))
        }
    }

    pub fn unwrap(self) -> Id<T, SENTINEL> {
        self.into_option().expect("IdOpt::unwrap: none value")
    }

    pub fn raw(self) -> T {
        self.0
    }
}

/// Layer id, but allowing only "real" layers
pub type LayerId = Id<u16, { LAYERS_COUNT }>;
/// Layer id, but allowing only "real" layers and a "none" value
pub type LayerIdOpt = IdOpt<u16, { LAYERS_COUNT }>;

pub type LayerbankId = Id<u8, { LAYERBANKS_COUNT }>;
pub type LayerbankIdOpt = IdOpt<u8, { LAYERBANKS_COUNT }>;

pub type PlaneId = Id<u8, { PLANES_COUNT }>;
pub type PlaneIdOpt = IdOpt<u8, { PLANES_COUNT }>;

/// Layer id, allowing for the special values -1, -2, -3, -4, -5
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VLayerId(i32);

#[derive(Debug)]
pub enum VLayerIdRepr {
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

impl FromNumber for VLayerId {
    fn from_number(number: i32) -> Self {
        VLayerId::new(number)
    }
}

impl FromNumber for LayerId {
    fn from_number(number: i32) -> Self {
        LayerId::new(number.try_into().unwrap())
    }
}
