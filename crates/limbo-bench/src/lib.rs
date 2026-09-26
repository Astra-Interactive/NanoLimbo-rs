//! The pieces of the load generator: what it was asked to measure, the client it logs in
//! with, how it reads a server's memory, and the table it prints.

mod client {
    //! A minimal client that logs one player in and holds the connection open.

    mod login_client;

    pub use login_client::LoginClient;
}

mod memory {
    //! Where a server's memory is read from, and the reading itself.

    mod memory_source;
    mod resident_memory;

    pub use memory_source::MemorySource;
    pub use resident_memory::ResidentMemory;
}

mod options {
    //! The command line: which servers to measure, and how hard to push them.

    mod bench_error;
    pub(crate) mod bench_options;
    mod bench_target;

    pub use bench_error::BenchError;
    pub use bench_options::BenchOptions;
    pub use bench_target::BenchTarget;
}

mod report {
    //! What each server did, laid out side by side.

    mod target_report;

    pub use target_report::TargetReport;
}

pub use client::LoginClient;
pub use memory::{MemorySource, ResidentMemory};
pub use options::{BenchError, BenchOptions, BenchTarget};
pub use report::TargetReport;

#[cfg(test)]
#[path = "../test/lib.rs"]
mod test;
