use limbo_world::DimensionType;

use crate::settings::SettingsError;

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
