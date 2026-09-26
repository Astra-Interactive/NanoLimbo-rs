/// Why the comparison could not be run, or could not be run meaningfully.
#[derive(Debug, thiserror::Error)]
pub enum BenchError {
    #[error("expected a target of the form name=address[@pid], got {argument:?}")]
    MalformedTarget { argument: String },

    #[error("{flag} expects a value")]
    MissingValue { flag: String },

    #[error("{flag} does not understand {value:?}")]
    UnreadableValue { flag: String, value: String },

    #[error("unknown option {flag:?}")]
    UnknownOption { flag: String },

    #[error("no targets given; pass at least one as name=address[@pid]")]
    NoTargets,

    #[error("protocol {number} is not one this build speaks")]
    UnsupportedProtocol { number: i32 },
}
