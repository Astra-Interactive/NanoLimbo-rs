use crate::chat::HsvColor;
use crate::chat::NamedColor;
use crate::chat::RgbColor;

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

/// Ties are broken by declaration order, as they are upstream: pure red sits exactly
/// between `red` and `dark_red`, and the client is sent `dark_red`.
fn nearest_named(rgb: RgbColor) -> NamedColor {
    let target = HsvColor::from_rgb(rgb);
    let mut best = NamedColor::Black;
    let mut best_distance = f32::MAX;

    for candidate in NamedColor::ALL {
        let distance = target.distance_squared(&HsvColor::from_rgb(candidate.rgb()));
        if distance < best_distance {
            best_distance = distance;
            best = candidate;
        }
    }

    best
}
