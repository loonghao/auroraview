//! Tests for auroraview-pack pyoxidizer module

use auroraview_pack::{DistributionFlavor, PyOxidizerBuilder, PyOxidizerBuilderConfig};

#[test]
fn test_distribution_flavor() {
    assert_eq!(DistributionFlavor::Standalone.as_str(), "standalone");
    assert_eq!(
        DistributionFlavor::StandaloneDynamic.as_str(),
        "standalone_dynamic"
    );
    assert_eq!(DistributionFlavor::System.as_str(), "system");
}

#[test]
fn test_default_config() {
    let config = PyOxidizerBuilderConfig::default();
    assert_eq!(config.executable, "pyoxidizer");
    assert_eq!(config.python_version, "3.10");
    assert!(config.release);
    assert_eq!(config.optimize, 1);
}

#[test]
fn test_generate_config() {
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
fn test_get_run_module() {
    let builder = PyOxidizerBuilder::new(PyOxidizerBuilderConfig::default(), "/tmp", "app")
        .entry_point("myapp.main:run_server");

    let config = builder.generate_config().unwrap();
    // The run_module should be extracted from entry_point
    assert!(config.contains("run_module = \"myapp.main\""));
}

#[test]
fn test_config_with_options() {
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
fn test_distribution_flavor_default() {
    let flavor = DistributionFlavor::default();
    assert_eq!(flavor, DistributionFlavor::Standalone);
}
