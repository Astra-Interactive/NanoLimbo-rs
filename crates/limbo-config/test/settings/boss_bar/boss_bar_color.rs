use crate::settings::boss_bar::BossBarColor;

#[test]
fn given_a_name_in_any_case_when_parsed_then_the_colour_is_recognised() {
    for name in ["blue", "Blue", "BLUE"] {
        assert_eq!(name.parse().ok(), Some(BossBarColor::Blue));
    }
}

#[test]
fn given_an_unknown_name_when_parsed_then_it_is_rejected() {
    assert!("chartreuse".parse::<BossBarColor>().is_err());
    assert!("".parse::<BossBarColor>().is_err());
}
