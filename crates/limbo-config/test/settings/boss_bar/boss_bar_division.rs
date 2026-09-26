use crate::settings::boss_bar::BossBarDivision;

#[test]
fn given_a_dashed_name_when_parsed_then_the_division_is_recognised() {
    assert_eq!("dashes_10".parse().ok(), Some(BossBarDivision::Dashes10));
    assert_eq!("DASHES_20".parse().ok(), Some(BossBarDivision::Dashes20));
}

#[test]
fn given_a_count_that_is_not_offered_when_parsed_then_it_is_rejected() {
    assert!("DASHES_7".parse::<BossBarDivision>().is_err());
}
