// UX Layout Configuration Module
//
// This module provides YAML-based configuration for the WOS browser UI.
// Following Extreme TDD: Start with minimal types and comprehensive tests.

use serde::{Deserialize, Serialize};

/// Root configuration structure for UX layout
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UxLayoutConfig {
    /// Configuration schema version (e.g., "1.0")
    pub version: String,
    /// Deployment environment for this configuration
    pub environment: Environment,
    /// UI configuration (optional - allows minimal configs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiConfig>,
}

/// UI configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiConfig {
    /// UI mode (controls progressive disclosure)
    pub mode: UiMode,
    /// Color theme
    pub theme: Theme,
    /// Panel visibility and layout configuration
    #[serde(default)]
    pub panels: PanelsConfig,
    /// Terminal-specific configuration
    #[serde(default)]
    pub terminal: TerminalConfig,
    /// Progressive disclosure settings
    #[serde(default)]
    pub progressive_disclosure: ProgressiveDisclosureConfig,
    /// Accessibility settings
    #[serde(default)]
    pub accessibility: AccessibilityConfig,
}

/// UI mode controlling progressive disclosure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UiMode {
    /// Minimal UI - essential elements only
    Minimal,
    /// Standard UI - balanced interface
    Standard,
    /// Debug UI - all panels and debugging info
    Debug,
}

/// Color theme
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// Auto-detect based on system preferences
    Auto,
}

/// Panel visibility and layout configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct PanelsConfig {
    /// Process list panel
    #[serde(default)]
    pub process_list: PanelConfig,
    /// Memory map panel
    #[serde(default)]
    pub memory_map: PanelConfig,
    /// System call trace panel
    #[serde(default)]
    pub syscall_trace: PanelConfig,
    /// File system browser panel
    #[serde(default)]
    pub filesystem: PanelConfig,
    /// System monitor panel (CPU/memory/performance)
    #[serde(default)]
    pub system_monitor: PanelConfig,
}

/// Individual panel configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PanelConfig {
    /// Whether panel is visible
    #[serde(default = "default_true")]
    pub visible: bool,
    /// Whether panel starts collapsed
    #[serde(default)]
    pub collapsed: bool,
    /// Panel position priority (lower = higher priority)
    #[serde(default)]
    pub position: u32,
}

/// Terminal configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TerminalConfig {
    /// Number of lines in history buffer
    #[serde(default = "default_history_size")]
    pub history_size: usize,
    /// Font size in pixels
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Whether to show line numbers
    #[serde(default)]
    pub show_line_numbers: bool,
}

/// Progressive disclosure configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressiveDisclosureConfig {
    /// Auto-collapse panels after this many seconds of inactivity
    #[serde(default)]
    pub auto_collapse_timeout_sec: Option<u32>,
    /// Show tooltips on hover
    #[serde(default = "default_true")]
    pub show_tooltips: bool,
}

/// Accessibility configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccessibilityConfig {
    /// Enable screen reader support
    #[serde(default)]
    pub screen_reader: bool,
    /// Enable high contrast mode
    #[serde(default)]
    pub high_contrast: bool,
    /// Keyboard navigation enabled
    #[serde(default = "default_true")]
    pub keyboard_navigation: bool,
}

// Serde default functions
fn default_true() -> bool {
    true
}

fn default_history_size() -> usize {
    1000
}

fn default_font_size() -> u32 {
    14
}

// Implement Default for structs with serde(default)
impl Default for PanelConfig {
    fn default() -> Self {
        Self {
            visible: true,
            collapsed: false,
            position: 0,
        }
    }
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            history_size: default_history_size(),
            font_size: default_font_size(),
            show_line_numbers: false,
        }
    }
}

impl Default for ProgressiveDisclosureConfig {
    fn default() -> Self {
        Self {
            auto_collapse_timeout_sec: None,
            show_tooltips: true,
        }
    }
}

impl Default for AccessibilityConfig {
    fn default() -> Self {
        Self {
            screen_reader: false,
            high_contrast: false,
            keyboard_navigation: true,
        }
    }
}

/// Deployment environment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    /// Development environment with debugging features
    Development,
    /// Staging environment for pre-production testing
    Staging,
    /// Production environment with optimizations
    Production,
}

#[cfg(test)]
mod tests {
    use super::*;

    // RED Phase: These tests should FAIL until we implement full deserialization

    #[test]
    fn test_deserialize_minimal_config() {
        let yaml = r#"
version: "1.0"
environment: development
"#;
        let config: Result<UxLayoutConfig, _> = serde_yaml::from_str(yaml);
        assert!(config.is_ok(), "Should deserialize minimal config");

        let config = config.unwrap();
        assert_eq!(config.version, "1.0");
        assert_eq!(config.environment, Environment::Development);
    }

    #[test]
    fn test_deserialize_staging_environment() {
        let yaml = r#"
version: "1.0"
environment: staging
"#;
        let config: UxLayoutConfig =
            serde_yaml::from_str(yaml).expect("Should deserialize staging config");
        assert_eq!(config.environment, Environment::Staging);
    }

    #[test]
    fn test_deserialize_production_environment() {
        let yaml = r#"
version: "1.0"
environment: production
"#;
        let config: UxLayoutConfig =
            serde_yaml::from_str(yaml).expect("Should deserialize production config");
        assert_eq!(config.environment, Environment::Production);
    }

    #[test]
    fn test_invalid_environment_fails() {
        let yaml = r#"
version: "1.0"
environment: invalid
"#;
        let config: Result<UxLayoutConfig, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err(), "Should fail on invalid environment");
    }

    #[test]
    fn test_serialize_roundtrip() {
        let config = UxLayoutConfig {
            version: "1.0".to_string(),
            environment: Environment::Development,
            ui: None,
        };

        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(config, deserialized);
    }

    // NEW TESTS FOR FULL STRUCTURE

    #[test]
    fn test_deserialize_full_config_development() {
        let yaml = r#"
version: "1.0"
environment: development
ui:
  mode: debug
  theme: dark
  panels:
    process_list:
      visible: true
      collapsed: false
      position: 0
    memory_map:
      visible: true
      collapsed: false
      position: 1
    syscall_trace:
      visible: true
      collapsed: false
      position: 2
    filesystem:
      visible: true
      collapsed: false
      position: 3
    system_monitor:
      visible: true
      collapsed: false
      position: 4
  terminal:
    history_size: 5000
    font_size: 16
    show_line_numbers: true
  progressive_disclosure:
    auto_collapse_timeout_sec: 30
    show_tooltips: true
  accessibility:
    screen_reader: true
    high_contrast: false
    keyboard_navigation: true
"#;
        let config: UxLayoutConfig =
            serde_yaml::from_str(yaml).expect("Should deserialize full config");
        assert_eq!(config.version, "1.0");
        assert_eq!(config.environment, Environment::Development);

        let ui = config.ui.expect("UI config should exist");
        assert_eq!(ui.mode, UiMode::Debug);
        assert_eq!(ui.theme, Theme::Dark);
        assert_eq!(ui.terminal.history_size, 5000);
        assert_eq!(ui.terminal.font_size, 16);
        assert!(ui.terminal.show_line_numbers);
        assert_eq!(
            ui.progressive_disclosure.auto_collapse_timeout_sec,
            Some(30)
        );
        assert!(ui.accessibility.screen_reader);
    }

    #[test]
    fn test_deserialize_minimal_ui_config() {
        let yaml = r#"
version: "1.0"
environment: production
ui:
  mode: minimal
  theme: light
"#;
        let config: UxLayoutConfig =
            serde_yaml::from_str(yaml).expect("Should deserialize minimal UI config");

        let ui = config.ui.expect("UI config should exist");
        assert_eq!(ui.mode, UiMode::Minimal);
        assert_eq!(ui.theme, Theme::Light);
        // Check defaults are applied
        assert_eq!(ui.terminal.history_size, 1000);
        assert_eq!(ui.terminal.font_size, 14);
        assert!(!ui.terminal.show_line_numbers);
    }

    #[test]
    fn test_ui_mode_deserialization() {
        let yaml_minimal = r#"
version: "1.0"
environment: development
ui:
  mode: minimal
  theme: dark
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_minimal).unwrap();
        assert_eq!(config.ui.unwrap().mode, UiMode::Minimal);

        let yaml_standard = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: dark
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_standard).unwrap();
        assert_eq!(config.ui.unwrap().mode, UiMode::Standard);

        let yaml_debug = r#"
version: "1.0"
environment: development
ui:
  mode: debug
  theme: dark
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_debug).unwrap();
        assert_eq!(config.ui.unwrap().mode, UiMode::Debug);
    }

    #[test]
    fn test_theme_deserialization() {
        let yaml_dark = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: dark
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_dark).unwrap();
        assert_eq!(config.ui.unwrap().theme, Theme::Dark);

        let yaml_light = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: light
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_light).unwrap();
        assert_eq!(config.ui.unwrap().theme, Theme::Light);

        let yaml_auto = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: auto
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml_auto).unwrap();
        assert_eq!(config.ui.unwrap().theme, Theme::Auto);
    }

    #[test]
    fn test_panel_config_defaults() {
        let yaml = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: dark
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml).unwrap();
        let ui = config.ui.unwrap();

        // All panels should have default values
        assert!(ui.panels.process_list.visible);
        assert!(!ui.panels.process_list.collapsed);
        assert_eq!(ui.panels.process_list.position, 0);
    }

    #[test]
    fn test_panel_visibility_override() {
        let yaml = r#"
version: "1.0"
environment: production
ui:
  mode: minimal
  theme: dark
  panels:
    process_list:
      visible: false
    memory_map:
      visible: false
    syscall_trace:
      visible: false
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml).unwrap();
        let ui = config.ui.unwrap();

        assert!(!ui.panels.process_list.visible);
        assert!(!ui.panels.memory_map.visible);
        assert!(!ui.panels.syscall_trace.visible);
        // filesystem and system_monitor should still have defaults
        assert!(ui.panels.filesystem.visible);
        assert!(ui.panels.system_monitor.visible);
    }

    #[test]
    fn test_terminal_config_custom_values() {
        let yaml = r#"
version: "1.0"
environment: development
ui:
  mode: debug
  theme: dark
  terminal:
    history_size: 10000
    font_size: 18
    show_line_numbers: true
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml).unwrap();
        let ui = config.ui.unwrap();

        assert_eq!(ui.terminal.history_size, 10000);
        assert_eq!(ui.terminal.font_size, 18);
        assert!(ui.terminal.show_line_numbers);
    }

    #[test]
    fn test_progressive_disclosure_timeout() {
        let yaml = r#"
version: "1.0"
environment: staging
ui:
  mode: standard
  theme: auto
  progressive_disclosure:
    auto_collapse_timeout_sec: 60
    show_tooltips: false
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml).unwrap();
        let ui = config.ui.unwrap();

        assert_eq!(
            ui.progressive_disclosure.auto_collapse_timeout_sec,
            Some(60)
        );
        assert!(!ui.progressive_disclosure.show_tooltips);
    }

    #[test]
    fn test_accessibility_settings() {
        let yaml = r#"
version: "1.0"
environment: production
ui:
  mode: standard
  theme: light
  accessibility:
    screen_reader: true
    high_contrast: true
    keyboard_navigation: true
"#;
        let config: UxLayoutConfig = serde_yaml::from_str(yaml).unwrap();
        let ui = config.ui.unwrap();

        assert!(ui.accessibility.screen_reader);
        assert!(ui.accessibility.high_contrast);
        assert!(ui.accessibility.keyboard_navigation);
    }

    #[test]
    fn test_invalid_ui_mode_fails() {
        let yaml = r#"
version: "1.0"
environment: development
ui:
  mode: invalid_mode
  theme: dark
"#;
        let config: Result<UxLayoutConfig, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err(), "Should fail on invalid UI mode");
    }

    #[test]
    fn test_invalid_theme_fails() {
        let yaml = r#"
version: "1.0"
environment: development
ui:
  mode: standard
  theme: invalid_theme
"#;
        let config: Result<UxLayoutConfig, _> = serde_yaml::from_str(yaml);
        assert!(config.is_err(), "Should fail on invalid theme");
    }

    #[test]
    fn test_full_config_roundtrip() {
        let config = UxLayoutConfig {
            version: "1.0".to_string(),
            environment: Environment::Development,
            ui: Some(UiConfig {
                mode: UiMode::Debug,
                theme: Theme::Dark,
                panels: PanelsConfig::default(),
                terminal: TerminalConfig {
                    history_size: 2000,
                    font_size: 16,
                    show_line_numbers: true,
                },
                progressive_disclosure: ProgressiveDisclosureConfig {
                    auto_collapse_timeout_sec: Some(45),
                    show_tooltips: true,
                },
                accessibility: AccessibilityConfig {
                    screen_reader: true,
                    high_contrast: false,
                    keyboard_navigation: true,
                },
            }),
        };

        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_defaults_trait_implementations() {
        let panel = PanelConfig::default();
        assert!(panel.visible);
        assert!(!panel.collapsed);
        assert_eq!(panel.position, 0);

        let terminal = TerminalConfig::default();
        assert_eq!(terminal.history_size, 1000);
        assert_eq!(terminal.font_size, 14);
        assert!(!terminal.show_line_numbers);

        let pd = ProgressiveDisclosureConfig::default();
        assert_eq!(pd.auto_collapse_timeout_sec, None);
        assert!(pd.show_tooltips);

        let a11y = AccessibilityConfig::default();
        assert!(!a11y.screen_reader);
        assert!(!a11y.high_contrast);
        assert!(a11y.keyboard_navigation);
    }
}

// Property-based tests using proptest
#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;

    // Strategy for generating arbitrary Environment values
    fn environment_strategy() -> impl Strategy<Value = Environment> {
        prop_oneof![
            Just(Environment::Development),
            Just(Environment::Staging),
            Just(Environment::Production),
        ]
    }

    // Strategy for generating arbitrary version strings
    fn version_strategy() -> impl Strategy<Value = String> {
        // Version strings should be reasonable (semver-like)
        prop_oneof![
            Just("1.0".to_string()),
            Just("2.0".to_string()),
            Just("1.1.0".to_string()),
            Just("0.1.0".to_string()),
            "[0-9]{1,2}\\.[0-9]{1,2}".prop_map(|s| s),
            "[0-9]{1,2}\\.[0-9]{1,2}\\.[0-9]{1,2}".prop_map(|s| s),
        ]
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10_000))]

        /// Property: Serialization roundtrip preserves equality
        /// Any valid config serialized then deserialized should equal the original
        #[test]
        fn prop_serialize_deserialize_roundtrip(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version,
                environment,
                ui: None,
            };

            let yaml = serde_yaml::to_string(&config)
                .expect("Serialization should never fail for valid config");
            let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml)
                .expect("Deserialization should never fail for serialized config");

            prop_assert_eq!(config, deserialized);
        }

        /// Property: Environment serialization is deterministic
        /// Same environment always serializes to same YAML string
        #[test]
        fn prop_environment_serialization_deterministic(
            environment in environment_strategy()
        ) {
            let config1 = UxLayoutConfig {
                version: "1.0".to_string(),
                environment: environment.clone(),
                ui: None,
            };
            let config2 = UxLayoutConfig {
                version: "1.0".to_string(),
                environment,
                ui: None,
            };

            let yaml1 = serde_yaml::to_string(&config1)
                .expect("Serialization should succeed");
            let yaml2 = serde_yaml::to_string(&config2)
                .expect("Serialization should succeed");

            prop_assert_eq!(yaml1, yaml2);
        }

        /// Property: Version string is preserved exactly
        /// The version string should survive roundtrip without modification
        #[test]
        fn prop_version_preserved(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version: version.clone(),
                environment,
                ui: None,
            };

            let yaml = serde_yaml::to_string(&config)
                .expect("Serialization should succeed");
            let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml)
                .expect("Deserialization should succeed");

            prop_assert_eq!(version, deserialized.version);
        }

        /// Property: Environment is preserved exactly
        /// The environment should survive roundtrip without modification
        #[test]
        fn prop_environment_preserved(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version,
                environment: environment.clone(),
                ui: None,
            };

            let yaml = serde_yaml::to_string(&config)
                .expect("Serialization should succeed");
            let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml)
                .expect("Deserialization should succeed");

            prop_assert_eq!(environment, deserialized.environment);
        }

        /// Property: Clone creates equal instance
        /// Cloning a config should produce an identical config
        #[test]
        fn prop_clone_equality(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version,
                environment,
                ui: None,
            };
            let cloned = config.clone();

            prop_assert_eq!(config, cloned);
        }

        /// Property: PartialEq is reflexive
        /// Any config should equal itself
        #[test]
        fn prop_partialeq_reflexive(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version,
                environment,
                ui: None,
            };

            prop_assert_eq!(&config, &config);
        }

        /// Property: PartialEq is symmetric
        /// If a == b then b == a
        #[test]
        fn prop_partialeq_symmetric(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config1 = UxLayoutConfig {
                version: version.clone(),
                environment: environment.clone(),
                ui: None,
            };
            let config2 = UxLayoutConfig {
                version,
                environment,
                ui: None,
            };

            let eq_forward = config1 == config2;
            let eq_backward = config2 == config1;

            prop_assert_eq!(eq_forward, eq_backward);
        }

        /// Property: Debug formatting never panics
        /// Debug implementation should handle all inputs
        #[test]
        fn prop_debug_no_panic(
            version in version_strategy(),
            environment in environment_strategy()
        ) {
            let config = UxLayoutConfig {
                version,
                environment,
                ui: None,
            };

            let _ = format!("{:?}", config);
            // If we get here without panic, test passes
        }
    }
}
