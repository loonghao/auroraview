//! Tests for HooksConfig, VxHooksConfig, and CollectPattern

use auroraview_pack::{CollectPattern, HooksConfig, VxHooksConfig};

// ---------------------------------------------------------------------------
// HooksConfig default
// ---------------------------------------------------------------------------

#[test]
fn hooks_config_default_is_empty() {
    let cfg = HooksConfig::default();
    assert!(cfg.before_collect.is_empty());
    assert!(cfg.collect.is_empty());
    assert!(cfg.after_pack.is_empty());
    assert!(!cfg.use_vx);
}

#[test]
fn hooks_config_debug_clone() {
    let cfg = HooksConfig {
        before_collect: vec!["echo before".to_string()],
        after_pack: vec!["echo after".to_string()],
        ..Default::default()
    };
    let cloned = cfg.clone();
    assert_eq!(cloned.before_collect, cfg.before_collect);
    assert_eq!(cloned.after_pack, cfg.after_pack);
    let debug_str = format!("{:?}", cfg);
    assert!(debug_str.contains("before_collect"));
}

// ---------------------------------------------------------------------------
// HooksConfig with commands
// ---------------------------------------------------------------------------

#[test]
fn hooks_config_before_collect_commands() {
    let cfg = HooksConfig {
        before_collect: vec![
            "npm run build".to_string(),
            "python setup.py build".to_string(),
        ],
        ..Default::default()
    };
    assert_eq!(cfg.before_collect.len(), 2);
    assert_eq!(cfg.before_collect[0], "npm run build");
}

#[test]
fn hooks_config_after_pack_commands() {
    let cfg = HooksConfig {
        after_pack: vec!["./scripts/sign.sh".to_string()],
        ..Default::default()
    };
    assert_eq!(cfg.after_pack.len(), 1);
    assert_eq!(cfg.after_pack[0], "./scripts/sign.sh");
}

#[test]
fn hooks_config_use_vx_flag() {
    let cfg = HooksConfig {
        use_vx: true,
        ..Default::default()
    };
    assert!(cfg.use_vx);
}

#[test]
fn hooks_config_with_collect_patterns() {
    let cfg = HooksConfig {
        collect: vec![
            CollectPattern::new("assets/**/*.png"),
            CollectPattern::new("data/*.json").with_dest("resources/data"),
        ],
        ..Default::default()
    };
    assert_eq!(cfg.collect.len(), 2);
    assert_eq!(cfg.collect[0].source, "assets/**/*.png");
    assert!(cfg.collect[0].dest.is_none());
    assert_eq!(cfg.collect[1].source, "data/*.json");
    assert_eq!(
        cfg.collect[1].dest.as_deref(),
        Some("resources/data")
    );
}

// ---------------------------------------------------------------------------
// HooksConfig serialization / deserialization
// ---------------------------------------------------------------------------

#[test]
fn hooks_config_serde_roundtrip() {
    let cfg = HooksConfig {
        before_collect: vec!["cmd1".to_string()],
        after_pack: vec!["cmd2".to_string()],
        use_vx: true,
        collect: vec![CollectPattern::new("src/**")],
        vx: VxHooksConfig {
            before_collect: vec!["vx-cmd".to_string()],
            after_pack: vec![],
        },
    };
    let json = serde_json::to_string(&cfg).unwrap();
    let parsed: HooksConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.before_collect, cfg.before_collect);
    assert_eq!(parsed.after_pack, cfg.after_pack);
    assert_eq!(parsed.use_vx, cfg.use_vx);
    assert_eq!(parsed.collect.len(), 1);
    assert_eq!(parsed.vx.before_collect.len(), 1);
}

#[test]
fn hooks_config_deserialize_empty_json() {
    let cfg: HooksConfig = serde_json::from_str("{}").unwrap();
    assert!(cfg.before_collect.is_empty());
    assert!(!cfg.use_vx);
}

// ---------------------------------------------------------------------------
// VxHooksConfig
// ---------------------------------------------------------------------------

#[test]
fn vx_hooks_config_default_is_empty() {
    let vx = VxHooksConfig::default();
    assert!(vx.before_collect.is_empty());
    assert!(vx.after_pack.is_empty());
}

#[test]
fn vx_hooks_config_with_commands() {
    let vx = VxHooksConfig {
        before_collect: vec!["vx node npm run build".to_string()],
        after_pack: vec!["vx python scripts/sign.py".to_string()],
    };
    assert_eq!(vx.before_collect.len(), 1);
    assert_eq!(vx.after_pack.len(), 1);
    assert!(vx.before_collect[0].starts_with("vx node"));
}

#[test]
fn vx_hooks_config_clone() {
    let vx = VxHooksConfig {
        before_collect: vec!["a".to_string(), "b".to_string()],
        after_pack: vec![],
    };
    let cloned = vx.clone();
    assert_eq!(cloned.before_collect, vx.before_collect);
}

#[test]
fn vx_hooks_config_serde_roundtrip() {
    let vx = VxHooksConfig {
        before_collect: vec!["cmd1".to_string()],
        after_pack: vec!["cmd2".to_string()],
    };
    let json = serde_json::to_string(&vx).unwrap();
    let parsed: VxHooksConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.before_collect, vx.before_collect);
    assert_eq!(parsed.after_pack, vx.after_pack);
}

// ---------------------------------------------------------------------------
// CollectPattern
// ---------------------------------------------------------------------------

#[test]
fn collect_pattern_new_defaults() {
    let p = CollectPattern::new("src/**/*.rs");
    assert_eq!(p.source, "src/**/*.rs");
    assert!(p.dest.is_none());
    assert!(p.preserve_structure);
    assert!(p.description.is_none());
}

#[test]
fn collect_pattern_with_dest() {
    let p = CollectPattern::new("assets/**").with_dest("bundle/assets");
    assert_eq!(p.dest.as_deref(), Some("bundle/assets"));
    assert_eq!(p.source, "assets/**");
}

#[test]
fn collect_pattern_clone() {
    let p = CollectPattern::new("data/*.json").with_dest("out/data");
    let cloned = p.clone();
    assert_eq!(cloned.source, p.source);
    assert_eq!(cloned.dest, p.dest);
}

#[test]
fn collect_pattern_debug() {
    let p = CollectPattern::new("test/**");
    let debug_str = format!("{:?}", p);
    assert!(debug_str.contains("source"));
}

#[test]
fn collect_pattern_serde_roundtrip() {
    let p = CollectPattern {
        source: "src/**".to_string(),
        dest: Some("out".to_string()),
        preserve_structure: false,
        description: Some("all source files".to_string()),
    };
    let json = serde_json::to_string(&p).unwrap();
    let parsed: CollectPattern = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.source, p.source);
    assert_eq!(parsed.dest, p.dest);
    assert!(!parsed.preserve_structure);
    assert_eq!(parsed.description, p.description);
}

#[test]
fn collect_pattern_serde_minimal() {
    let json = r#"{"source": "assets/**"}"#;
    let p: CollectPattern = serde_json::from_str(json).unwrap();
    assert_eq!(p.source, "assets/**");
    assert!(p.dest.is_none());
    // preserve_structure defaults to true via serde default_true helper
    assert!(p.preserve_structure);
}

// ---------------------------------------------------------------------------
// Integration: HooksConfig with both patterns and vx commands
// ---------------------------------------------------------------------------

#[test]
fn hooks_config_full_integration() {
    let cfg = HooksConfig {
        before_collect: vec!["npm run build".to_string()],
        collect: vec![
            CollectPattern::new("dist/**").with_dest("bundle/dist"),
            CollectPattern {
                source: "assets/**/*.png".to_string(),
                dest: Some("bundle/assets".to_string()),
                preserve_structure: true,
                description: Some("PNG images".to_string()),
            },
        ],
        after_pack: vec!["./post_pack.sh".to_string()],
        use_vx: false,
        vx: VxHooksConfig {
            before_collect: vec![],
            after_pack: vec![],
        },
    };

    assert_eq!(cfg.before_collect.len(), 1);
    assert_eq!(cfg.collect.len(), 2);
    assert_eq!(cfg.collect[0].dest.as_deref(), Some("bundle/dist"));
    assert_eq!(
        cfg.collect[1].description.as_deref(),
        Some("PNG images")
    );
    assert_eq!(cfg.after_pack.len(), 1);
    assert!(!cfg.use_vx);
}
