use crate::chat::named_color::NamedColor;
use crate::chat::rgb_color::RgbColor;

/// The colour of a component, either one of the sixteen legacy names or an arbitrary
/// 24-bit value.
///
/// The distinction survives until serialization, because clients older than 1.16 accept
/// only names and everything else has to be reduced with [`TextColor::to_named`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextColor {
    Named(NamedColor),
    Rgb(RgbColor),
}

impl TextColor {
    pub const fn rgb(self) -> RgbColor {
        match self {
            Self::Named(named) => named.rgb(),
            Self::Rgb(rgb) => rgb,
        }
    }

    /// Reduces this colour to the closest of the sixteen named colours.
    ///
    /// Distance is measured in HSV with the hue difference weighted three times, which is
    /// what the reference implementation uses. An RGB metric picks visibly different
    /// colours: a pale blue is nearest to grey by RGB distance but to blue by hue, and
    /// the client shows whichever one we send.
    pub fn to_named(self) -> NamedColor {
        match self {
            Self::Named(named) => named,
            Self::Rgb(rgb) => nearest_named(rgb),
        }
    }
}

/// Hue in turns, saturation and value, each in `0.0..=1.0`.
///
/// Single precision on purpose. The reference implementation uses Java `float`, and the
/// difference decides real cases: pure red sits equidistant between `red` and `dark_red`
/// in exact arithmetic, and only the rounding of 32-bit maths picks the same one.
struct HsvColor {
    hue: f32,
    saturation: f32,
    value: f32,
}

fn to_hsv(rgb: RgbColor) -> HsvColor {
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

    HsvColor {
        hue,
        saturation: if max == 0.0 { 0.0 } else { span / max },
        value: max,
    }
}

fn distance_squared(left: &HsvColor, right: &HsvColor) -> f32 {
    let hue_gap = (left.hue - right.hue).abs();
    let hue_distance = 3.0 * hue_gap.min(1.0 - hue_gap);
    let saturation_gap = left.saturation - right.saturation;
    let value_gap = left.value - right.value;

    hue_distance * hue_distance + saturation_gap * saturation_gap + value_gap * value_gap
}

/// Ties are broken by declaration order, as they are upstream: pure red sits exactly
/// between `red` and `dark_red`, and the client is sent `dark_red`.
fn nearest_named(rgb: RgbColor) -> NamedColor {
    let target = to_hsv(rgb);
    let mut best = NamedColor::Black;
    let mut best_distance = f32::MAX;

    for candidate in NamedColor::ALL {
        let distance = distance_squared(&target, &to_hsv(candidate.rgb()));
        if distance < best_distance {
            best_distance = distance;
            best = candidate;
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_an_exact_named_colour_value_when_reduced_then_that_name_is_chosen() {
        for named in NamedColor::ALL {
            assert_eq!(TextColor::Rgb(named.rgb()).to_named(), named, "{named:?}");
        }
    }

    #[test]
    fn given_a_colour_that_only_differs_in_saturation_then_hue_decides_the_name() {
        // Pale blue: nearer to grey by RGB distance, but the client should see blue.
        let pale_blue = RgbColor::from_packed(0xAAAAFF);

        assert_eq!(TextColor::Rgb(pale_blue).to_named(), NamedColor::Blue);
    }

    #[test]
    fn given_a_colour_equidistant_between_two_names_then_declaration_order_breaks_the_tie() {
        assert_eq!(
            TextColor::Rgb(RgbColor::from_packed(0xFF0000)).to_named(),
            NamedColor::DarkRed
        );
        assert_eq!(
            TextColor::Rgb(RgbColor::from_packed(0x00FF00)).to_named(),
            NamedColor::DarkGreen
        );
    }
}
