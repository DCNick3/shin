use glam::{vec4, Vec4};

#[derive(Copy, Clone, Debug, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(transparent)]
pub struct UnormColor(pub u32);

impl UnormColor {
    pub fn from_decimal_rgb(rgb: i32) -> Self {
        fn decimal_to_8bit(v: u8) -> u8 {
            match v {
                0 => 0,
                1 => 0x1c,
                2 => 0x39,
                3 => 0x55,
                4 => 0x71,
                5 => 0x8e,
                6 => 0xaa,
                7 => 0xc6,
                8 => 0xe3,
                9 => 0xff,
                _ => unreachable!(),
            }
        }

        let rgb = rgb.clamp(0, 999);
        Self::from_rgba(
            decimal_to_8bit((rgb % 10) as u8),
            decimal_to_8bit(((rgb / 10) % 10) as u8),
            decimal_to_8bit(((rgb / 100) % 10) as u8),
            0xff,
        )
    }

    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self((r as u32) | ((g as u32) << 8) | ((b as u32) << 16) | ((a as u32) << 24))
    }

    pub fn into_vec4(self) -> Vec4 {
        vec4(
            (self.0 & 0xff) as f32 / 255.0,
            ((self.0 >> 8) & 0xff) as f32 / 255.0,
            ((self.0 >> 16) & 0xff) as f32 / 255.0,
            ((self.0 >> 24) & 0xff) as f32 / 255.0,
        )
    }

    pub const RED: Self = Self(0xff0000ff);
    pub const GREEN: Self = Self(0xff00ff00);
    pub const BLUE: Self = Self(0xffff0000);

    pub const PASTEL_GREEN: Self = Self(0xffc1e1c1);
    pub const PASTEL_PINK: Self = Self(0xffdcd1ff);

    pub const WHITE: Self = Self(0xffffffff);
    pub const BLACK: Self = Self(0xff000000);
}

#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[cfg_attr(feature = "encase", derive(encase::ShaderType))]
#[repr(C)]
pub struct FloatColor4 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl FloatColor4 {
    /// Creates a new [`FloatColor4`] from an integer of form `0xARGB` (4 bits per channel).
    #[expect(clippy::identity_op)]
    pub fn from_4bpp_property(value: i32) -> Self {
        let alpha = ((value & 0xf000) >> 12) as f32 / 0xf as f32;
        let red = ((value & 0x0f00) >> 8) as f32 / 0xf as f32;
        let green = ((value & 0x00f0) >> 4) as f32 / 0xf as f32;
        let blue = ((value & 0x000f) >> 0) as f32 / 0xf as f32;

        Self::from_rgba(red, green, blue, alpha)
    }

    pub const fn from_unorm(color: UnormColor) -> Self {
        Self {
            r: (color.0 & 0xff) as f32 / 255.0,
            g: ((color.0 >> 8) & 0xff) as f32 / 255.0,
            b: ((color.0 >> 16) & 0xff) as f32 / 255.0,
            a: ((color.0 >> 24) & 0xff) as f32 / 255.0,
        }
    }

    pub const fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const fn from_vec4(vec: Vec4) -> Self {
        let [r, g, b, a] = vec.to_array();
        Self { r, g, b, a }
    }

    pub const fn into_vec4(self) -> Vec4 {
        vec4(self.r, self.g, self.b, self.a)
    }

    pub const fn into_array(self) -> [f32; 4] {
        [self.r, self.g, self.b, self.a]
    }

    pub fn into_unorm(self) -> UnormColor {
        let [r, g, b, a] = self.into_array().map(|c| (c * 255.0) as u8);
        UnormColor::from_rgba(r, g, b, a)
    }

    pub fn premultiply(self) -> Self {
        let [r, g, b, a] = self.into_array();
        Self::from_rgba(r * a, g * a, b * a, a)
    }

    pub fn with_alpha(self, new_alpha: f32) -> Self {
        Self::from_rgba(self.r, self.g, self.b, new_alpha)
    }

    pub const RED: Self = Self::from_unorm(UnormColor::RED);
    pub const GREEN: Self = Self::from_unorm(UnormColor::GREEN);
    pub const BLUE: Self = Self::from_unorm(UnormColor::BLUE);

    pub const PASTEL_GREEN: Self = Self::from_unorm(UnormColor::PASTEL_GREEN);
    pub const PASTEL_PINK: Self = Self::from_unorm(UnormColor::PASTEL_PINK);

    pub const WHITE: Self = Self::from_unorm(UnormColor::WHITE);
    pub const BLACK: Self = Self::from_unorm(UnormColor::BLACK);
}

impl std::ops::Mul<FloatColor4> for FloatColor4 {
    type Output = FloatColor4;

    fn mul(self, rhs: FloatColor4) -> Self::Output {
        Self::from_vec4(self.into_vec4() * rhs.into_vec4())
    }
}
