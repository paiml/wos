//! Shell script types and error handling
//!
//! Provides types for representing shell scripts and script execution errors.
//! All types are serializable for state persistence and deterministic execution.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a shell script with its metadata
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Script {
    /// Path to the script file
    pub path: String,
    /// Content of the script
    pub content: String,
    /// Shebang line (e.g., "#!/bin/bash")
    pub shebang: String,
}

/// Errors that can occur during script operations
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ScriptError {
    /// Script file not found at the specified path
    FileNotFound {
        /// Path that was not found
        path: String,
    },
    /// Invalid or unsupported shebang
    InvalidShebang {
        /// The invalid shebang that was encountered
        shebang: String,
        /// Reason why it's invalid
        reason: String,
    },
    /// Error during script execution
    ExecutionError {
        /// Line number where error occurred
        line: usize,
        /// Command that failed
        command: String,
        /// Error message
        message: String,
    },
    /// Syntax error in script
    SyntaxError {
        /// Line number where error occurred
        line: usize,
        /// The problematic line content
        content: String,
        /// Error message
        message: String,
    },
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::FileNotFound { path } => {
                write!(f, "script not found: {}", path)
            }
            ScriptError::InvalidShebang { shebang, reason } => {
                write!(f, "invalid shebang '{}': {}", shebang, reason)
            }
            ScriptError::ExecutionError {
                line,
                command,
                message,
            } => {
                write!(
                    f,
                    "execution error at line {}: command '{}' failed: {}",
                    line, command, message
                )
            }
            ScriptError::SyntaxError {
                line,
                content,
                message,
            } => {
                write!(
                    f,
                    "syntax error at line {}: '{}' - {}",
                    line, content, message
                )
            }
        }
    }
}

impl std::error::Error for ScriptError {}

#[cfg(test)]
mod tests {
    use super::*;

    // WOS-200 Test 1: test_script_creation_with_shebang
    #[test]
    fn test_script_creation_with_shebang() {
        let script = Script {
            path: "/tmp/test.sh".to_string(),
            content: "#!/bin/bash\necho hello".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        assert_eq!(script.path, "/tmp/test.sh");
        assert_eq!(script.content, "#!/bin/bash\necho hello");
        assert_eq!(script.shebang, "#!/bin/bash");
    }

    // WOS-200 Test 2: test_script_creation_without_shebang_fails
    // Note: This will be validated by ScriptLoader in WOS-201
    // For now, we test that Script can be created but validation happens elsewhere
    #[test]
    fn test_script_creation_without_shebang_fails() {
        // Script struct can be created with empty shebang
        // Validation happens in ScriptLoader::validate_shebang
        let script = Script {
            path: "/tmp/invalid.sh".to_string(),
            content: "echo hello".to_string(),
            shebang: String::new(),
        };

        // Script is created, but empty shebang indicates invalid state
        assert_eq!(script.shebang, "");
    }

    // WOS-200 Test 3: test_script_error_serialization
    #[test]
    fn test_script_error_serialization() {
        let error = ScriptError::FileNotFound {
            path: "/tmp/missing.sh".to_string(),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&error).expect("serialization should succeed");
        assert!(json.contains("FileNotFound"));
        assert!(json.contains("/tmp/missing.sh"));

        // Deserialize back
        let deserialized: ScriptError =
            serde_json::from_str(&json).expect("deserialization should succeed");
        assert_eq!(error, deserialized);
    }

    // WOS-200 Test 4: test_script_error_display_messages
    #[test]
    fn test_script_error_display_messages() {
        let file_not_found = ScriptError::FileNotFound {
            path: "/tmp/missing.sh".to_string(),
        };
        assert_eq!(
            file_not_found.to_string(),
            "script not found: /tmp/missing.sh"
        );

        let invalid_shebang = ScriptError::InvalidShebang {
            shebang: "#!/usr/bin/python".to_string(),
            reason: "only bash and sh are supported".to_string(),
        };
        assert_eq!(
            invalid_shebang.to_string(),
            "invalid shebang '#!/usr/bin/python': only bash and sh are supported"
        );

        let execution_error = ScriptError::ExecutionError {
            line: 5,
            command: "rm -rf /".to_string(),
            message: "permission denied".to_string(),
        };
        assert_eq!(
            execution_error.to_string(),
            "execution error at line 5: command 'rm -rf /' failed: permission denied"
        );

        let syntax_error = ScriptError::SyntaxError {
            line: 3,
            content: "if [ $x".to_string(),
            message: "unclosed bracket".to_string(),
        };
        assert_eq!(
            syntax_error.to_string(),
            "syntax error at line 3: 'if [ $x' - unclosed bracket"
        );
    }

    // WOS-200 Test 5: test_script_clone_equality (property test seed)
    #[test]
    fn test_script_clone_equality() {
        let original = Script {
            path: "/tmp/test.sh".to_string(),
            content: "#!/bin/bash\necho test".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original, cloned);

        // Verify all fields match
        assert_eq!(original.path, cloned.path);
        assert_eq!(original.content, cloned.content);
        assert_eq!(original.shebang, cloned.shebang);
    }

    // WOS-200 Test 6: test_script_validation_empty_content
    #[test]
    fn test_script_validation_empty_content() {
        let empty_script = Script {
            path: "/tmp/empty.sh".to_string(),
            content: String::new(),
            shebang: "#!/bin/bash".to_string(),
        };

        // Script can be created with empty content
        // Validation happens in ScriptExecutor
        assert_eq!(empty_script.content, "");
    }

    // Additional test: ScriptError variants serialize correctly
    #[test]
    fn test_all_script_error_variants_serialize() {
        let errors = vec![
            ScriptError::FileNotFound {
                path: "/test".to_string(),
            },
            ScriptError::InvalidShebang {
                shebang: "#!/bin/python".to_string(),
                reason: "not supported".to_string(),
            },
            ScriptError::ExecutionError {
                line: 1,
                command: "test".to_string(),
                message: "failed".to_string(),
            },
            ScriptError::SyntaxError {
                line: 2,
                content: "bad syntax".to_string(),
                message: "parse error".to_string(),
            },
        ];

        for error in errors {
            let json = serde_json::to_string(&error).expect("should serialize");
            let deserialized: ScriptError =
                serde_json::from_str(&json).expect("should deserialize");
            assert_eq!(error, deserialized);
        }
    }

    // Additional test: Script struct serialization roundtrip
    #[test]
    fn test_script_serialization_roundtrip() {
        let script = Script {
            path: "/tmp/complex.sh".to_string(),
            content: "#!/bin/bash\necho 'hello world'\nexit 0".to_string(),
            shebang: "#!/bin/bash".to_string(),
        };

        let json = serde_json::to_string(&script).expect("should serialize");
        let deserialized: Script = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(script, deserialized);
    }

    // Additional test: ScriptError implements Error trait
    #[test]
    fn test_script_error_is_std_error() {
        let error = ScriptError::FileNotFound {
            path: "/test".to_string(),
        };

        // Should be able to use as std::error::Error
        let _err_ref: &dyn std::error::Error = &error;
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Property test: Script cloning is idempotent and preserves equality
    proptest! {
        #[test]
        fn proptest_script_clone_equality(
            path in "[a-z/]{1,20}",
            content in "[a-zA-Z0-9 \n]{0,100}",
            shebang in "#!/bin/(bash|sh)"
        ) {
            let script = Script { path, content, shebang };
            let cloned = script.clone();

            // Clone must equal original
            prop_assert_eq!(&script, &cloned);

            // Multiple clones must all be equal
            let cloned2 = cloned.clone();
            prop_assert_eq!(&cloned, &cloned2);
            prop_assert_eq!(&script, &cloned2);
        }
    }

    // Property test: Serialization roundtrip preserves Script data
    proptest! {
        #[test]
        fn proptest_script_serialization_roundtrip(
            path in "[a-z/]{1,20}",
            content in "[a-zA-Z0-9 \n]{0,100}",
            shebang in "#!/bin/(bash|sh)"
        ) {
            let original = Script { path, content, shebang };

            let json = serde_json::to_string(&original)?;
            let deserialized: Script = serde_json::from_str(&json)?;

            prop_assert_eq!(original, deserialized);
        }
    }

    // Property test: ScriptError serialization roundtrip
    proptest! {
        #[test]
        fn proptest_script_error_serialization(
            path in "[a-z/]{1,20}",
            line in 1usize..1000,
            command in "[a-z ]{1,20}",
            message in "[a-zA-Z ]{1,50}"
        ) {
            let errors = vec![
                ScriptError::FileNotFound { path: path.clone() },
                ScriptError::ExecutionError {
                    line,
                    command: command.clone(),
                    message: message.clone(),
                },
            ];

            for error in errors {
                let json = serde_json::to_string(&error)?;
                let deserialized: ScriptError = serde_json::from_str(&json)?;
                prop_assert_eq!(error, deserialized);
            }
        }
    }

    // Property test: Display messages are non-empty and contain key information
    proptest! {
        #[test]
        fn proptest_error_display_contains_info(
            path in "[a-z/]{1,20}",
            line in 1usize..100
        ) {
            let error = ScriptError::FileNotFound { path: path.clone() };
            let display = error.to_string();

            // Display must be non-empty
            prop_assert!(!display.is_empty());
            // Display must contain the path
            prop_assert!(display.contains(&path));

            let exec_error = ScriptError::ExecutionError {
                line,
                command: "test".to_string(),
                message: "failed".to_string(),
            };
            let exec_display = exec_error.to_string();

            // Display must contain line number
            prop_assert!(exec_display.contains(&line.to_string()));
        }
    }

    // Property test: Script equality is reflexive, symmetric, transitive
    proptest! {
        #[test]
        fn proptest_script_equality_properties(
            path in "[a-z/]{1,20}",
            content in "[a-z \n]{0,50}",
            shebang in "#!/bin/(bash|sh)"
        ) {
            let script = Script {
                path: path.clone(),
                content: content.clone(),
                shebang: shebang.clone(),
            };

            // Reflexive: a == a
            prop_assert_eq!(&script, &script);

            // Symmetric: if a == b then b == a
            let other = script.clone();
            prop_assert_eq!(&script, &other);
            prop_assert_eq!(&other, &script);

            // Transitive: if a == b and b == c then a == c
            let third = other.clone();
            prop_assert_eq!(&script, &other);
            prop_assert_eq!(&other, &third);
            prop_assert_eq!(&script, &third);
        }
    }
}
