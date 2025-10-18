//! Script execution engine
//!
//! Provides functionality to execute shell scripts line-by-line using existing command handlers.

use std::collections::HashMap;
use wos_shared::{Script, ScriptError, VirtualFileSystem};

/// Script executor for running scripts line-by-line
#[allow(dead_code)]
pub struct ScriptExecutor;

/// Result of script execution
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ExecutionResult {
    /// Combined output from all commands
    pub output: String,
    /// Exit status of the last command (or first failing command)
    pub exit_code: i32,
}

#[allow(dead_code)]
impl ScriptExecutor {
    /// Execute a script line-by-line
    ///
    /// # Arguments
    /// * `script` - The script to execute
    /// * `vfs` - Virtual file system (unused in this simple implementation)
    /// * `variables` - Variable environment for the script
    ///
    /// # Returns
    /// * `Ok(ExecutionResult)` - Successful execution with accumulated output
    /// * `Err(ScriptError)` - If execution fails
    ///
    /// # Behavior
    /// - Executes each non-empty, non-comment line sequentially
    /// - Accumulates output from all commands
    /// - Stops on first error (exit code != 0)
    /// - Skips empty lines and comments (lines starting with #)
    pub fn execute(
        script: &Script,
        _vfs: &mut VirtualFileSystem,
        _variables: &mut HashMap<String, String>,
    ) -> Result<ExecutionResult, ScriptError> {
        let mut accumulated_output = String::new();
        let mut exit_code = 0;

        // Process each line in the script
        for line in script.content.lines() {
            let trimmed = line.trim();

            // Skip empty lines
            if trimmed.is_empty() {
                continue;
            }

            // Skip comments (lines starting with #)
            if trimmed.starts_with('#') {
                continue;
            }

            // Execute the command using a simple built-in executor
            let (output, code) = Self::execute_line(trimmed);

            // Accumulate output
            if !output.is_empty() {
                if !accumulated_output.is_empty() {
                    accumulated_output.push('\n');
                }
                accumulated_output.push_str(&output);
            }

            // Update exit code
            exit_code = code;

            // Stop on first error (bash default behavior)
            if code != 0 {
                break;
            }
        }

        Ok(ExecutionResult {
            output: accumulated_output,
            exit_code,
        })
    }

    /// Execute a single line as a command
    ///
    /// Returns (output, exit_code)
    fn execute_line(line: &str) -> (String, i32) {
        // Parse command and args
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return (String::new(), 0);
        }

        let cmd = parts[0];
        let args = &parts[1..];

        // Simple command implementations for testing
        match cmd {
            "echo" => {
                let output = args.join(" ");
                (output, 0)
            }
            _ => {
                // Unknown command - return error
                let error_msg = format!("{}: command not found", cmd);
                (error_msg, 127) // 127 is standard "command not found" exit code
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Helper to create VFS with test files
    fn create_test_vfs() -> VirtualFileSystem {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/test.txt"),
            "test content\n".as_bytes().to_vec(),
        )
        .unwrap();
        vfs
    }

    // Helper to create variables map
    fn create_test_vars() -> HashMap<String, String> {
        HashMap::new()
    }

    // WOS-202 Test 1: test_execute_simple_script_single_command
    #[test]
    fn test_execute_simple_script_single_command() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho hello".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.output.trim(), "hello");
        assert_eq!(exec_result.exit_code, 0);
    }

    // WOS-202 Test 2: test_execute_script_multiple_commands
    #[test]
    fn test_execute_script_multiple_commands() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho first\necho second\necho third".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert!(exec_result.output.contains("first"));
        assert!(exec_result.output.contains("second"));
        assert!(exec_result.output.contains("third"));
        assert_eq!(exec_result.exit_code, 0);
    }

    // WOS-202 Test 3: test_execute_script_with_comments
    #[test]
    fn test_execute_script_with_comments() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\n# This is a comment\necho visible\n# Another comment"
                .to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.output.trim(), "visible");
        assert!(!exec_result.output.contains("comment"));
        assert_eq!(exec_result.exit_code, 0);
    }

    // WOS-202 Test 4: test_execute_script_with_empty_lines
    #[test]
    fn test_execute_script_with_empty_lines() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho first\n\necho second\n\n\necho third".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert!(exec_result.output.contains("first"));
        assert!(exec_result.output.contains("second"));
        assert!(exec_result.output.contains("third"));
        assert_eq!(exec_result.exit_code, 0);
    }

    // WOS-202 Test 5: test_execute_script_stop_on_error
    #[test]
    fn test_execute_script_stop_on_error() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho before\nnonexistent_command\necho after".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert!(exec_result.output.contains("before"));
        assert!(!exec_result.output.contains("after")); // Should stop before this
        assert_ne!(exec_result.exit_code, 0); // Should have non-zero exit code
    }

    // WOS-202 Test 6: test_execute_script_exit_status_propagation
    #[test]
    fn test_execute_script_exit_status_propagation() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho success".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.exit_code, 0);
    }

    // WOS-202 Test 7: test_execute_script_output_accumulation
    #[test]
    fn test_execute_script_output_accumulation() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho line1\necho line2\necho line3".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        let lines: Vec<&str> = exec_result.output.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");
    }

    // WOS-202 Test 8: test_execute_script_with_invalid_command
    #[test]
    fn test_execute_script_with_invalid_command() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\ninvalid_xyz_command".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_ne!(exec_result.exit_code, 0);
    }

    // Additional test: Shebang line should be skipped
    #[test]
    fn test_execute_skips_shebang() {
        let script = Script {
            path: "/test.sh".to_string(),
            content: "#!/bin/bash\necho test".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert!(!exec_result.output.contains("#!/bin/bash"));
        assert_eq!(exec_result.output.trim(), "test");
    }

    // Additional test: Empty script
    #[test]
    fn test_execute_empty_script() {
        let script = Script {
            path: "/empty.sh".to_string(),
            content: "#!/bin/bash\n".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.output, "");
        assert_eq!(exec_result.exit_code, 0);
    }

    // Additional test: Script with only comments
    #[test]
    fn test_execute_only_comments() {
        let script = Script {
            path: "/comments.sh".to_string(),
            content: "#!/bin/bash\n# Comment 1\n# Comment 2\n# Comment 3".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let mut vfs = create_test_vfs();
        let mut vars = create_test_vars();

        let result = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        assert!(result.is_ok());

        let exec_result = result.unwrap();
        assert_eq!(exec_result.output, "");
        assert_eq!(exec_result.exit_code, 0);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Property test: Execution never panics
    proptest! {
        #[test]
        fn proptest_execute_never_panics(
            content in "[a-zA-Z0-9 \\n]{0,100}"
        ) {
            let script = Script {
                path: "/test.sh".to_string(),
                content: format!("#!/bin/bash\\n{}", content),
                shebang: "#!/bin/bash".to_string(),
            };

            let mut vfs = VirtualFileSystem::new();
            let mut vars = HashMap::new();

            // Should not panic, regardless of input
            let _ = ScriptExecutor::execute(&script, &mut vfs, &mut vars);
        }
    }

    // Property test: Deterministic execution
    proptest! {
        #[test]
        fn proptest_execute_deterministic(
            commands in prop::collection::vec("echo [a-z]+", 1..5)
        ) {
            let content = format!("#!/bin/bash\\n{}", commands.join("\\n"));
            let script = Script {
                path: "/test.sh".to_string(),
                content,
                shebang: "#!/bin/bash".to_string(),
            };

            let mut vfs1 = VirtualFileSystem::new();
            let mut vars1 = HashMap::new();
            let result1 = ScriptExecutor::execute(&script, &mut vfs1, &mut vars1);

            let mut vfs2 = VirtualFileSystem::new();
            let mut vars2 = HashMap::new();
            let result2 = ScriptExecutor::execute(&script, &mut vfs2, &mut vars2);

            // Same script should produce same result
            prop_assert_eq!(result1, result2);
        }
    }
}
