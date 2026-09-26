use crate::settings::boss_bar::BossBarHealth;

#[test]
fn given_the_ends_of_the_range_when_validated_then_both_are_accepted() {
    assert_eq!(
        BossBarHealth::new(0.0).map(BossBarHealth::value).ok(),
        Some(0.0)
    );
    assert_eq!(
        BossBarHealth::new(1.0).map(BossBarHealth::value).ok(),
        Some(1.0)
    );
}

#[test]
fn given_a_value_just_outside_the_range_when_validated_then_it_is_rejected() {
    assert!(BossBarHealth::new(-0.1).is_err());
    assert!(BossBarHealth::new(1.1).is_err());
}

/// Java compares with `<` and `>`, both of which are false for a NaN, so a boss bar
/// configured with `.nan` reached the client and drew nothing.
#[test]
fn given_a_nan_when_validated_then_it_is_rejected() {
    assert!(BossBarHealth::new(f32::NAN).is_err());
}
