//! Script loader for reading and validating shell scripts from VFS
//!
//! Provides ScriptLoader for loading script files, parsing shebangs,
//! and validating script content before execution.

use crate::script::{Script, ScriptError};
use crate::vfs::VirtualFileSystem;
use std::path::PathBuf;

/// Loads and validates shell scripts from the virtual filesystem
pub struct ScriptLoader;

impl ScriptLoader {
    /// Load a script file from the VFS
    ///
    /// # Arguments
    /// * `vfs` - Virtual filesystem to read from
    /// * `path` - Path to script file (absolute or relative)
    ///
    /// # Returns
    /// * `Ok(Script)` - Successfully loaded and validated script
    /// * `Err(ScriptError)` - File not found or validation failed
    ///
    /// # Examples
    /// ```
    /// use wos_shared::script_loader::ScriptLoader;
    /// use wos_shared::vfs::VirtualFileSystem;
    /// use std::path::PathBuf;
    ///
    /// let mut vfs = VirtualFileSystem::new();
    /// vfs.create_file(PathBuf::from("/test.sh"), b"#!/bin/bash\necho hello".to_vec()).unwrap();
    ///
    /// let script = ScriptLoader::load(&vfs, "/test.sh").unwrap();
    /// assert_eq!(script.path, "/test.sh");
    /// assert_eq!(script.shebang, "#!/bin/bash");
    /// ```
    pub fn load(vfs: &VirtualFileSystem, path: &str) -> Result<Script, ScriptError> {
        // Resolve path (handle relative paths)
        let full_path = Self::resolve_path(path);
        let path_buf = PathBuf::from(&full_path);

        // Read file from VFS
        let content_bytes = vfs
            .read_file(&path_buf)
            .map_err(|_| ScriptError::FileNotFound {
                path: path.to_string(),
            })?;

        // Convert to UTF-8 string
        let content = String::from_utf8_lossy(&content_bytes).to_string();

        // Parse and validate shebang
        let shebang = Self::parse_shebang(&content)?;

        Ok(Script {
            path: full_path,
            content,
            shebang,
        })
    }

    /// Validate that a shebang is supported (bash or sh only)
    ///
    /// # Arguments
    /// * `content` - Script content to validate
    ///
    /// # Returns
    /// * `Ok(())` - Shebang is valid or missing (allowed)
    /// * `Err(ScriptError)` - Invalid shebang (e.g., python, ruby)
    pub fn validate_shebang(content: &str) -> Result<(), ScriptError> {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") && !Self::is_bash_shebang(first_line) {
                return Err(ScriptError::InvalidShebang {
                    shebang: first_line.to_string(),
                    reason: "only bash and sh are supported".to_string(),
                });
            }
        }
        Ok(())
    }

    /// Parse shebang from script content
    ///
    /// Returns the shebang line if present, or empty string if missing
    fn parse_shebang(content: &str) -> Result<String, ScriptError> {
        if let Some(first_line) = content.lines().next() {
            if first_line.starts_with("#!") {
                // Validate shebang
                if !Self::is_bash_shebang(first_line) {
                    return Err(ScriptError::InvalidShebang {
                        shebang: first_line.to_string(),
                        reason: "only bash and sh are supported".to_string(),
                    });
                }
                return Ok(first_line.to_string());
            }
        }
        // No shebang is allowed (defaults to sh)
        Ok(String::new())
    }

    /// Check if shebang indicates bash or sh interpreter
    fn is_bash_shebang(shebang: &str) -> bool {
        shebang.contains("/bash")
            || shebang.contains("/sh")
            || shebang.ends_with(" bash")
            || shebang.ends_with(" sh")
    }

    /// Resolve path to absolute path
    ///
    /// Handles:
    /// - Absolute paths (/foo/bar.sh) - returned as-is
    /// - Relative paths (./foo.sh, foo.sh) - prepended with /
    fn resolve_path(path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            // Remove leading ./ if present
            let clean_path = path.strip_prefix("./").unwrap_or(path);
            format!("/{}", clean_path)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // WOS-201 Test 1: test_load_script_with_valid_shebang
    #[test]
    fn test_load_script_with_valid_shebang() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/test.sh"),
            b"#!/bin/bash\necho hello".to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "/test.sh").unwrap();

        assert_eq!(script.path, "/test.sh");
        assert_eq!(script.shebang, "#!/bin/bash");
        assert!(script.content.contains("echo hello"));
    }

    // WOS-201 Test 2: test_load_script_with_invalid_shebang
    #[test]
    fn test_load_script_with_invalid_shebang() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/python.sh"),
            b"#!/usr/bin/python\nprint('hello')".to_vec(),
        )
        .unwrap();

        let result = ScriptLoader::load(&vfs, "/python.sh");

        assert!(result.is_err());
        match result.unwrap_err() {
            ScriptError::InvalidShebang { shebang, reason } => {
                assert_eq!(shebang, "#!/usr/bin/python");
                assert!(reason.contains("bash"));
            }
            _ => panic!("Expected InvalidShebang error"),
        }
    }

    // WOS-201 Test 3: test_load_script_file_not_found
    #[test]
    fn test_load_script_file_not_found() {
        let vfs = VirtualFileSystem::new();

        let result = ScriptLoader::load(&vfs, "/nonexistent.sh");

        assert!(result.is_err());
        match result.unwrap_err() {
            ScriptError::FileNotFound { path } => {
                assert_eq!(path, "/nonexistent.sh");
            }
            _ => panic!("Expected FileNotFound error"),
        }
    }

    // WOS-201 Test 4: test_load_script_from_absolute_path
    #[test]
    fn test_load_script_from_absolute_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/usr/local/bin/script.sh"),
            b"#!/bin/sh\nls".to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "/usr/local/bin/script.sh").unwrap();

        assert_eq!(script.path, "/usr/local/bin/script.sh");
        assert_eq!(script.shebang, "#!/bin/sh");
    }

    // WOS-201 Test 5: test_load_script_from_relative_path
    #[test]
    fn test_load_script_from_relative_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/hello.sh"),
            b"#!/bin/bash\necho world".to_vec(),
        )
        .unwrap();

        // Load with relative path
        let script = ScriptLoader::load(&vfs, "hello.sh").unwrap();

        // Should resolve to absolute path
        assert_eq!(script.path, "/hello.sh");
        assert_eq!(script.shebang, "#!/bin/bash");
    }

    // WOS-201 Test 6: test_validate_bash_shebang
    #[test]
    fn test_validate_bash_shebang() {
        let content = "#!/bin/bash\necho test";
        assert!(ScriptLoader::validate_shebang(content).is_ok());
    }

    // WOS-201 Test 7: test_validate_sh_shebang
    #[test]
    fn test_validate_sh_shebang() {
        let content = "#!/bin/sh\necho test";
        assert!(ScriptLoader::validate_shebang(content).is_ok());
    }

    // WOS-201 Test 8: test_reject_python_shebang
    #[test]
    fn test_reject_python_shebang() {
        let content = "#!/usr/bin/python\nprint('test')";
        let result = ScriptLoader::validate_shebang(content);

        assert!(result.is_err());
        match result.unwrap_err() {
            ScriptError::InvalidShebang { shebang, .. } => {
                assert!(shebang.contains("python"));
            }
            _ => panic!("Expected InvalidShebang error"),
        }
    }

    // WOS-201 Test 9: test_reject_empty_shebang
    #[test]
    fn test_reject_empty_shebang() {
        let content = "#!\necho test";
        let result = ScriptLoader::validate_shebang(content);

        // Empty shebang (#! with no interpreter) is invalid
        assert!(result.is_err());
    }

    // WOS-201 Test 10: test_script_content_roundtrip
    #[test]
    fn test_script_content_roundtrip() {
        let mut vfs = VirtualFileSystem::new();
        let original_content = "#!/bin/bash\n# Comment\necho \"hello world\"\nexit 0";
        vfs.create_file(
            PathBuf::from("/roundtrip.sh"),
            original_content.as_bytes().to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "/roundtrip.sh").unwrap();

        assert_eq!(script.content, original_content);
        assert_eq!(script.shebang, "#!/bin/bash");
    }

    // Additional test: Load script with ./relative path
    #[test]
    fn test_load_script_with_dot_slash() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/script.sh"), b"#!/bin/bash\nls".to_vec())
            .unwrap();

        let script = ScriptLoader::load(&vfs, "./script.sh").unwrap();

        assert_eq!(script.path, "/script.sh");
    }

    // Additional test: Script without shebang (allowed)
    #[test]
    fn test_load_script_without_shebang() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/no-shebang.sh"), b"echo hello".to_vec())
            .unwrap();

        let script = ScriptLoader::load(&vfs, "/no-shebang.sh").unwrap();

        assert_eq!(script.shebang, "");
        assert!(script.content.contains("echo hello"));
    }

    // Additional test: is_bash_shebang helper
    #[test]
    fn test_is_bash_shebang_helper() {
        assert!(ScriptLoader::is_bash_shebang("#!/bin/bash"));
        assert!(ScriptLoader::is_bash_shebang("#!/usr/bin/env bash"));
        assert!(ScriptLoader::is_bash_shebang("#!/bin/sh"));
        assert!(ScriptLoader::is_bash_shebang("#!/usr/bin/sh"));
        assert!(!ScriptLoader::is_bash_shebang("#!/usr/bin/python"));
        assert!(!ScriptLoader::is_bash_shebang("#!/usr/bin/ruby"));
        assert!(!ScriptLoader::is_bash_shebang("#!"));
    }

    // Additional test: resolve_path helper
    #[test]
    fn test_resolve_path() {
        assert_eq!(ScriptLoader::resolve_path("/abs/path.sh"), "/abs/path.sh");
        assert_eq!(ScriptLoader::resolve_path("relative.sh"), "/relative.sh");
        assert_eq!(ScriptLoader::resolve_path("./relative.sh"), "/relative.sh");
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // WOS-201 Property Test: load never panics
    proptest! {
        #[test]
        fn proptest_load_never_panics(
            path in "[a-z./]{1,20}",
            content in "[a-zA-Z0-9 \n#!]{0,100}"
        ) {
            let mut vfs = VirtualFileSystem::new();
            let file_path = format!("/{}", path.trim_start_matches("./"));
            vfs.create_file(PathBuf::from(&file_path), content.as_bytes().to_vec()).ok();

            // Should never panic, regardless of input
            let _ = ScriptLoader::load(&vfs, &path);
        }
    }

    // Property test: Shebang validation is deterministic
    proptest! {
        #[test]
        fn proptest_shebang_validation_deterministic(
            content in "[a-zA-Z0-9 \n#/!]{0,100}"
        ) {
            let result1 = ScriptLoader::validate_shebang(&content);
            let result2 = ScriptLoader::validate_shebang(&content);

            // Same input should produce same result
            match (result1, result2) {
                (Ok(()), Ok(())) => {},
                (Err(e1), Err(e2)) => {
                    prop_assert_eq!(e1.to_string(), e2.to_string());
                }
                _ => prop_assert!(false, "Validation not deterministic"),
            }
        }
    }

    // Property test: Absolute paths remain unchanged
    proptest! {
        #[test]
        fn proptest_absolute_path_unchanged(
            path in "/[a-z/]{1,30}"
        ) {
            let resolved = ScriptLoader::resolve_path(&path);
            prop_assert_eq!(resolved, path);
        }
    }

    // Property test: Relative paths get / prefix
    proptest! {
        #[test]
        fn proptest_relative_path_gets_prefix(
            path in "[a-z]{1,20}"
        ) {
            let resolved = ScriptLoader::resolve_path(&path);
            prop_assert!(resolved.starts_with('/'));
            prop_assert!(resolved.ends_with(&path));
        }
    }

    // Property test: Valid bash shebangs always pass
    proptest! {
        #[test]
        fn proptest_bash_shebangs_valid(
            interpreter in "(bash|sh)",
            prefix in "(/bin/|/usr/bin/|/usr/bin/env )"
        ) {
            let shebang = format!("#!/{}{}", prefix, interpreter);
            prop_assert!(ScriptLoader::is_bash_shebang(&shebang));
        }
    }
}
