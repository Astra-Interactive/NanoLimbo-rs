use crate::chat::{NamedColor, RgbColor, TextColor};

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
