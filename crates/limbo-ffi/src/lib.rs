//! A C ABI for running the limbo server inside a host process.
//!
//! The Java wrapper loads this library and drives it through the four functions below.
//! Running in the host's own process rather than as a child of it is what lets a proxy
//! start a limbo without a second executable to ship, supervise and reap.
//!
//! Everything here is a thin shell over [`embedded_server::run`]. Raw pointers are
//! checked and converted at once, and no logic lives in an `unsafe` function, so the
//! behaviour is tested through the safe types instead.
//!
//! The contract, as C:
//!
//! ```c
//! CancellationToken* get_cancellation_token(void);
//! int32_t            start_app(CancellationToken* ptr, const char* config_dir);
//! void               stop_app(CancellationToken* ptr);
//! void               cleanup_token(CancellationToken* ptr);
//! ```
//!
//! `start_app` blocks for the lifetime of the server and returns a [`StartStatus`] code.
//! `stop_app` is expected on another thread. The host owns the token and must free it
//! with `cleanup_token` exactly once, after `start_app` has returned.

use std::ffi::{CStr, c_char};
use std::path::Path;
use std::sync::Arc;

use tokio::sync::Notify;

pub mod cancellation_token;
pub mod embedded_server;
pub mod start_status;

use crate::cancellation_token::CancellationToken;
use crate::start_status::StartStatus;

/// Creates a token the host can later use to stop the server.
///
/// Returns null only if the allocation itself fails. The host must pass the result to
/// `cleanup_token` once it is done with it.
#[unsafe(no_mangle)]
pub extern "C" fn get_cancellation_token() -> *mut CancellationToken {
    Box::into_raw(Box::new(CancellationToken::new(Arc::new(Notify::new()))))
}

/// Runs the server until the token is cancelled, blocking the calling thread.
///
/// Returns a [`StartStatus`] code: `0` on a clean shutdown, non-zero on failure.
///
/// # Safety
///
/// `token` must be a pointer returned by `get_cancellation_token` that has not yet been
/// passed to `cleanup_token`, and `configuration_directory` must be a null-terminated C
/// string that stays valid for the duration of this call. Both may be null, which is
/// reported rather than dereferenced.
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

/// Asks a running server to stop. Safe to call before it has finished starting, and safe
/// to call more than once.
///
/// # Safety
///
/// `token` must be a pointer returned by `get_cancellation_token` that has not yet been
/// passed to `cleanup_token`. Null is ignored.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn stop_app(token: *mut CancellationToken) {
    if let Some(token) = unsafe { token.as_ref() } {
        token.cancel();
    }
}

/// Releases a token.
///
/// # Safety
///
/// `token` must be a pointer returned by `get_cancellation_token`, and must not be used
/// again afterwards - including by a `stop_app` still in flight on another thread. Null
/// is ignored, and no pointer may be passed here twice.
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

    /// A host that mismanages its pointers gets a code back, not a crash. This is the
    /// one thing the safe layer underneath cannot be made to prove.
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
