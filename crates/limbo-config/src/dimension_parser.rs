use limbo_world::DimensionType;

use crate::settings_error::SettingsError;

/// Reads the `dimension` setting.
///
/// `NETHER` and `END` are accepted alongside the registry names `THE_NETHER` and
/// `THE_END`, which is the shorthand the reference implementation allowed. A free
/// function rather than a `FromStr`, because the type belongs to another crate.
pub fn parse_dimension_type(value: &str) -> Result<DimensionType, SettingsError> {
    match value.to_ascii_uppercase().as_str() {
        "OVERWORLD" => Ok(DimensionType::Overworld),
        "THE_NETHER" | "NETHER" => Ok(DimensionType::TheNether),
        "THE_END" | "END" => Ok(DimensionType::TheEnd),
        _ => Err(SettingsError::UnknownDimension {
            value: value.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_a_registry_name_when_parsed_then_it_is_the_matching_dimension() {
        assert_eq!(
            parse_dimension_type("OVERWORLD").ok(),
            Some(DimensionType::Overworld)
        );
        assert_eq!(
            parse_dimension_type("THE_NETHER").ok(),
            Some(DimensionType::TheNether)
        );
        assert_eq!(
            parse_dimension_type("THE_END").ok(),
            Some(DimensionType::TheEnd)
        );
    }

    #[test]
    fn given_the_shorthand_names_when_parsed_then_they_mean_the_same_dimensions() {
        assert_eq!(
            parse_dimension_type("NETHER").ok(),
            Some(DimensionType::TheNether)
        );
        assert_eq!(
            parse_dimension_type("end").ok(),
            Some(DimensionType::TheEnd)
        );
    }

    #[test]
    fn given_a_dimension_that_does_not_exist_when_parsed_then_it_is_rejected() {
        assert!(parse_dimension_type("THE_AETHER").is_err());
        assert!(parse_dimension_type("").is_err());
    }
}
