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
    #[case(10, 20, true)] // Top-left corner
    #[case(50, 40, true)] // Center
    #[case(109, 69, true)] // Inside (width=100, height=50)
    #[case(9, 20, false)] // Left of region
    #[case(10, 19, false)] // Above region
    #[case(110, 20, false)] // Right of region (x + width = 110)
    #[case(10, 70, false)] // Below region (y + height = 70)
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

        // When disabled, everything should be interactive
        assert!(config.is_interactive(0, 0));
        assert!(config.is_interactive(1000, 1000));
        assert!(config.is_interactive(-100, -100));
    }

    #[rstest]
    fn test_is_interactive_with_regions(sample_regions: Vec<InteractiveRegion>) {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(sample_regions);

        // Inside first region (10, 20, 100, 50)
        assert!(config.is_interactive(50, 40));

        // Inside second region (200, 100, 150, 80)
        assert!(config.is_interactive(250, 140));

        // Outside all regions
        assert!(!config.is_interactive(0, 0));
        assert!(!config.is_interactive(150, 50));
        assert!(!config.is_interactive(500, 500));
    }

    #[rstest]
    fn test_is_interactive_empty_regions() {
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(vec![]);

        // With no regions, nothing is interactive (all clicks pass through)
        assert!(!config.is_interactive(0, 0));
        assert!(!config.is_interactive(100, 100));
    }

    #[rstest]
    fn test_overlapping_regions() {
        let regions = vec![
            InteractiveRegion::new(0, 0, 100, 100),
            InteractiveRegion::new(50, 50, 100, 100), // Overlaps with first
        ];
        let config = ClickThroughConfig::new()
            .with_enabled(true)
            .with_regions(regions);

        // Point in overlap area
        assert!(config.is_interactive(75, 75));

        // Point only in first region
        assert!(config.is_interactive(25, 25));

        // Point only in second region
        assert!(config.is_interactive(125, 125));

        // Point outside both
        assert!(!config.is_interactive(200, 200));
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
}
