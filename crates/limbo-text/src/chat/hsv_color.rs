use crate::chat::RgbColor;

/// Hue in turns, saturation and value, each in `0.0..=1.0`.
///
/// Single precision on purpose. The reference implementation uses Java `float`, and the
/// difference decides real cases: pure red sits equidistant between `red` and `dark_red`
/// in exact arithmetic, and only the rounding of 32-bit maths picks the same one.
pub(crate) struct HsvColor {
    hue: f32,
    saturation: f32,
    value: f32,
}

impl HsvColor {
    pub(crate) fn from_rgb(rgb: RgbColor) -> Self {
        let red = f32::from(rgb.red) / 255.0;
        let green = f32::from(rgb.green) / 255.0;
        let blue = f32::from(rgb.blue) / 255.0;

        let max = red.max(green).max(blue);
        let min = red.min(green).min(blue);
        let span = max - min;

        let hue = if span == 0.0 {
            0.0
        } else if max == red {
            ((green - blue) / span).rem_euclid(6.0)
        } else if max == green {
            (blue - red) / span + 2.0
        } else {
            (red - green) / span + 4.0
        } / 6.0;

        Self {
            hue,
            saturation: if max == 0.0 { 0.0 } else { span / max },
            value: max,
        }
    }

    /// The hue difference is weighted three times, as the reference implementation weighs it.
    pub(crate) fn distance_squared(&self, other: &Self) -> f32 {
        let hue_gap = (self.hue - other.hue).abs();
        let hue_distance = 3.0 * hue_gap.min(1.0 - hue_gap);
        let saturation_gap = self.saturation - other.saturation;
        let value_gap = self.value - other.value;

        hue_distance * hue_distance + saturation_gap * saturation_gap + value_gap * value_gap
    }
}
