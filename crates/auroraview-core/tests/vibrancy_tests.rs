//! Tests for vibrancy (background blur) functionality

use auroraview_core::builder::{VibrancyEffect, VibrancyResult};
use rstest::*;

mod vibrancy_effect_tests {
    use super::*;

    #[rstest]
    fn test_effect_default() {
        let effect = VibrancyEffect::default();
        assert_eq!(effect, VibrancyEffect::None);
    }

    #[rstest]
    #[case(VibrancyEffect::None)]
    #[case(VibrancyEffect::Blur)]
    #[case(VibrancyEffect::Acrylic)]
    #[case(VibrancyEffect::Mica)]
    #[case(VibrancyEffect::MicaAlt)]
    fn test_effect_variants(#[case] effect: VibrancyEffect) {
        // Test that all variants can be created
        let _ = effect;
    }

    #[rstest]
    fn test_effect_equality() {
        assert_eq!(VibrancyEffect::Blur, VibrancyEffect::Blur);
        assert_ne!(VibrancyEffect::Blur, VibrancyEffect::Acrylic);
        assert_ne!(VibrancyEffect::Mica, VibrancyEffect::MicaAlt);
    }

    #[rstest]
    fn test_effect_clone() {
        let effect = VibrancyEffect::Acrylic;
        let cloned = effect;
        assert_eq!(effect, cloned);
    }
}

mod vibrancy_result_tests {
    use super::*;

    #[rstest]
    fn test_result_success() {
        let result = VibrancyResult {
            success: true,
            effect: VibrancyEffect::Blur,
            error: None,
        };
        assert!(result.success);
        assert_eq!(result.effect, VibrancyEffect::Blur);
        assert!(result.error.is_none());
    }

    #[rstest]
    fn test_result_error() {
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::Mica,
            error: Some("Test error".to_string()),
        };
        assert!(!result.success);
        assert_eq!(result.effect, VibrancyEffect::Mica);
        assert_eq!(result.error, Some("Test error".to_string()));
    }
}

mod vibrancy_color_tests {
    use auroraview_core::builder::VibrancyColor;

    #[rstest::rstest]
    #[case((0, 0, 0, 0))] // Fully transparent black
    #[case((255, 255, 255, 255))] // Fully opaque white
    #[case((30, 30, 30, 200))] // Dark with high opacity
    #[case((128, 128, 128, 128))] // Mid gray, mid opacity
    fn test_color_values(#[case] color: VibrancyColor) {
        // VibrancyColor is (u8, u8, u8, u8), all values are valid by type constraint
        let (_r, _g, _b, _a) = color;
        // Test that the color can be destructured and used
    }
}

mod platform_detection_tests {
    use auroraview_core::builder::{
        is_backdrop_type_supported, is_mica_supported, is_swca_supported,
    };

    #[rstest::rstest]
    fn test_platform_detection_functions() {
        // These functions should not panic on any platform
        let _ = is_swca_supported();
        let _ = is_mica_supported();
        let _ = is_backdrop_type_supported();
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_non_windows_returns_false() {
        assert!(!is_swca_supported());
        assert!(!is_mica_supported());
        assert!(!is_backdrop_type_supported());
    }
}

mod effect_serialization_tests {
    use super::*;

    #[rstest]
    fn test_effect_serialize() {
        let effect = VibrancyEffect::Acrylic;
        let json = serde_json::to_string(&effect).unwrap();
        assert_eq!(json, "\"Acrylic\"");
    }

    #[rstest]
    fn test_effect_deserialize() {
        let json = "\"Mica\"";
        let effect: VibrancyEffect = serde_json::from_str(json).unwrap();
        assert_eq!(effect, VibrancyEffect::Mica);
    }

    #[rstest]
    #[case(VibrancyEffect::None, "\"None\"")]
    #[case(VibrancyEffect::Blur, "\"Blur\"")]
    #[case(VibrancyEffect::Acrylic, "\"Acrylic\"")]
    #[case(VibrancyEffect::Mica, "\"Mica\"")]
    #[case(VibrancyEffect::MicaAlt, "\"MicaAlt\"")]
    fn test_effect_roundtrip(#[case] effect: VibrancyEffect, #[case] expected_json: &str) {
        let json = serde_json::to_string(&effect).unwrap();
        assert_eq!(json, expected_json);

        let deserialized: VibrancyEffect = serde_json::from_str(&json).unwrap();
        assert_eq!(effect, deserialized);
    }
}
