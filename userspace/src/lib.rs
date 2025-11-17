//! WOS User Space
//!
//! User-level programs and services:
//! - Init process (PID 1)
//! - Shell
//! - User programs (echo, ls, ps, etc.)
//! - Vim editor

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod init;
pub mod programs;
pub mod programs_extra;
pub mod shell;
/// Vim modal text editor implementation
pub mod vim;

pub use init::{init_main_loop, InitProcess};
pub use programs::{echo_main_loop, ls_main_loop, ps_main_loop, vim_main_loop, Echo, Ls, Ps, Vim};
pub use programs_extra::{
    cat_main_loop, cp_main_loop, grep_main_loop, head_main_loop, mkdir_main_loop, mv_main_loop,
    rm_main_loop, tail_main_loop, wc_main_loop, Cat, Cp, Grep, Head, Mkdir, Mv, Rm, Tail, Wc,
};
pub use shell::{shell_main_loop, Command, Shell};
pub use vim::{VimBuffer, VimCommand, VimError, VimMode, VimState};

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
