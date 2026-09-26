use std::ffi::{CStr, c_char};
use std::path::Path;
use std::sync::Arc;

use tokio::sync::Notify;

use crate::embedding::{CancellationToken, StartStatus, run};

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

    run(token, Path::new(directory)).code()
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
