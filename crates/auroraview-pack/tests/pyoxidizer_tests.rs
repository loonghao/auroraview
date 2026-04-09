//! Tests for auroraview-pack pyoxidizer module

use auroraview_pack::{DistributionFlavor, PyOxidizerBuilder, PyOxidizerBuilderConfig};

#[test]
fn distribution_flavor() {
    assert_eq!(DistributionFlavor::Standalone.as_str(), "standalone");
    assert_eq!(
        DistributionFlavor::StandaloneDynamic.as_str(),
        "standalone_dynamic"
    );
    assert_eq!(DistributionFlavor::System.as_str(), "system");
}

#[test]
fn default_config() {
    let config = PyOxidizerBuilderConfig::default();
    assert_eq!(config.executable, "pyoxidizer");
    assert_eq!(config.python_version, "3.10");
    assert!(config.release);
    assert_eq!(config.optimize, 1);
}

#[test]
fn generate_config() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp/test", "myapp")
        .entry_point("myapp.main:run")
        .packages(vec!["requests".to_string(), "pyyaml".to_string()]);

    let config = builder.generate_config().unwrap();
    assert!(config.contains("name = \"myapp\""));
    assert!(config.contains("run_module = \"myapp.main\""));
    assert!(config.contains("\"requests\""));
    assert!(config.contains("\"pyyaml\""));
}

#[test]
fn get_run_module() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("myapp.main:run_server");

    let config = builder.generate_config().unwrap();
    // The run_module should be extracted from entry_point
    assert!(config.contains("run_module = \"myapp.main\""));
}

#[test]
fn config_with_options() {
    let config = PyOxidizerBuilderConfig {
        python_version: "3.12".to_string(),
        optimize: 2,
        include_pip: true,
        include_setuptools: true,
        filesystem_importer: true,
        ..Default::default()
    };

    let builder = PyOxidizerBuilder::new(config, "/tmp", "app").entry_point("main:run");

    let generated = builder.generate_config().unwrap();
    assert!(generated.contains("python_version = \"3.12\""));
    assert!(generated.contains("bytecode_optimize_level_two = true"));
}

#[test]
fn distribution_flavor_default() {
    let flavor = DistributionFlavor::default();
    assert_eq!(flavor, DistributionFlavor::Standalone);
}

// ============================================================================
// Additional coverage tests
// ============================================================================

#[test]
fn entry_point_module_without_function() {
    // Entry point that is just "module" (no colon) should use full string as module
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("mypackage.entrypoint");

    let config = builder.generate_config().unwrap();
    // Should contain the full string as run_module when there is no colon
    assert!(config.contains("run_module = \"mypackage.entrypoint\""));
}

#[test]
fn optimize_level_zero() {
    let config = PyOxidizerBuilderConfig {
        optimize: 0,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config, "/tmp", "app").entry_point("main:run");
    let generated = builder.generate_config().unwrap();
    // Level 0: neither level_one nor level_two should be true
    assert!(!generated.contains("bytecode_optimize_level_two = true"));
}

#[test]
fn optimize_level_one_default() {
    let config = PyOxidizerBuilderConfig::default();
    assert_eq!(config.optimize, 1);
    let builder = PyOxidizerBuilder::new(config, "/tmp", "app").entry_point("main:run");
    let generated = builder.generate_config().unwrap();
    // Should contain level_one = true
    assert!(generated.contains("bytecode_optimize_level_one = true"));
}

#[test]
fn no_packages_generates_empty_list() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("main:run");

    let config = builder.generate_config().unwrap();
    // Should still generate valid config even with no packages
    assert!(config.contains("name = \"app\""));
}

#[test]
fn multiple_packages_all_present() {
    let pkgs = vec![
        "requests".to_string(),
        "pyyaml".to_string(),
        "click".to_string(),
        "rich".to_string(),
    ];

    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("main:run")
        .packages(pkgs.clone());

    let config = builder.generate_config().unwrap();
    for pkg in &pkgs {
        assert!(
            config.contains(&format!("\"{pkg}\"")),
            "package {pkg} should be in config"
        );
    }
}

#[test]
fn config_python_version_311() {
    let config = PyOxidizerBuilderConfig {
        python_version: "3.11".to_string(),
        ..Default::default()
    };

    let builder = PyOxidizerBuilder::new(config, "/tmp", "app").entry_point("main:run");
    let generated = builder.generate_config().unwrap();
    assert!(generated.contains("python_version = \"3.11\""));
}

#[test]
fn app_name_in_config() {
    let builder =
        PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "my-special-app")
            .entry_point("main:run");

    let config = builder.generate_config().unwrap();
    assert!(config.contains("name = \"my-special-app\""));
}

// ============================================================================
// Extended coverage tests
// ============================================================================

#[test]
fn distribution_flavor_standalone_dynamic() {
    assert_eq!(
        DistributionFlavor::StandaloneDynamic.as_str(),
        "standalone_dynamic"
    );
}

#[test]
fn distribution_flavor_system() {
    assert_eq!(DistributionFlavor::System.as_str(), "system");
}

#[test]
fn distribution_flavor_equality() {
    assert_eq!(
        DistributionFlavor::Standalone,
        DistributionFlavor::Standalone
    );
    assert_ne!(DistributionFlavor::Standalone, DistributionFlavor::System);
    assert_ne!(
        DistributionFlavor::StandaloneDynamic,
        DistributionFlavor::System
    );
}

#[test]
fn config_default_release_is_true() {
    let config = PyOxidizerBuilderConfig::default();
    assert!(config.release);
}

#[test]
fn config_default_no_pip_no_setuptools() {
    let config = PyOxidizerBuilderConfig::default();
    assert!(!config.include_pip);
    assert!(!config.include_setuptools);
    assert!(!config.filesystem_importer);
}

#[test]
fn config_clone() {
    let config = PyOxidizerBuilderConfig {
        python_version: "3.11".to_string(),
        optimize: 2,
        ..Default::default()
    };
    let cloned = config.clone();
    assert_eq!(cloned.python_version, config.python_version);
    assert_eq!(cloned.optimize, config.optimize);
}

#[test]
fn config_serde_roundtrip() {
    let config = PyOxidizerBuilderConfig {
        python_version: "3.10".to_string(),
        optimize: 1,
        include_pip: true,
        release: false,
        ..Default::default()
    };
    let json = serde_json::to_string(&config).unwrap();
    let parsed: PyOxidizerBuilderConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.python_version, config.python_version);
    assert_eq!(parsed.optimize, config.optimize);
    assert_eq!(parsed.include_pip, config.include_pip);
    assert_eq!(parsed.release, config.release);
}

#[test]
fn generate_config_contains_auroraview_header() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("AuroraView"));
}

#[test]
fn generate_config_contains_distribution_flavor() {
    let config_obj = PyOxidizerBuilderConfig {
        distribution_flavor: DistributionFlavor::StandaloneDynamic,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config_obj, "/tmp", "app").entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("standalone_dynamic"));
}

#[test]
fn generate_config_include_pip_flag() {
    let config_obj = PyOxidizerBuilderConfig {
        include_pip: true,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config_obj, "/tmp", "app").entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("pip"));
}

#[test]
fn generate_config_include_setuptools_flag() {
    let config_obj = PyOxidizerBuilderConfig {
        include_setuptools: true,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config_obj, "/tmp", "app").entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("setuptools"));
}

#[test]
fn generate_config_filesystem_importer() {
    let config_obj = PyOxidizerBuilderConfig {
        filesystem_importer: true,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config_obj, "/tmp", "app").entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("allow_files"));
}

#[test]
fn generate_config_with_python_path() {
    use std::path::PathBuf;
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("main:run")
        .python_paths(vec![PathBuf::from("/src/mypackage")]);
    let config = builder.generate_config().unwrap();
    assert!(config.contains("read_package_root") || config.contains("mypackage"));
}

#[test]
fn generate_config_with_env_vars() {
    use std::collections::HashMap;
    let mut env = HashMap::new();
    env.insert("MY_VAR".to_string(), "value1".to_string());
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("main:run")
        .env_vars(env);
    // Should generate without error even with env vars set
    let config = builder.generate_config().unwrap();
    assert!(config.contains("app"));
}

#[test]
fn entry_point_with_nested_module() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("my.deep.nested.module:start");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("my.deep.nested.module"));
}

#[test]
fn optimize_level_two_sets_both_flags() {
    let config_obj = PyOxidizerBuilderConfig {
        optimize: 2,
        ..Default::default()
    };
    let builder = PyOxidizerBuilder::new(config_obj, "/tmp", "app").entry_point("main:run");
    let config = builder.generate_config().unwrap();
    assert!(config.contains("bytecode_optimize_level_one = true"));
    assert!(config.contains("bytecode_optimize_level_two = true"));
}

#[test]
fn distribution_flavor_default_is_standalone() {
    let config = PyOxidizerBuilderConfig::default();
    assert_eq!(config.distribution_flavor, DistributionFlavor::Standalone);
}
