//! WOS Microkernel
//!
//! Minimal trusted computing base providing:
//! - Process scheduling
//! - Memory management
//! - System call dispatch
//! - IPC primitives

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// Placeholder for kernel implementation
pub fn kernel_version() -> &'static str {
    "0.1.0"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_version() {
        assert_eq!(kernel_version(), "0.1.0");
    }
}
