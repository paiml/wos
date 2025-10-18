//! Script loading and shebang validation
//!
//! Provides functionality to load shell scripts from the VFS and validate their shebangs.

use std::path::PathBuf;
use wos_shared::{Script, ScriptError, VirtualFileSystem};

/// Script loader for loading and validating shell scripts from VFS
#[allow(dead_code)]
pub struct ScriptLoader;

#[allow(dead_code)]
impl ScriptLoader {
    /// Load a script from the VFS at the given path
    ///
    /// # Arguments
    /// * `vfs` - Virtual file system to read from
    /// * `path` - Path to the script file (relative or absolute)
    ///
    /// # Returns
    /// * `Ok(Script)` - Loaded and validated script
    /// * `Err(ScriptError)` - If file not found, invalid shebang, or read error
    pub fn load(vfs: &VirtualFileSystem, path: &str) -> Result<Script, ScriptError> {
        // Read file content from VFS
        let content =
            vfs.read_file(&PathBuf::from(path))
                .map_err(|_| ScriptError::FileNotFound {
                    path: path.to_string(),
                })?;

        // Convert Vec<u8> to String
        let content_str = String::from_utf8_lossy(&content).to_string();

        // Validate shebang
        Self::validate_shebang(&content_str)?;

        // Extract shebang (first line)
        let shebang = content_str.lines().next().unwrap_or("").to_string();

        Ok(Script {
            path: path.to_string(),
            content: content_str,
            shebang,
        })
    }

    /// Validate that a script has a valid shebang
    ///
    /// # Arguments
    /// * `content` - Script content to validate
    ///
    /// # Returns
    /// * `Ok(())` - Shebang is valid
    /// * `Err(ScriptError::InvalidShebang)` - If shebang is missing or invalid
    ///
    /// Valid shebangs: `#!/bin/bash`, `#!/bin/sh`
    /// Invalid: `#!/usr/bin/python`, `#!/usr/bin/env python`, etc.
    pub fn validate_shebang(content: &str) -> Result<(), ScriptError> {
        // Get first line
        let first_line = content.lines().next().unwrap_or("");

        // Check if starts with shebang
        if !first_line.starts_with("#!") {
            return Err(ScriptError::InvalidShebang {
                shebang: first_line.to_string(),
                reason: "missing shebang (must start with #!)".to_string(),
            });
        }

        // Check if valid shell shebang
        if first_line == "#!/bin/bash" || first_line == "#!/bin/sh" {
            return Ok(());
        }

        // Invalid shebang
        Err(ScriptError::InvalidShebang {
            shebang: first_line.to_string(),
            reason: "only #!/bin/bash and #!/bin/sh are supported".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create VFS with a test script
    fn create_test_vfs() -> VirtualFileSystem {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/test.sh"),
            "#!/bin/bash\necho hello".as_bytes().to_vec(),
        )
        .unwrap();
        vfs
    }

    // WOS-201 Test 1: test_load_script_with_valid_shebang
    #[test]
    fn test_load_script_with_valid_shebang() {
        let vfs = create_test_vfs();
        let script = ScriptLoader::load(&vfs, "/test.sh").expect("should load script");

        assert_eq!(script.path, "/test.sh");
        assert_eq!(script.content, "#!/bin/bash\necho hello");
        assert_eq!(script.shebang, "#!/bin/bash");
    }

    // WOS-201 Test 2: test_load_script_with_invalid_shebang
    #[test]
    fn test_load_script_with_invalid_shebang() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/invalid.sh"),
            "#!/usr/bin/python\nprint('hello')".as_bytes().to_vec(),
        )
        .unwrap();

        let result = ScriptLoader::load(&vfs, "/invalid.sh");
        assert!(result.is_err());

        if let Err(ScriptError::InvalidShebang { shebang, reason }) = result {
            assert_eq!(shebang, "#!/usr/bin/python");
            assert!(reason.contains("only #!/bin/bash and #!/bin/sh are supported"));
        } else {
            panic!("Expected InvalidShebang error");
        }
    }

    // WOS-201 Test 3: test_load_script_file_not_found
    #[test]
    fn test_load_script_file_not_found() {
        let vfs = VirtualFileSystem::new();
        let result = ScriptLoader::load(&vfs, "/nonexistent.sh");

        assert!(result.is_err());
        if let Err(ScriptError::FileNotFound { path }) = result {
            assert_eq!(path, "/nonexistent.sh");
        } else {
            panic!("Expected FileNotFound error");
        }
    }

    // WOS-201 Test 4: test_load_script_from_absolute_path
    #[test]
    fn test_load_script_from_absolute_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("/home/user/script.sh"),
            "#!/bin/sh\nls".as_bytes().to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "/home/user/script.sh")
            .expect("should load from absolute path");

        assert_eq!(script.path, "/home/user/script.sh");
        assert_eq!(script.shebang, "#!/bin/sh");
    }

    // WOS-201 Test 5: test_load_script_from_relative_path
    #[test]
    fn test_load_script_from_relative_path() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(
            PathBuf::from("test.sh"),
            "#!/bin/bash\npwd".as_bytes().to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "test.sh").expect("should load from relative path");

        assert_eq!(script.path, "test.sh");
        assert_eq!(script.shebang, "#!/bin/bash");
    }

    // WOS-201 Test 6: test_validate_bash_shebang
    #[test]
    fn test_validate_bash_shebang() {
        let content = "#!/bin/bash\necho test";
        let result = ScriptLoader::validate_shebang(content);
        assert!(result.is_ok());
    }

    // WOS-201 Test 7: test_validate_sh_shebang
    #[test]
    fn test_validate_sh_shebang() {
        let content = "#!/bin/sh\nls -la";
        let result = ScriptLoader::validate_shebang(content);
        assert!(result.is_ok());
    }

    // WOS-201 Test 8: test_reject_python_shebang
    #[test]
    fn test_reject_python_shebang() {
        let content = "#!/usr/bin/python\nprint('hello')";
        let result = ScriptLoader::validate_shebang(content);

        assert!(result.is_err());
        if let Err(ScriptError::InvalidShebang { shebang, .. }) = result {
            assert_eq!(shebang, "#!/usr/bin/python");
        } else {
            panic!("Expected InvalidShebang error");
        }
    }

    // WOS-201 Test 9: test_reject_empty_shebang
    #[test]
    fn test_reject_empty_shebang() {
        let content = "echo hello";
        let result = ScriptLoader::validate_shebang(content);

        assert!(result.is_err());
        if let Err(ScriptError::InvalidShebang { shebang, reason }) = result {
            assert_eq!(shebang, "echo hello");
            assert!(reason.contains("missing shebang"));
        } else {
            panic!("Expected InvalidShebang error");
        }
    }

    // WOS-201 Test 10: test_script_content_roundtrip
    #[test]
    fn test_script_content_roundtrip() {
        let mut vfs = VirtualFileSystem::new();
        let original_content = "#!/bin/bash\necho 'hello world'\nexit 0";

        vfs.create_file(
            PathBuf::from("/roundtrip.sh"),
            original_content.as_bytes().to_vec(),
        )
        .unwrap();

        let script = ScriptLoader::load(&vfs, "/roundtrip.sh").unwrap();
        assert_eq!(script.content, original_content);
    }

    // Additional test: Reject env shebangs
    #[test]
    fn test_reject_env_shebang() {
        let content = "#!/usr/bin/env bash\necho test";
        let result = ScriptLoader::validate_shebang(content);

        assert!(result.is_err());
        if let Err(ScriptError::InvalidShebang { shebang, reason }) = result {
            assert_eq!(shebang, "#!/usr/bin/env bash");
            assert!(reason.contains("only #!/bin/bash and #!/bin/sh are supported"));
        } else {
            panic!("Expected InvalidShebang error");
        }
    }

    // Additional test: Empty file
    #[test]
    fn test_load_empty_file() {
        let mut vfs = VirtualFileSystem::new();
        vfs.create_file(PathBuf::from("/empty.sh"), "".as_bytes().to_vec())
            .unwrap();

        let result = ScriptLoader::load(&vfs, "/empty.sh");
        assert!(result.is_err());

        if let Err(ScriptError::InvalidShebang { shebang, reason }) = result {
            assert_eq!(shebang, "");
            assert!(reason.contains("missing shebang"));
        } else {
            panic!("Expected InvalidShebang error");
        }
    }

    // Additional test: Multi-line script
    #[test]
    fn test_load_multiline_script() {
        let mut vfs = VirtualFileSystem::new();
        let content = "#!/bin/bash\necho line1\necho line2\necho line3";

        vfs.create_file(PathBuf::from("/multi.sh"), content.as_bytes().to_vec())
            .unwrap();

        let script = ScriptLoader::load(&vfs, "/multi.sh").unwrap();
        assert_eq!(script.content, content);
        assert_eq!(script.shebang, "#!/bin/bash");
        assert_eq!(script.content.lines().count(), 4);
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Property test: Loading never panics
    proptest! {
        #[test]
        fn proptest_load_never_panics(
            path in "[a-z/]{1,20}",
            content in "[a-zA-Z0-9 \n]{0,100}"
        ) {
            let vfs = VirtualFileSystem::new();
            // Loading may succeed or fail, but should never panic
            let _ = ScriptLoader::load(&vfs, &path);
        }
    }

    // Property test: Valid shebangs always accepted
    proptest! {
        #[test]
        fn proptest_valid_shebangs_accepted(
            shebang in prop_oneof!["#!/bin/bash", "#!/bin/sh"],
            rest in "[a-zA-Z0-9 \n]{0,50}"
        ) {
            let content = format!("{}\n{}", shebang, rest);
            let result = ScriptLoader::validate_shebang(&content);
            prop_assert!(result.is_ok());
        }
    }

    // Property test: Invalid shebangs always rejected
    proptest! {
        #[test]
        fn proptest_invalid_shebangs_rejected(
            interpreter in "(/usr)?/(bin|local/bin)/(python|ruby|node|perl|php)",
            rest in "[a-zA-Z0-9 \n]{0,50}"
        ) {
            let content = format!("#!/{}\\n{}", interpreter, rest);
            let result = ScriptLoader::validate_shebang(&content);
            prop_assert!(result.is_err());
        }
    }

    // Property test: Content without shebang rejected
    proptest! {
        #[test]
        fn proptest_no_shebang_rejected(
            content in "[a-zA-Z0-9 \n]{1,100}"
        ) {
            // Ensure content doesn't start with #!
            prop_assume!(!content.starts_with("#!"));

            let result = ScriptLoader::validate_shebang(&content);
            prop_assert!(result.is_err());
        }
    }

    // Property test: Loaded scripts preserve content
    proptest! {
        #[test]
        fn proptest_content_preservation(
            filename in "[a-z]{1,10}\\.sh",
            body in "[a-zA-Z0-9 \n]{0,100}"
        ) {
            let content = format!("#!/bin/bash\n{}", body);
            let path = format!("/{}", filename);

            let mut vfs = VirtualFileSystem::new();
            if vfs
                .create_file(PathBuf::from(&path), content.as_bytes().to_vec())
                .is_ok()
            {
                if let Ok(script) = ScriptLoader::load(&vfs, &path) {
                    prop_assert_eq!(script.content, content);
                    prop_assert_eq!(script.path, path);
                }
            }
        }
    }
}
