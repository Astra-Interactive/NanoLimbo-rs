use crate::play::PlayerAbilities;
use crate::play::player_abilities::FLAG_FLYING;

#[test]
fn given_every_ability_when_flags_are_built_then_each_claims_its_own_bit() {
    let abilities = PlayerAbilities {
        invincible: true,
        can_fly: true,
        flying: true,
        creative: true,
        flying_speed: 0.0,
        field_of_view: 0.1,
    };

    assert_eq!(abilities.flags(), 0x0F);
}

#[test]
fn given_only_flying_when_flags_are_built_then_the_can_fly_bit_stays_clear() {
    let abilities = PlayerAbilities {
        invincible: false,
        can_fly: false,
        flying: true,
        creative: false,
        flying_speed: 0.0,
        field_of_view: 0.1,
    };

    assert_eq!(abilities.flags(), FLAG_FLYING);
}
