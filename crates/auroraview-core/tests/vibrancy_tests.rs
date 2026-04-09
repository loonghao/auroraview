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
        let _ = effect;
    }

    #[rstest]
    fn test_effect_equality() {
        assert_eq!(VibrancyEffect::Blur, VibrancyEffect::Blur);
        assert_ne!(VibrancyEffect::Blur, VibrancyEffect::Acrylic);
        assert_ne!(VibrancyEffect::Mica, VibrancyEffect::MicaAlt);
        assert_ne!(VibrancyEffect::None, VibrancyEffect::Blur);
        assert_ne!(VibrancyEffect::Acrylic, VibrancyEffect::MicaAlt);
    }

    #[rstest]
    fn test_effect_clone() {
        let effect = VibrancyEffect::Acrylic;
        let cloned = effect;
        assert_eq!(effect, cloned);
    }

    #[rstest]
    fn test_effect_copy() {
        let effect = VibrancyEffect::Mica;
        let copied = effect;
        // Both are valid after copy
        assert_eq!(effect, copied);
        assert_eq!(copied, VibrancyEffect::Mica);
    }

    #[rstest]
    fn test_effect_debug_format() {
        assert_eq!(format!("{:?}", VibrancyEffect::None), "None");
        assert_eq!(format!("{:?}", VibrancyEffect::Blur), "Blur");
        assert_eq!(format!("{:?}", VibrancyEffect::Acrylic), "Acrylic");
        assert_eq!(format!("{:?}", VibrancyEffect::Mica), "Mica");
        assert_eq!(format!("{:?}", VibrancyEffect::MicaAlt), "MicaAlt");
    }

    #[rstest]
    fn test_all_variants_not_equal_to_none() {
        for effect in &[
            VibrancyEffect::Blur,
            VibrancyEffect::Acrylic,
            VibrancyEffect::Mica,
            VibrancyEffect::MicaAlt,
        ] {
            assert_ne!(*effect, VibrancyEffect::None);
        }
    }

    #[rstest]
    fn test_all_variants_are_distinct() {
        let variants = [
            VibrancyEffect::None,
            VibrancyEffect::Blur,
            VibrancyEffect::Acrylic,
            VibrancyEffect::Mica,
            VibrancyEffect::MicaAlt,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                if i == j {
                    assert_eq!(a, b);
                } else {
                    assert_ne!(a, b);
                }
            }
        }
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

    #[rstest]
    fn test_result_none_effect_success() {
        let result = VibrancyResult {
            success: true,
            effect: VibrancyEffect::None,
            error: None,
        };
        assert!(result.success);
        assert_eq!(result.effect, VibrancyEffect::None);
    }

    #[rstest]
    fn test_result_acrylic_success() {
        let result = VibrancyResult {
            success: true,
            effect: VibrancyEffect::Acrylic,
            error: None,
        };
        assert!(result.success);
        assert_eq!(result.effect, VibrancyEffect::Acrylic);
        assert!(result.error.is_none());
    }

    #[rstest]
    fn test_result_mica_alt_error() {
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::MicaAlt,
            error: Some("Mica Alt requires Windows 11 (build 22523) or later".to_string()),
        };
        assert!(!result.success);
        assert_eq!(result.effect, VibrancyEffect::MicaAlt);
        assert!(result.error.unwrap().contains("22523"));
    }

    #[rstest]
    fn test_result_debug_is_available() {
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::Blur,
            error: Some("some error".to_string()),
        };
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("success"));
        assert!(debug_str.contains("effect"));
    }

    #[rstest]
    fn test_result_error_message_unicode() {
        let msg = "エラー: 効果を適用できませんでした";
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::Blur,
            error: Some(msg.to_string()),
        };
        assert_eq!(result.error.as_deref(), Some(msg));
    }

    #[rstest]
    fn test_result_error_long_message() {
        let msg = "x".repeat(1024);
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::Mica,
            error: Some(msg.clone()),
        };
        assert_eq!(result.error.as_deref(), Some(msg.as_str()));
        assert_eq!(result.error.unwrap().len(), 1024);
    }

    #[rstest]
    fn test_result_false_success_implies_error_present() {
        // Conventionally, if success=false there should be an error message
        let result = VibrancyResult {
            success: false,
            effect: VibrancyEffect::Acrylic,
            error: Some("Platform not supported".to_string()),
        };
        assert!(!result.success);
        assert!(result.error.is_some());
    }
}

mod vibrancy_color_tests {
    use auroraview_core::builder::VibrancyColor;

    #[rstest::rstest]
    #[case((0, 0, 0, 0))]
    #[case((255, 255, 255, 255))]
    #[case((30, 30, 30, 200))]
    #[case((128, 128, 128, 128))]
    #[case((255, 0, 0, 255))]
    #[case((0, 255, 0, 128))]
    #[case((0, 0, 255, 64))]
    fn test_color_values(#[case] color: VibrancyColor) {
        let (_r, _g, _b, _a) = color;
    }

    #[rstest::rstest]
    fn test_color_components_range() {
        let (r, g, b, a): VibrancyColor = (200, 150, 100, 50);
        // u8 values are always 0-255, just verify they can be used
        let _ = (r, g, b, a);
    }

    #[rstest::rstest]
    fn test_transparent_color() {
        let color: VibrancyColor = (0, 0, 0, 0);
        let (_, _, _, a) = color;
        assert_eq!(a, 0);
    }

    #[rstest::rstest]
    fn test_opaque_white() {
        let color: VibrancyColor = (255, 255, 255, 255);
        let (r, g, b, a) = color;
        assert_eq!(r, 255);
        assert_eq!(g, 255);
        assert_eq!(b, 255);
        assert_eq!(a, 255);
    }
}

mod platform_detection_tests {
    use auroraview_core::builder::{
        is_backdrop_type_supported, is_mica_supported, is_swca_supported,
    };

    #[rstest::rstest]
    fn test_platform_detection_functions() {
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

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_non_windows_swca_false() {
        assert!(!is_swca_supported());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_non_windows_mica_false() {
        assert!(!is_mica_supported());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_non_windows_backdrop_false() {
        assert!(!is_backdrop_type_supported());
    }

    #[rstest::rstest]
    fn test_detection_is_idempotent() {
        let a = is_swca_supported();
        let b = is_swca_supported();
        assert_eq!(a, b);

        let c = is_mica_supported();
        let d = is_mica_supported();
        assert_eq!(c, d);
    }

    // On Windows: backdrop_type requires build >= 22523, mica requires >= 22000
    // If backdrop is supported, mica must also be supported (22523 > 22000)
    #[cfg(target_os = "windows")]
    #[rstest::rstest]
    fn test_windows_backdrop_implies_mica() {
        if is_backdrop_type_supported() {
            assert!(is_mica_supported());
        }
    }
}

mod vibrancy_api_tests {
    #[cfg(not(target_os = "windows"))]
    use auroraview_core::builder::{
        apply_acrylic, apply_blur, apply_mica, apply_mica_alt, clear_acrylic, clear_blur,
        clear_mica, clear_mica_alt,
    };

    // On non-Windows all APIs return an error VibrancyResult
    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_blur_non_windows() {
        let result = apply_blur(0, None);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_acrylic_non_windows() {
        let result = apply_acrylic(0, None);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_mica_non_windows() {
        let result = apply_mica(0, false);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_mica_alt_non_windows() {
        let result = apply_mica_alt(0, false);
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_clear_blur_non_windows() {
        let result = clear_blur(0);
        assert!(!result.success);
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_clear_acrylic_non_windows() {
        let result = clear_acrylic(0);
        assert!(!result.success);
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_clear_mica_non_windows() {
        let result = clear_mica(0);
        assert!(!result.success);
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_clear_mica_alt_non_windows() {
        let result = clear_mica_alt(0);
        assert!(!result.success);
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_blur_with_color_non_windows() {
        let result = apply_blur(0, Some((30, 30, 30, 200)));
        assert!(!result.success);
        assert!(result
            .error
            .as_deref()
            .map_or(false, |e| e.contains("Windows")));
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_apply_acrylic_with_color_non_windows() {
        let result = apply_acrylic(0, Some((0, 0, 0, 128)));
        assert!(!result.success);
    }

    #[cfg(not(target_os = "windows"))]
    #[rstest::rstest]
    fn test_mica_dark_mode_non_windows() {
        let result_light = apply_mica(0, false);
        let result_dark = apply_mica(0, true);
        // Both fail on non-Windows
        assert!(!result_light.success);
        assert!(!result_dark.success);
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

    #[rstest]
    fn test_effect_deserialize_unknown_returns_error() {
        let json = "\"UnknownEffect\"";
        let result: Result<VibrancyEffect, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[rstest]
    fn test_effect_serialize_all_uppercase_start() {
        // All variant names start with uppercase
        for effect in &[
            VibrancyEffect::None,
            VibrancyEffect::Blur,
            VibrancyEffect::Acrylic,
            VibrancyEffect::Mica,
            VibrancyEffect::MicaAlt,
        ] {
            let json = serde_json::to_string(effect).unwrap();
            let name = json.trim_matches('"');
            assert!(name.chars().next().unwrap().is_uppercase());
        }
    }
}
