/// Why a connection ended. Every variant is ordinary; none is a server fault.
#[derive(Debug)]
pub(crate) enum Ending {
    ClientClosed,
    Timeout,
    Refused,
    Malformed,
    TooMuchTraffic,
    ServerStopping,
}
