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
        };

        let yaml = serde_yaml::to_string(&config).expect("Should serialize");
        let deserialized: UxLayoutConfig = serde_yaml::from_str(&yaml).expect("Should deserialize");

        assert_eq!(config, deserialized);
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
            let config = UxLayoutConfig { version, environment };

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
            };
            let config2 = UxLayoutConfig {
                version: "1.0".to_string(),
                environment,
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
                environment
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
                environment: environment.clone()
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
            let config = UxLayoutConfig { version, environment };
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
            let config = UxLayoutConfig { version, environment };

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
                environment: environment.clone()
            };
            let config2 = UxLayoutConfig { version, environment };

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
            let config = UxLayoutConfig { version, environment };

            let _ = format!("{:?}", config);
            // If we get here without panic, test passes
        }
    }
}
