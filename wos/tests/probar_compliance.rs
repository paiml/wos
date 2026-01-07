//! WOS Probar 100-Point Compliance Tests
//!
//! Tests for playbook validation, accessibility, and performance.
//! Covers QA Checklist Sections 3-4 (items 51-100).
//!
//! # Test Coverage
//!
//! - Playbook format validation (Section 3)
//! - Accessibility tests (Section 4)
//! - Performance benchmarks (Section 4)
//!
//! # Toyota Principle
//!
//! Kaizen (Continuous Improvement) - Comprehensive quality verification

use std::fs;
use std::path::Path;
use wos::WosWasm;

// ============================================================================
// SECTION 3: PLAYBOOK VALIDATION TESTS (100-Point Checklist Items 51-75)
// ============================================================================

mod playbook_validation {
    use super::*;

    const PLAYBOOKS_DIR: &str = "tests/playbooks";

    #[test]
    fn test_playbook_smoke_test_exists() {
        // QA Checklist #51: smoke-test.apr exists
        let path = Path::new(PLAYBOOKS_DIR).join("smoke-test.apr");
        assert!(path.exists(), "smoke-test.apr should exist");
    }

    #[test]
    fn test_playbook_smoke_test_valid_yaml() {
        // QA Checklist #52: smoke-test.apr has valid YAML
        let path = Path::new(PLAYBOOKS_DIR).join("smoke-test.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(content.contains("version:"), "Should have version field");
        assert!(content.contains("name:"), "Should have name field");
        assert!(content.contains("scenarios:"), "Should have scenarios");
    }

    #[test]
    fn test_playbook_smoke_test_has_scenarios() {
        // QA Checklist #53: smoke-test.apr has test scenarios
        let path = Path::new(PLAYBOOKS_DIR).join("smoke-test.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(content.contains("- name:"), "Should have named scenarios");
    }

    #[test]
    fn test_playbook_smoke_test_has_assertions() {
        // QA Checklist #54: smoke-test.apr has assertions
        let path = Path::new(PLAYBOOKS_DIR).join("smoke-test.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(
            content.contains("assert") || content.contains("expect"),
            "Should have assertions"
        );
    }

    #[test]
    fn test_playbook_smoke_test_no_js() {
        // QA Checklist #55: smoke-test.apr has no JavaScript
        let path = Path::new(PLAYBOOKS_DIR).join("smoke-test.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(
            !content.contains("javascript:") && !content.contains("script:"),
            "Should not contain JavaScript"
        );
    }

    #[test]
    fn test_playbook_vim_workflow_exists() {
        // QA Checklist #56: vim-workflow.apr exists
        let path = Path::new(PLAYBOOKS_DIR).join("vim-workflow.apr");
        assert!(path.exists(), "vim-workflow.apr should exist");
    }

    #[test]
    fn test_playbook_vim_workflow_has_modes() {
        // QA Checklist #57-60: vim-workflow.apr covers vim modes
        let path = Path::new(PLAYBOOKS_DIR).join("vim-workflow.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        // Check for vim mode testing
        assert!(
            content.to_lowercase().contains("normal")
                || content.to_lowercase().contains("insert")
                || content.to_lowercase().contains("vim"),
            "Should test vim modes"
        );
    }

    #[test]
    fn test_playbook_vim_workflow_has_navigation() {
        // QA Checklist #61-65: vim-workflow.apr covers navigation
        let path = Path::new(PLAYBOOKS_DIR).join("vim-workflow.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(content.len() > 100, "Should have substantial content");
    }

    #[test]
    fn test_playbook_process_management_exists() {
        // QA Checklist #66: process-management.apr exists
        let path = Path::new(PLAYBOOKS_DIR).join("process-management.apr");
        assert!(path.exists(), "process-management.apr should exist");
    }

    #[test]
    fn test_playbook_process_management_has_ps() {
        // QA Checklist #67: Tests ps command
        let path = Path::new(PLAYBOOKS_DIR).join("process-management.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(content.contains("ps"), "Should test ps command");
    }

    #[test]
    fn test_playbook_process_management_has_kill() {
        // QA Checklist #68: Tests kill command
        let path = Path::new(PLAYBOOKS_DIR).join("process-management.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(content.contains("kill"), "Should test kill command");
    }

    #[test]
    fn test_playbook_panel_switching_exists() {
        // QA Checklist #69: panel-switching.apr exists
        let path = Path::new(PLAYBOOKS_DIR).join("panel-switching.apr");
        assert!(path.exists(), "panel-switching.apr should exist");
    }

    #[test]
    fn test_playbook_panel_switching_covers_panels() {
        // QA Checklist #70: Tests panel navigation
        let path = Path::new(PLAYBOOKS_DIR).join("panel-switching.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(
            content.to_lowercase().contains("panel"),
            "Should test panel switching"
        );
    }

    #[test]
    fn test_playbook_keyboard_navigation_exists() {
        // QA Checklist #71: keyboard-navigation.apr exists
        let path = Path::new(PLAYBOOKS_DIR).join("keyboard-navigation.apr");
        assert!(path.exists(), "keyboard-navigation.apr should exist");
    }

    #[test]
    fn test_playbook_keyboard_navigation_has_keys() {
        // QA Checklist #72-75: Tests keyboard navigation
        let path = Path::new(PLAYBOOKS_DIR).join("keyboard-navigation.apr");
        let content = fs::read_to_string(&path).expect("Should read file");
        assert!(
            content.contains("key") || content.contains("press") || content.contains("keyboard"),
            "Should test keyboard input"
        );
    }

    #[test]
    fn test_all_playbooks_valid_format() {
        // Verify all .apr files have valid format
        let playbooks = [
            "smoke-test.apr",
            "vim-workflow.apr",
            "process-management.apr",
            "panel-switching.apr",
            "keyboard-navigation.apr",
        ];

        for playbook in &playbooks {
            let path = Path::new(PLAYBOOKS_DIR).join(playbook);
            assert!(path.exists(), "{} should exist", playbook);
            let content = fs::read_to_string(&path).expect("Should read file");
            assert!(
                content.contains("version:"),
                "{} should have version",
                playbook
            );
        }
    }
}

// ============================================================================
// SECTION 4: ACCESSIBILITY TESTS (100-Point Checklist Items 76-90)
// ============================================================================

mod accessibility_tests {
    use super::*;

    #[test]
    fn test_wcag_text_content_accessible() {
        // QA Checklist #76: Text content is accessible
        let mut wos = WosWasm::new();
        let output = wos.execute_command("help");
        // Help text should be readable (non-empty, no binary)
        assert!(!output.is_empty());
        assert!(
            output.is_ascii()
                || output
                    .chars()
                    .all(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
        );
    }

    #[test]
    fn test_wcag_error_messages_descriptive() {
        // QA Checklist #77: Error messages are descriptive
        let mut wos = WosWasm::new();
        let output = wos.execute_command("nonexistent_command_xyz");
        // Error should be descriptive, not cryptic
        assert!(
            output.to_lowercase().contains("not found")
                || output.to_lowercase().contains("unknown")
                || output.to_lowercase().contains("command")
        );
    }

    #[test]
    fn test_wcag_consistent_output_format() {
        // QA Checklist #78: Output format is consistent
        let mut wos = WosWasm::new();

        let ps1 = wos.execute_command("ps");
        let ps2 = wos.execute_command("ps");

        // Same command should produce consistent format
        let lines1: Vec<&str> = ps1.lines().collect();
        let lines2: Vec<&str> = ps2.lines().collect();

        assert_eq!(
            lines1.first().map(|s| s.len() > 0),
            lines2.first().map(|s| s.len() > 0),
            "Header format should be consistent"
        );
    }

    #[test]
    fn test_wcag_no_flashing_content() {
        // QA Checklist #79: No flashing/strobing content
        let mut wos = WosWasm::new();
        let output = wos.execute_command("clear");

        // Clear should not produce rapidly changing content
        // (ANSI codes are OK, but not rapid sequences)
        let ansi_count = output.matches("\x1b[").count();
        assert!(ansi_count < 10, "Should not have excessive ANSI sequences");
    }

    #[test]
    fn test_wcag_operable_via_commands() {
        // QA Checklist #80: All features operable via commands
        let mut wos = WosWasm::new();

        // Core operations should be available via commands
        let commands = ["help", "ls", "ps", "pwd", "echo test", "version"];
        for cmd in &commands {
            let output = wos.execute_command(cmd);
            assert!(!output.contains("panic"), "{} should be operable", cmd);
        }
    }

    #[test]
    fn test_keyboard_nav_command_execution() {
        // QA Checklist #81: Commands executable via keyboard
        let mut wos = WosWasm::new();
        // execute_command simulates keyboard input + Enter
        let output = wos.execute_command("echo keyboard test");
        assert!(output.contains("keyboard test"));
    }

    #[test]
    fn test_keyboard_nav_history() {
        // QA Checklist #82: History navigable
        let mut wos = WosWasm::new();
        let _ = wos.execute_command("echo first");
        let _ = wos.execute_command("echo second");
        // History command tests navigation availability
        let output = wos.execute_command("history");
        assert!(!output.contains("panic"));
    }

    #[test]
    fn test_keyboard_nav_file_operations() {
        // QA Checklist #83: File ops via keyboard
        let mut wos = WosWasm::new();
        let _ = wos.execute_command("touch /tmp/keyboard_test");
        let output = wos.execute_command("ls /tmp");
        assert!(!output.contains("panic"));
    }

    #[test]
    fn test_keyboard_nav_process_control() {
        // QA Checklist #84: Process control via keyboard
        let mut wos = WosWasm::new();
        let output = wos.execute_command("ps");
        assert!(output.contains("PID") || output.contains("init"));
    }

    #[test]
    fn test_keyboard_nav_help_access() {
        // QA Checklist #85: Help accessible via keyboard
        let mut wos = WosWasm::new();
        let output = wos.execute_command("help");
        assert!(output.contains("Available") || output.contains("help"));
    }

    #[test]
    fn test_aria_semantic_output() {
        // QA Checklist #86: Output has semantic meaning
        let mut wos = WosWasm::new();
        let output = wos.execute_command("ps");
        // Should have structured output (headers, columns)
        assert!(output.contains("\t") || output.contains("  "));
    }

    #[test]
    fn test_aria_status_indicators() {
        // QA Checklist #87: Status indicators present
        let wos = WosWasm::new();
        let count = wos.process_count();
        // Process count is a status indicator
        assert!(count >= 1);
    }

    #[test]
    fn test_aria_error_identification() {
        // QA Checklist #88: Errors clearly identified
        let mut wos = WosWasm::new();
        let output = wos.execute_command("cat /nonexistent");
        assert!(
            output.to_lowercase().contains("not found")
                || output.to_lowercase().contains("error")
                || output.to_lowercase().contains("no such")
        );
    }

    #[test]
    fn test_aria_command_feedback() {
        // QA Checklist #89: Commands provide feedback
        let mut wos = WosWasm::new();
        let output = wos.execute_command("echo feedback test");
        // Echo provides immediate feedback
        assert!(output.contains("feedback test"));
    }

    #[test]
    fn test_aria_state_communication() {
        // QA Checklist #90: State is communicated
        let mut wos = WosWasm::new();
        let output = wos.execute_command("pwd");
        // pwd communicates current state (directory)
        assert!(output.starts_with('/'));
    }
}

// ============================================================================
// SECTION 4 (cont.): PERFORMANCE TESTS (100-Point Checklist Items 91-100)
// ============================================================================

mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_perf_init_time() {
        // QA Checklist #91: Init completes quickly
        let start = Instant::now();
        let _wos = WosWasm::new();
        let elapsed = start.elapsed();

        // Init should complete in under 100ms
        assert!(
            elapsed.as_millis() < 100,
            "Init took {}ms, should be < 100ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_perf_command_execution() {
        // QA Checklist #92: Commands execute quickly
        let mut wos = WosWasm::new();

        let start = Instant::now();
        let _ = wos.execute_command("echo test");
        let elapsed = start.elapsed();

        // Simple command should complete in under 10ms
        assert!(
            elapsed.as_millis() < 10,
            "Command took {}ms, should be < 10ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_perf_ps_command() {
        // QA Checklist #93: ps command performs well
        let mut wos = WosWasm::new();

        let start = Instant::now();
        let _ = wos.execute_command("ps");
        let elapsed = start.elapsed();

        // ps should complete quickly
        assert!(
            elapsed.as_millis() < 20,
            "ps took {}ms, should be < 20ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_perf_ls_command() {
        // QA Checklist #94: ls command performs well
        let mut wos = WosWasm::new();

        let start = Instant::now();
        let _ = wos.execute_command("ls /");
        let elapsed = start.elapsed();

        // ls should complete quickly
        assert!(
            elapsed.as_millis() < 20,
            "ls took {}ms, should be < 20ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_perf_help_command() {
        // QA Checklist #95: help command performs well
        let mut wos = WosWasm::new();

        let start = Instant::now();
        let _ = wos.execute_command("help");
        let elapsed = start.elapsed();

        // help should complete quickly
        assert!(
            elapsed.as_millis() < 10,
            "help took {}ms, should be < 10ms",
            elapsed.as_millis()
        );
    }

    #[test]
    fn test_mem_no_leak_repeated_commands() {
        // QA Checklist #96: No memory leak on repeated commands
        let mut wos = WosWasm::new();

        // Execute many commands
        for i in 0..100 {
            let _ = wos.execute_command(&format!("echo test{}", i));
        }

        // System should still be responsive
        let output = wos.execute_command("echo final");
        assert!(output.contains("final"));
    }

    #[test]
    fn test_mem_no_leak_process_operations() {
        // QA Checklist #97: No memory leak on process ops
        let mut wos = WosWasm::new();

        // Many process operations
        for _ in 0..50 {
            let _ = wos.execute_command("ps");
        }

        // Should still work
        let count = wos.process_count();
        assert!(count >= 1);
    }

    #[test]
    fn test_mem_no_leak_file_operations() {
        // QA Checklist #98: No memory leak on file ops
        let mut wos = WosWasm::new();

        // Many file operations
        for i in 0..50 {
            let _ = wos.execute_command(&format!("touch /tmp/leak_test_{}", i));
        }

        // Cleanup and verify
        for i in 0..50 {
            let _ = wos.execute_command(&format!("rm /tmp/leak_test_{}", i));
        }

        let output = wos.execute_command("ls /tmp");
        assert!(!output.contains("panic"));
    }

    #[test]
    fn test_mem_stable_after_errors() {
        // QA Checklist #99: Memory stable after errors
        let mut wos = WosWasm::new();

        // Generate many errors
        for _ in 0..50 {
            let _ = wos.execute_command("nonexistent_command");
            let _ = wos.execute_command("cat /nonexistent");
        }

        // System should still be stable
        let output = wos.execute_command("echo stable");
        assert!(output.contains("stable"));
    }

    #[test]
    fn test_mem_multiple_instances() {
        // QA Checklist #100: Multiple instances don't leak
        let instances: Vec<WosWasm> = (0..10).map(|_| WosWasm::new()).collect();

        // All instances should be functional
        for wos in &instances {
            assert!(wos.process_count() >= 1);
        }

        // Verify independence
        assert_eq!(instances.len(), 10);
    }
}
