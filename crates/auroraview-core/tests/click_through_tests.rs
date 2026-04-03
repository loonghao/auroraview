//! Tests for click-through functionality

use auroraview_core::builder::{ClickThroughConfig, InteractiveRegion};
use rstest::*;

#[fixture]
fn sample_regions() -> Vec<InteractiveRegion> {
    vec![
        InteractiveRegion::new(10, 20, 100, 50),
        InteractiveRegion::new(200, 100, 150, 80),
    ]
}

mod interactive_region_tests {
    use super::*;

    #[rstest]
    fn test_region_creation() {
        let region = InteractiveRegion::new(10, 20, 100, 50);
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 100);
        assert_eq!(region.height, 50);
    }

    #[rstest]
    #[case(10, 20, true)]
    #[case(50, 40, true)]
    #[case(109, 69, true)]
    #[case(9, 20, false)]
    #[case(10, 19, false)]
    #[case(110, 20, false)]
    #[case(10, 70, false)]
    fn test_region_contains(#[case] x: i32, #[case] y: i32, #[case] expected: bool) {
        let region = InteractiveRegion::new(10, 20, 100, 50);
        assert_eq!(region.contains(x, y), expected);
    }

    #[rstest]
    fn test_region_at_origin() {
        let region = InteractiveRegion::new(0, 0, 50, 50);
        assert!(region.contains(0, 0));
        assert!(region.contains(25, 25));
        assert!(region.contains(49, 49));
        assert!(!region.contains(50, 50));
        assert!(!region.contains(-1, 0));
    }

    #[rstest]
    fn test_region_negative_coords() {
        let region = InteractiveRegion::new(-50, -50, 100, 100);
        assert!(region.contains(-50, -50));
        assert!(region.contains(0, 0));
        assert!(region.contains(49, 49));
        assert!(!region.contains(50, 50));
    }

    #[rstest]
    fn test_region_equality() {
        let r1 = InteractiveRegion::new(1, 2, 3, 4);
        let r2 = InteractiveRegion::new(1, 2, 3, 4);
        assert_eq!(r1, r2);
    }

    #[rstest]
    fn test_region_inequality() {
        let r1 = InteractiveRegion::new(1, 2, 3, 4);
        let r2 = InteractiveRegion::new(5, 6, 7, 8);
        assert_ne!(r1, r2);
    }

    #[rstest]
    fn test_region_clone() {
        let region = InteractiveRegion::new(10, 20, 100, 50);
        let copied = region;
        assert_eq!(region, copied);
    }


    #[rstest]
    fn test_region_zero_size() {
        let region = InteractiveRegion::new(5, 5, 0, 0);
        // Zero-size region should not contain any point
        assert!(!region.contains(5, 5));
        assert!(!region.contains(4, 4));
    }

    #[rstest]
    fn test_region_one_by_one() {
        let region = InteractiveRegion::new(10, 10, 1, 1);
        assert!(region.contains(10, 10));
        assert!(!region.contains(11, 10));
        assert!(!region.contains(10, 11));
    }

    #[rstest]
    fn test_region_large_coords() {
        let region = InteractiveRegion::new(9000, 9000, 500, 500);
        assert!(region.contains(9000, 9000));
        assert!(region.contains(9250, 9250));
        assert!(region.contains(9499, 9499));
        assert!(!region.contains(9500, 9500));
    }

    #[rstest]
    #[case(0, 0, 10, 10)]
    #[case(100, 200, 50, 75)]
    #[case(-10, -20, 30, 40)]
    fn test_region_fields_via_rstest(#[case] x: i32, #[case] y: i32, #[case] w: i32, #[case] h: i32) {
        let r = InteractiveRegion::new(x, y, w, h);
        assert_eq!(r.x, x);
        assert_eq!(r.y, y);
        assert_eq!(r.width, w);
        assert_eq!(r.height, h);
    }

    #[rstest]
    fn test_region_boundary_edges() {
        let region = InteractiveRegion::new(0, 0, 100, 100);
        // Left edge (x=0)
        assert!(region.contains(0, 50));
        // Top edge (y=0)
        assert!(region.contains(50, 0));
        // Right boundary (x=99, width=100 → right edge exclusive at 100)
        assert!(region.contains(99, 50));
        // Bottom boundary (y=99)
        assert!(region.contains(50, 99));
    }
}

mod click_through_config_tests {
    use super::*;

    #[rstest]
    fn test_config_default() {
        let config = ClickThroughConfig::default();
        assert!(!config.enabled);
        assert!(config.regions.is_empty());
    }

    #[rstest]
    fn test_config_builder(sample_regions: Vec<InteractiveRegion>) {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(sample_regions.clone());

        assert!(config.enabled);
        assert_eq!(config.regions.len(), 2);
    }

    #[rstest]
    fn test_is_interactive_when_disabled() {
        let config = ClickThroughConfig::new().with_enabled(false);

        assert!(config.is_interactive(0, 0));
        assert!(config.is_interactive(1000, 1000));
        assert!(config.is_interactive(-100, -100));
    }

    #[rstest]
    fn test_is_interactive_with_regions(sample_regions: Vec<InteractiveRegion>) {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(sample_regions);

        assert!(config.is_interactive(50, 40));
        assert!(config.is_interactive(250, 140));
        assert!(!config.is_interactive(0, 0));
        assert!(!config.is_interactive(150, 50));
        assert!(!config.is_interactive(500, 500));
    }

    #[rstest]
    fn test_is_interactive_empty_regions() {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(vec![]);

        assert!(!config.is_interactive(0, 0));
        assert!(!config.is_interactive(100, 100));
    }

    #[rstest]
    fn test_overlapping_regions() {
        let regions = vec![
            InteractiveRegion::new(0, 0, 100, 100),
            InteractiveRegion::new(50, 50, 100, 100),
        ];
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(regions);

        assert!(config.is_interactive(75, 75));
        assert!(config.is_interactive(25, 25));
        assert!(config.is_interactive(125, 125));
        assert!(!config.is_interactive(200, 200));
    }

    #[rstest]
    fn test_config_enabled_false_all_interactive() {
        // When disabled, all coords are interactive (click-through is not applied)
        let config = ClickThroughConfig::new().with_enabled(false);
        for (x, y) in &[(0, 0), (999, 999), (-1, -1), (5000, 5000)] {
            assert!(config.is_interactive(*x, *y), "disabled config should always return true");
        }
    }

    #[rstest]
    fn test_config_add_region_increments_count() {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(vec![InteractiveRegion::new(0, 0, 50, 50)]);
        assert_eq!(config.regions.len(), 1);

        let config2 = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(vec![
                InteractiveRegion::new(0, 0, 50, 50),
                InteractiveRegion::new(100, 100, 50, 50),
            ]);
        assert_eq!(config2.regions.len(), 2);
    }

    #[rstest]
    fn test_config_with_many_regions() {
        let regions: Vec<_> = (0..100).map(|i| InteractiveRegion::new(i * 10, 0, 10, 10)).collect();
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(regions);
        assert_eq!(config.regions.len(), 100);
        // Point inside region 5 (x=50..60, y=0..10)
        assert!(config.is_interactive(55, 5));
        // Point outside all regions
        assert!(!config.is_interactive(0, 20));
    }

    #[rstest]
    fn test_config_with_enabled_false_default() {
        let config = ClickThroughConfig::default();
        // Default is not enabled
        assert!(!config.enabled);
        // Even without regions, disabled means everything interactive
        assert!(config.is_interactive(0, 0));
    }
}

mod region_serialization_tests {
    use super::*;

    #[rstest]
    fn test_region_serialize() {
        let region = InteractiveRegion::new(10, 20, 100, 50);
        let json = serde_json::to_string(&region).unwrap();
        assert!(json.contains("\"x\":10"));
        assert!(json.contains("\"y\":20"));
        assert!(json.contains("\"width\":100"));
        assert!(json.contains("\"height\":50"));
    }

    #[rstest]
    fn test_region_deserialize() {
        let json = r#"{"x":10,"y":20,"width":100,"height":50}"#;
        let region: InteractiveRegion = serde_json::from_str(json).unwrap();
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 100);
        assert_eq!(region.height, 50);
    }

    #[rstest]
    fn test_region_roundtrip() {
        let original = InteractiveRegion::new(42, 84, 200, 150);
        let json = serde_json::to_string(&original).unwrap();
        let deserialized: InteractiveRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(original, deserialized);
    }

    #[rstest]
    fn test_region_negative_roundtrip() {
        let original = InteractiveRegion::new(-100, -200, 300, 400);
        let json = serde_json::to_string(&original).unwrap();
        let restored: InteractiveRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[rstest]
    fn test_region_zero_roundtrip() {
        let original = InteractiveRegion::new(0, 0, 0, 0);
        let json = serde_json::to_string(&original).unwrap();
        let restored: InteractiveRegion = serde_json::from_str(&json).unwrap();
        assert_eq!(original, restored);
    }

    #[rstest]
    fn test_region_json_is_object() {
        let region = InteractiveRegion::new(1, 2, 3, 4);
        let json = serde_json::to_string(&region).unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(value.is_object());
    }

    #[rstest]
    fn test_region_array_roundtrip() {
        let regions = vec![
            InteractiveRegion::new(10, 20, 100, 50),
            InteractiveRegion::new(200, 100, 150, 80),
        ];
        let json = serde_json::to_string(&regions).unwrap();
        let restored: Vec<InteractiveRegion> = serde_json::from_str(&json).unwrap();
        assert_eq!(regions.len(), restored.len());
        assert_eq!(regions[0], restored[0]);
        assert_eq!(regions[1], restored[1]);
    }

    #[rstest]
    fn test_region_debug_format() {
        let region = InteractiveRegion::new(5, 10, 20, 30);
        let debug_str = format!("{:?}", region);
        assert!(!debug_str.is_empty());
    }
}
