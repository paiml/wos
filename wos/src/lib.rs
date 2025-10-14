//! WOS - WASM Operating System
//!
//! Main entry point integrating kernel and userspace for WASM execution.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

/// Get WOS version
#[wasm_bindgen]
pub fn wos_version() -> String {
    format!(
        "WOS v{} (kernel: {}, userspace: {})",
        env!("CARGO_PKG_VERSION"),
        wos_kernel::kernel_version(),
        wos_userspace::userspace_version()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wos_version() {
        let version = wos_version();
        assert!(version.starts_with("WOS v"));
    }
}
