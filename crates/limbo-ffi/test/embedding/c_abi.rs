use crate::embedding::{StartStatus, cleanup_token, get_cancellation_token, start_app, stop_app};

#[test]
fn given_a_null_token_when_the_server_is_started_then_it_reports_rather_than_crashes() {
    let directory = c"/tmp";

    let status = unsafe { start_app(std::ptr::null_mut(), directory.as_ptr()) };

    assert_eq!(status, StartStatus::NullToken.code());
}

#[test]
fn given_a_null_directory_when_the_server_is_started_then_it_reports_rather_than_crashes() {
    let token = get_cancellation_token();

    let status = unsafe { start_app(token, std::ptr::null()) };

    unsafe { cleanup_token(token) };
    assert_eq!(status, StartStatus::InvalidConfigDirectory.code());
}

#[test]
fn given_a_null_pointer_when_the_host_stops_or_frees_it_then_nothing_happens() {
    unsafe { stop_app(std::ptr::null_mut()) };
    unsafe { cleanup_token(std::ptr::null_mut()) };
}

#[test]
fn given_a_token_when_it_is_created_and_freed_then_the_round_trip_holds() {
    let token = get_cancellation_token();
    assert!(!token.is_null());

    unsafe { stop_app(token) };
    unsafe { cleanup_token(token) };
}
