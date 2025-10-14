//! WOS User Space
//!
//! User-level programs and services:
//! - Init process (PID 1)
//! - Shell
//! - User programs (echo, ls, ps, etc.)

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod init;

pub use init::{init_main_loop, InitProcess};

/// Placeholder for userspace implementation
pub fn userspace_version() -> &'static str {
    "0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_userspace_version() {
        assert_eq!(userspace_version(), "0.1.0");
    }
}
