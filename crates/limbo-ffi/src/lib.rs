//! ```c
//! CancellationToken* get_cancellation_token(void);
//! int32_t            start_app(CancellationToken* ptr, const char* config_dir);
//! void               stop_app(CancellationToken* ptr);
//! void               cleanup_token(CancellationToken* ptr);
//! ```

use std::ffi::{CStr, c_char};
use std::path::Path;
use std::sync::Arc;

use tokio::sync::Notify;

pub mod cancellation_token;
pub mod embedded_server;
pub mod start_status;

use crate::cancellation_token::CancellationToken;
use crate::start_status::StartStatus;

#[unsafe(no_mangle)]
pub extern "C" fn get_cancellation_token() -> *mut CancellationToken {
    Box::into_raw(Box::new(CancellationToken::new(Arc::new(Notify::new()))))
}

/// Blocks the calling thread until the token is cancelled.
///
/// # Safety
///
/// Pointers must come from `get_cancellation_token` and a null-terminated C string that
/// outlives the call. Null is reported, not dereferenced.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn start_app(
    token: *mut CancellationToken,
    configuration_directory: *const c_char,
) -> i32 {
    let Some(token) = (unsafe { token.as_ref() }) else {
        return StartStatus::NullToken.code();
    };
    if configuration_directory.is_null() {
        return StartStatus::InvalidConfigDirectory.code();
    }

    let Ok(directory) = (unsafe { CStr::from_ptr(configuration_directory) }).to_str() else {
        return StartStatus::InvalidConfigDirectory.code();
    };

    embedded_server::run(token, Path::new(directory)).code()
}

/// Safe before the server has started, and more than once.
///
/// # Safety
///
/// `token` must not have been passed to `cleanup_token`. Null is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_app(token: *mut CancellationToken) {
    if let Some(token) = unsafe { token.as_ref() } {
        token.cancel();
    }
}

/// # Safety
///
/// Never twice, and never while a `stop_app` is still in flight on another thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn cleanup_token(token: *mut CancellationToken) {
    if token.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(token) });
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
