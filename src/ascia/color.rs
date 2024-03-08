#[derive(Debug, Copy, Clone)]
pub struct Color8bit {
    pub data: u8,
}

impl From<ColorRGBu8> for Color8bit{
    fn from(value: ColorRGBu8) -> Self{
        if value.r == value.g && value.g == value.b{
            return Color8bit{
                data:232 + value.r / 11
            }
        }
        return Color8bit{
            data:16 + (value.r as f64 / (256.0 / 6.0)) as u8 * 36 + (value.g as f64 / (256.0 / 6.0)) as u8 * 6 + (value.b as f64 / (256.0 / 6.0)) as u8
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ColorRGBu8 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Default for ColorRGBu8 {
    fn default() -> Self {
        return ColorRGBu8 {
            r:0,
            g:0,
            b:0
        }
    }
}

impl std::ops::AddAssign<ColorRGBu8> for ColorRGBu8 {
    fn add_assign(&mut self, rhs: ColorRGBu8) {
        self.r = self.r.saturating_add(rhs.r);
        self.g = self.g.saturating_add(rhs.g);
        self.b = self.b.saturating_add(rhs.b);
    }
}

impl std::ops::SubAssign<ColorRGBu8> for ColorRGBu8 {
    fn sub_assign(&mut self, rhs: ColorRGBu8) {
        self.r = self.r.saturating_sub(rhs.r);
        self.g = self.g.saturating_sub(rhs.g);
        self.b = self.b.saturating_sub(rhs.b);
    }
}

impl std::ops::Add<ColorRGBu8> for ColorRGBu8 {
    type Output = ColorRGBu8;

    fn add(self, rhs: ColorRGBu8) -> Self::Output{
        return ColorRGBu8 {
            r: self.r.saturating_add(rhs.r),
            g: self.g.saturating_add(rhs.g),
            b: self.b.saturating_add(rhs.b),
        }
    }
}

impl std::ops::Sub<ColorRGBu8> for ColorRGBu8 {
    type Output = ColorRGBu8;

    fn sub(self, rhs: ColorRGBu8) -> Self::Output{
        return ColorRGBu8 {
            r: self.r.saturating_sub(rhs.r),
            g: self.g.saturating_sub(rhs.g),
            b: self.b.saturating_sub(rhs.b),
        }
    }
}

impl From<u32> for ColorRGBu8 {
    fn from(value: u32) -> Self {
        return ColorRGBu8 {
            r: ((value & 0xff0000) >> 16) as u8,
            g: ((value & 0x00ff00) >> 8) as u8,
            b: (value & 0x0000ff) as u8,
        }
    }
}

impl From<ColorRGBf32> for ColorRGBu8 {
    fn from(value: ColorRGBf32) -> Self {
        return ColorRGBu8 {
            r: f32::clamp(value.r * 255.0,0.0,255.0) as u8,
            g: f32::clamp(value.g * 255.0,0.0,255.0) as u8,
            b: f32::clamp(value.b * 255.0,0.0,255.0) as u8,
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ColorRGBf32 {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl std::ops::AddAssign<ColorRGBf32> for ColorRGBf32 {
    fn add_assign(&mut self, rhs: ColorRGBf32) {
        self.r = self.r + rhs.r;
        self.g = self.g + rhs.g;
        self.b = self.b + rhs.b;
    }
}

impl std::ops::SubAssign<ColorRGBf32> for ColorRGBf32 {
    fn sub_assign(&mut self, rhs: ColorRGBf32) {
        self.r = f32::max(self.r - rhs.r,0.0);
        self.g = f32::max(self.g - rhs.g,0.0);
        self.b = f32::max(self.b - rhs.b,0.0);
    }
}

impl std::ops::Add<ColorRGBf32> for ColorRGBf32 {
    type Output = ColorRGBf32;

    fn add(self, rhs: ColorRGBf32) -> Self::Output{
        return ColorRGBf32 {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl std::ops::Sub<ColorRGBf32> for ColorRGBf32 {
    type Output = ColorRGBf32;

    fn sub(self, rhs: ColorRGBf32) -> Self::Output{
        return ColorRGBf32 {
            r: f32::max(self.r - rhs.r,0.0),
            g: f32::max(self.g - rhs.g,0.0),
            b: f32::max(self.b - rhs.b,0.0),
        }
    }
}

impl Default for ColorRGBf32{
    fn default() -> Self {
        return ColorRGBf32 {
            r: 0.0,
            g: 0.0,
            b: 0.0,
        }
    }
}

impl From<ColorRGBu8> for ColorRGBf32 {
    fn from(value: ColorRGBu8) -> Self {
        return ColorRGBf32 {
            r: (value.r as f32 / 255.0),
            g: (value.g as f32 / 255.0),
            b: (value.b as f32 / 255.0),
        }
    }
}