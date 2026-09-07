/// A 24-bit colour, as written in MiniMessage with `<#rrggbb>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl RgbColor {
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    pub const fn from_packed(value: u32) -> Self {
        Self {
            red: ((value >> 16) & 0xFF) as u8,
            green: ((value >> 8) & 0xFF) as u8,
            blue: (value & 0xFF) as u8,
        }
    }

    pub const fn to_packed(self) -> u32 {
        ((self.red as u32) << 16) | ((self.green as u32) << 8) | (self.blue as u32)
    }
}
