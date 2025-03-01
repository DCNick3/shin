#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct BytesAddress(wgpu::BufferAddress);

impl BytesAddress {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: wgpu::BufferAddress) -> Self {
        Self(value)
    }

    pub const fn from_usize(value: usize) -> Self {
        Self(value as _)
    }

    pub const fn get(self) -> wgpu::BufferAddress {
        self.0
    }

    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }

    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub const fn is_aligned_to(&self, alignment: BytesAddress) -> bool {
        let remainder = self.0 % alignment.0;
        remainder == 0
    }

    pub fn assert_is_aligned_to(&self, alignment: BytesAddress) {
        assert!(
            self.is_aligned_to(alignment),
            "Address {:?} is not aligned to {:?}",
            self,
            alignment
        );
    }

    pub fn align_to(self, alignment: BytesAddress) -> Self {
        let remainder = BytesAddress(self.0 % alignment.0);
        if remainder == Self::ZERO {
            self
        } else {
            self + alignment - remainder
        }
    }
}

impl std::fmt::Display for BytesAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Add for BytesAddress {
    type Output = BytesAddress;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for BytesAddress {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for BytesAddress {
    type Output = BytesAddress;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for BytesAddress {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}
