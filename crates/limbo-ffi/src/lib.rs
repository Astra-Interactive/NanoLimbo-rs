//! ```c
//! CancellationToken* get_cancellation_token(void);
//! int32_t            start_app(CancellationToken* ptr, const char* config_dir);
//! void               stop_app(CancellationToken* ptr);
//! void               cleanup_token(CancellationToken* ptr);
//! ```

pub mod embedding {
    //! Running the server inside a host process: the C entry points, the token a host
    //! stops the server with, and the status codes it reads back.

    mod c_abi;
    mod cancellation_token;
    mod embedded_server;
    mod start_status;

    pub use c_abi::{cleanup_token, get_cancellation_token, start_app, stop_app};
    pub use cancellation_token::CancellationToken;
    pub use embedded_server::run;
    pub use start_status::StartStatus;
}

pub use embedding::{cleanup_token, get_cancellation_token, start_app, stop_app};
