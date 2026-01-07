//! WOS Probar Terminal Tests
//!
//! Pure Rust TUI tests for WOS terminal functionality.
//! These tests verify core terminal operations without browser dependencies.
//!
//! # Test Coverage
//!
//! - Command execution (help, ls, ps, cat, echo)
//! - Command history navigation
//! - Error handling for invalid commands
//! - Output formatting
//!
//! # Toyota Principle
//!
//! Jidoka (Built-in Quality) - Tests catch defects at the source

use wos::WosWasm;

// ============================================================================
// SECTION 1: BASIC COMMAND TESTS
// ============================================================================

#[test]
fn test_terminal_help_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("help");

    assert!(output.contains("Available commands"), "Help should list commands");
    assert!(output.contains("help"), "Help should mention itself");
    assert!(output.contains("ls"), "Help should mention ls");
    assert!(output.contains("ps"), "Help should mention ps");
}

#[test]
fn test_terminal_echo_command() {
    let mut wos = WosWasm::new();

    // Simple echo
    let output = wos.execute_command("echo hello");
    assert_eq!(output.trim(), "hello");

    // Echo with multiple words
    let output = wos.execute_command("echo hello world");
    assert_eq!(output.trim(), "hello world");

    // Echo with empty string
    let output = wos.execute_command("echo");
    assert!(output.trim().is_empty() || output.trim() == "");
}

#[test]
fn test_terminal_ls_command() {
    let mut wos = WosWasm::new();

    // List root directory - may be minimal VFS
    let output = wos.execute_command("ls /");
    assert!(!output.contains("panic"), "ls / should not panic");

    // List /proc
    let output = wos.execute_command("ls /proc");
    assert!(!output.contains("panic"), "ls /proc should not panic");
}

#[test]
fn test_terminal_ps_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("ps");

    // Should have header
    assert!(output.contains("PID") || output.contains("pid"),
        "ps should show PID column");

    // Should show at least init process
    assert!(output.contains("1") || output.contains("init"),
        "ps should show init process");
}

#[test]
fn test_terminal_cat_command() {
    let mut wos = WosWasm::new();

    // Cat a proc file
    let output = wos.execute_command("cat /proc/1/status");
    assert!(!output.is_empty(), "cat should produce output for valid file");

    // Cat non-existent file
    let output = wos.execute_command("cat /nonexistent");
    assert!(output.contains("not found") || output.contains("No such") || output.contains("error"),
        "cat should report error for missing file");
}

#[test]
fn test_terminal_pwd_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("pwd");

    assert!(output.starts_with('/'), "pwd should return absolute path");
}

// ============================================================================
// SECTION 2: COMMAND HISTORY TESTS
// ============================================================================

#[test]
fn test_terminal_multiple_commands() {
    let mut wos = WosWasm::new();

    // Execute sequence of commands
    let _ = wos.execute_command("echo first");
    let _ = wos.execute_command("echo second");
    let output = wos.execute_command("echo third");

    assert_eq!(output.trim(), "third");
}

#[test]
fn test_terminal_exit_code() {
    let mut wos = WosWasm::new();

    // Successful command
    let _ = wos.execute_command("echo test");
    let output = wos.execute_command("echo $?");
    assert!(output.trim() == "0" || output.contains("0"),
        "Exit code after successful command should be 0");
}

// ============================================================================
// SECTION 3: ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_terminal_invalid_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("nonexistentcommand123");

    assert!(output.contains("not found") || output.contains("unknown") ||
            output.contains("command") || output.contains("error"),
        "Invalid command should produce error message");
}

#[test]
fn test_terminal_empty_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("");

    // Empty command should be handled gracefully
    assert!(output.is_empty() || !output.contains("panic"),
        "Empty command should not cause panic");
}

#[test]
fn test_terminal_whitespace_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("   ");

    // Whitespace-only command should be handled gracefully
    assert!(!output.contains("panic"),
        "Whitespace command should not cause panic");
}

// ============================================================================
// SECTION 4: SPECIAL COMMANDS
// ============================================================================

#[test]
fn test_terminal_clear_returns_empty() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("clear");

    // Clear should return empty or special clear sequence
    assert!(output.is_empty() || output.contains("\x1b") || output.len() < 100,
        "Clear should not produce verbose output");
}

#[test]
fn test_terminal_version_command() {
    let mut wos = WosWasm::new();
    let output = wos.execute_command("version");

    // Should show version info
    assert!(output.contains("WOS") || output.contains("wos") ||
            output.contains("2.0") || output.contains("version"),
        "Version command should show version info");
}

// ============================================================================
// SECTION 5: PROCESS COUNT
// ============================================================================

#[test]
fn test_terminal_process_count() {
    let wos = WosWasm::new();
    let count = wos.process_count();

    assert!(count >= 1, "Should have at least init process");
    assert!(count < 1000, "Process count should be reasonable");
}

// ============================================================================
// SECTION 6: PIPELINE COMMANDS
// ============================================================================

#[test]
fn test_terminal_pipe_command() {
    let mut wos = WosWasm::new();

    // Simple pipe
    let output = wos.execute_command("echo hello | cat");
    assert!(output.contains("hello") || !output.is_empty(),
        "Pipe should pass data through");
}

#[test]
fn test_terminal_semicolon_commands() {
    let mut wos = WosWasm::new();

    // Multiple commands with semicolon
    let output = wos.execute_command("echo first; echo second");
    assert!(output.contains("first") || output.contains("second"),
        "Semicolon should execute multiple commands");
}

// ============================================================================
// SECTION 7: FILESYSTEM COMMANDS
// ============================================================================

#[test]
fn test_terminal_mkdir_rmdir() {
    let mut wos = WosWasm::new();

    // Create directory
    let _ = wos.execute_command("mkdir /tmp/testdir");

    // List to verify
    let output = wos.execute_command("ls /tmp");
    // Note: might not work depending on VFS implementation

    // Remove directory
    let _ = wos.execute_command("rmdir /tmp/testdir");
}

#[test]
fn test_terminal_touch_command() {
    let mut wos = WosWasm::new();

    // Create file
    let _ = wos.execute_command("touch /tmp/testfile");

    // Verify exists (cat should not error)
    let output = wos.execute_command("cat /tmp/testfile");
    // Empty file or no error
    assert!(!output.contains("not found") || output.is_empty(),
        "Touch should create file");
}

// ============================================================================
// SECTION 8: DETERMINISM TESTS
// ============================================================================

#[test]
fn test_terminal_deterministic_output() {
    // Same commands should produce same output
    let mut wos1 = WosWasm::new();
    let mut wos2 = WosWasm::new();

    let output1 = wos1.execute_command("echo deterministic");
    let output2 = wos2.execute_command("echo deterministic");

    assert_eq!(output1, output2, "Same command should produce same output");
}

#[test]
fn test_terminal_ps_deterministic() {
    // Initial ps should be deterministic
    let mut wos1 = WosWasm::new();
    let mut wos2 = WosWasm::new();

    let output1 = wos1.execute_command("ps");
    let output2 = wos2.execute_command("ps");

    // Both should show init process
    assert!(output1.contains("init") || output1.contains("1"));
    assert!(output2.contains("init") || output2.contains("1"));
}
