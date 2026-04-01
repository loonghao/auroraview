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
