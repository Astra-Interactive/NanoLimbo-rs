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
    /// Distance is measured the way the reference implementation measures it: squared
    /// Euclidean distance in RGB, with the red channel weighted by the mean of the two
    /// reds. Any other metric picks a different colour for some inputs, so this is
    /// wire-visible for pre-1.16 clients.
    pub fn to_named(self) -> NamedColor {
        match self {
            Self::Named(named) => named,
            Self::Rgb(rgb) => nearest_named(rgb),
        }
    }
}

fn distance_squared(left: RgbColor, right: RgbColor) -> f64 {
    let mean_red = (f64::from(left.red) + f64::from(right.red)) / 2.0;
    let delta_red = f64::from(left.red) - f64::from(right.red);
    let delta_green = f64::from(left.green) - f64::from(right.green);
    let delta_blue = f64::from(left.blue) - f64::from(right.blue);

    (2.0 + mean_red / 256.0) * delta_red * delta_red
        + 4.0 * delta_green * delta_green
        + (2.0 + (255.0 - mean_red) / 256.0) * delta_blue * delta_blue
}

fn nearest_named(rgb: RgbColor) -> NamedColor {
    let mut best = NamedColor::White;
    let mut best_distance = f64::MAX;

    for candidate in NamedColor::ALL {
        let distance = distance_squared(rgb, candidate.rgb());
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
    fn given_a_colour_between_names_when_reduced_then_the_nearer_name_wins() {
        let almost_red = RgbColor::from_packed(0xFE5A5A);

        assert_eq!(TextColor::Rgb(almost_red).to_named(), NamedColor::Red);
    }
}
