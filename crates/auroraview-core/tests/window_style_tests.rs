//! Window style utilities tests

use auroraview_core::builder::{
    compute_frameless_popup_window_styles, compute_frameless_window_styles,
    fix_webview2_child_windows,
};
use rstest::rstest;

// WinUser.h style constants
const WS_CAPTION: i32 = 0x00C00000;
const WS_THICKFRAME: i32 = 0x00040000;
const WS_BORDER: i32 = 0x00800000;
const WS_DLGFRAME: i32 = 0x00400000;
const WS_SYSMENU: i32 = 0x00080000;
const WS_MINIMIZEBOX: i32 = 0x00020000;
const WS_MAXIMIZEBOX: i32 = 0x00010000;
const WS_POPUP: i32 = 0x80000000u32 as i32;
const WS_CHILD: i32 = 0x40000000;

const WS_EX_DLGMODALFRAME: i32 = 0x00000001;
const WS_EX_WINDOWEDGE: i32 = 0x00000100;
const WS_EX_CLIENTEDGE: i32 = 0x00000200;
const WS_EX_STATICEDGE: i32 = 0x00020000;
const WS_EX_CONTEXTHELP: i32 = 0x00000400;

// ============================================================================
// compute_frameless_window_styles
// ============================================================================

#[test]
fn frameless_window_removes_caption_and_frame_bits() {
    let style = WS_CAPTION
        | WS_THICKFRAME
        | WS_BORDER
        | WS_DLGFRAME
        | WS_SYSMENU
        | WS_MINIMIZEBOX
        | WS_MAXIMIZEBOX
        | 0x00000010;
    let ex_style =
        WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE | 0x00000008;

    let (new_style, new_ex_style) = compute_frameless_window_styles(style, ex_style);

    assert_eq!(new_style & WS_CAPTION, 0);
    assert_eq!(new_style & WS_THICKFRAME, 0);
    assert_eq!(new_style & WS_BORDER, 0);
    assert_eq!(new_style & WS_DLGFRAME, 0);
    assert_eq!(new_style & WS_SYSMENU, 0);
    assert_eq!(new_style & WS_MINIMIZEBOX, 0);
    assert_eq!(new_style & WS_MAXIMIZEBOX, 0);

    assert_eq!(new_ex_style & WS_EX_DLGMODALFRAME, 0);
    assert_eq!(new_ex_style & WS_EX_WINDOWEDGE, 0);
    assert_eq!(new_ex_style & WS_EX_CLIENTEDGE, 0);
    assert_eq!(new_ex_style & WS_EX_STATICEDGE, 0);

    // Unrelated bits preserved
    assert_ne!(new_style & 0x00000010, 0);
    assert_ne!(new_ex_style & 0x00000008, 0);
}

#[test]
fn frameless_window_all_zeros_input() {
    let (new_style, new_ex_style) = compute_frameless_window_styles(0, 0);
    assert_eq!(new_style, 0);
    assert_eq!(new_ex_style, 0);
}

#[test]
fn frameless_window_no_frame_bits_unchanged() {
    // Only set bits that are NOT removed by compute_frameless_window_styles
    let custom_bit: i32 = 0x00000010;
    let (new_style, _) = compute_frameless_window_styles(custom_bit, 0);
    assert_ne!(new_style & custom_bit, 0);
}

#[test]
fn frameless_window_only_caption_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_CAPTION, 0);
    assert_eq!(new_style & WS_CAPTION, 0);
}

#[test]
fn frameless_window_only_thickframe_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_THICKFRAME, 0);
    assert_eq!(new_style & WS_THICKFRAME, 0);
}

#[test]
fn frameless_window_only_border_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_BORDER, 0);
    assert_eq!(new_style & WS_BORDER, 0);
}

#[test]
fn frameless_window_only_dlgframe_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_DLGFRAME, 0);
    assert_eq!(new_style & WS_DLGFRAME, 0);
}

#[test]
fn frameless_window_only_sysmenu_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_SYSMENU, 0);
    assert_eq!(new_style & WS_SYSMENU, 0);
}

#[test]
fn frameless_window_only_minimizebox_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_MINIMIZEBOX, 0);
    assert_eq!(new_style & WS_MINIMIZEBOX, 0);
}

#[test]
fn frameless_window_only_maximizebox_bit() {
    let (new_style, _) = compute_frameless_window_styles(WS_MAXIMIZEBOX, 0);
    assert_eq!(new_style & WS_MAXIMIZEBOX, 0);
}

#[rstest]
#[case(WS_EX_DLGMODALFRAME)]
#[case(WS_EX_WINDOWEDGE)]
#[case(WS_EX_CLIENTEDGE)]
#[case(WS_EX_STATICEDGE)]
#[case(WS_EX_CONTEXTHELP)]
fn frameless_window_ex_bits_removed(#[case] ex_bit: i32) {
    let (_, new_ex_style) = compute_frameless_window_styles(0, ex_bit);
    assert_eq!(new_ex_style & ex_bit, 0);
}

#[test]
fn frameless_window_idempotent() {
    let style = WS_CAPTION | WS_THICKFRAME | 0x00000010;
    let ex_style = WS_EX_WINDOWEDGE | 0x00000008;

    let (new_style_1, new_ex_style_1) = compute_frameless_window_styles(style, ex_style);
    let (new_style_2, new_ex_style_2) =
        compute_frameless_window_styles(new_style_1, new_ex_style_1);

    assert_eq!(new_style_1, new_style_2);
    assert_eq!(new_ex_style_1, new_ex_style_2);
}

// ============================================================================
// compute_frameless_popup_window_styles
// ============================================================================

#[test]
fn frameless_popup_sets_ws_popup_clears_ws_child() {
    let style = WS_CHILD | WS_CAPTION | 0x00000010;
    let ex_style = 0x00000008;

    let (new_style, new_ex_style) = compute_frameless_popup_window_styles(style, ex_style);

    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_style & WS_CHILD, 0);
    assert_eq!(new_style & WS_CAPTION, 0);
    assert_eq!(new_ex_style, ex_style);
}

#[test]
fn frameless_popup_zero_input() {
    let (new_style, new_ex_style) = compute_frameless_popup_window_styles(0, 0);
    // WS_POPUP should be set even from zero
    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_ex_style, 0);
}

#[test]
fn frameless_popup_preserves_unrelated_style_bits() {
    let custom_bit: i32 = 0x00000010;
    let style = WS_CHILD | custom_bit;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_style & WS_CHILD, 0);
    assert_ne!(new_style & custom_bit, 0);
}

#[test]
fn frameless_popup_all_frame_bits() {
    let style = WS_CHILD | WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_DLGFRAME;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_style & WS_CHILD, 0);
    assert_eq!(new_style & WS_CAPTION, 0);
    assert_eq!(new_style & WS_THICKFRAME, 0);
    assert_eq!(new_style & WS_BORDER, 0);
    assert_eq!(new_style & WS_DLGFRAME, 0);
}

#[test]
fn frameless_popup_idempotent() {
    let style = WS_CHILD | WS_CAPTION;
    let ex_style = 0;

    let (s1, e1) = compute_frameless_popup_window_styles(style, ex_style);
    let (s2, e2) = compute_frameless_popup_window_styles(s1, e1);

    assert_eq!(s1, s2);
    assert_eq!(e1, e2);
}

// ============================================================================
// Additional edge case tests
// ============================================================================

#[test]
fn frameless_window_only_ex_windowedge() {
    let (_, new_ex) = compute_frameless_window_styles(0, WS_EX_WINDOWEDGE);
    assert_eq!(new_ex & WS_EX_WINDOWEDGE, 0);
}

#[test]
fn frameless_window_only_ex_clientedge() {
    let (_, new_ex) = compute_frameless_window_styles(0, WS_EX_CLIENTEDGE);
    assert_eq!(new_ex & WS_EX_CLIENTEDGE, 0);
}

#[test]
fn frameless_window_only_ex_staticedge() {
    let (_, new_ex) = compute_frameless_window_styles(0, WS_EX_STATICEDGE);
    assert_eq!(new_ex & WS_EX_STATICEDGE, 0);
}

#[test]
fn frameless_window_only_ex_dlgmodalframe() {
    let (_, new_ex) = compute_frameless_window_styles(0, WS_EX_DLGMODALFRAME);
    assert_eq!(new_ex & WS_EX_DLGMODALFRAME, 0);
}

#[rstest]
#[case(WS_CAPTION)]
#[case(WS_THICKFRAME)]
#[case(WS_BORDER)]
#[case(WS_DLGFRAME)]
#[case(WS_SYSMENU)]
#[case(WS_MINIMIZEBOX)]
#[case(WS_MAXIMIZEBOX)]
fn frameless_window_single_style_bit_removed(#[case] bit: i32) {
    let (new_style, _) = compute_frameless_window_styles(bit, 0);
    assert_eq!(new_style & bit, 0, "Bit {:#010x} should be cleared", bit);
}

#[test]
fn frameless_popup_preserves_ex_style_unchanged() {
    let ex_style = 0x00000008;
    let (_, new_ex) = compute_frameless_popup_window_styles(0, ex_style);
    // popup function should not modify ex_style
    assert_eq!(new_ex, ex_style);
}

#[test]
fn frameless_popup_no_child_input_still_sets_popup() {
    // Input without WS_CHILD should still gain WS_POPUP
    let style: i32 = 0x00000010; // some unrelated bit
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(new_style & WS_POPUP, 0);
}

#[test]
fn frameless_window_and_popup_compose_cleanly() {
    // First apply frameless, then popup — result should have WS_POPUP and no frame bits
    let style = WS_CAPTION | WS_THICKFRAME | WS_CHILD | 0x10;
    let ex_style = WS_EX_WINDOWEDGE;

    let (s1, e1) = compute_frameless_window_styles(style, ex_style);
    let (s2, e2) = compute_frameless_popup_window_styles(s1, e1);

    assert_eq!(s2 & WS_CAPTION, 0);
    assert_eq!(s2 & WS_THICKFRAME, 0);
    assert_ne!(s2 & WS_POPUP, 0);
    assert_eq!(e1 & WS_EX_WINDOWEDGE, 0);
    assert_eq!(e2, e1); // popup doesn't touch ex_style
}

// ============================================================================
// R10 Extensions
// ============================================================================

#[test]
fn frameless_window_all_style_bits_removed_simultaneously() {
    let style = WS_CAPTION
        | WS_THICKFRAME
        | WS_BORDER
        | WS_DLGFRAME
        | WS_SYSMENU
        | WS_MINIMIZEBOX
        | WS_MAXIMIZEBOX;
    let (new_style, _) = compute_frameless_window_styles(style, 0);
    assert_eq!(new_style, 0, "All frame bits should be cleared");
}

#[test]
fn frameless_window_all_ex_bits_removed_simultaneously() {
    let ex_style = WS_EX_DLGMODALFRAME
        | WS_EX_WINDOWEDGE
        | WS_EX_CLIENTEDGE
        | WS_EX_STATICEDGE
        | WS_EX_CONTEXTHELP;
    let (_, new_ex) = compute_frameless_window_styles(0, ex_style);
    assert_eq!(new_ex, 0, "All ex frame bits should be cleared");
}

#[test]
fn frameless_window_high_bit_unrelated_preserved() {
    let custom: i32 = 0x00100000;
    let (new_style, _) = compute_frameless_window_styles(custom, 0);
    assert_ne!(new_style & custom, 0);
}

#[test]
fn frameless_popup_adds_popup_without_losing_custom_bit() {
    let custom: i32 = 0x00100000;
    let (new_style, _) = compute_frameless_popup_window_styles(custom, 0);
    assert_ne!(new_style & WS_POPUP, 0);
    assert_ne!(new_style & custom, 0);
}

#[test]
fn frameless_popup_removes_all_caption_bits() {
    let style = WS_CAPTION
        | WS_THICKFRAME
        | WS_BORDER
        | WS_DLGFRAME
        | WS_SYSMENU
        | WS_MINIMIZEBOX
        | WS_MAXIMIZEBOX
        | WS_CHILD;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_eq!(new_style & WS_CAPTION, 0);
    assert_eq!(new_style & WS_THICKFRAME, 0);
    assert_eq!(new_style & WS_BORDER, 0);
    assert_eq!(new_style & WS_DLGFRAME, 0);
    assert_eq!(new_style & WS_SYSMENU, 0);
    assert_eq!(new_style & WS_MINIMIZEBOX, 0);
    assert_eq!(new_style & WS_MAXIMIZEBOX, 0);
    assert_eq!(new_style & WS_CHILD, 0);
}

#[rstest]
#[case(0x00000010_i32)]
#[case(0x00000020_i32)]
#[case(0x00000040_i32)]
fn frameless_window_unrelated_bits_preserved_parametric(#[case] custom_bit: i32) {
    let (new_style, _) = compute_frameless_window_styles(custom_bit, 0);
    assert_ne!(
        new_style & custom_bit,
        0,
        "Custom bit {:#010x} should be preserved",
        custom_bit
    );
}

#[rstest]
#[case(0x00000010_i32)]
#[case(0x00000004_i32)]
#[case(0x00000002_i32)]
fn frameless_popup_unrelated_ex_bits_preserved(#[case] custom_ex: i32) {
    let (_, new_ex) = compute_frameless_popup_window_styles(0, custom_ex);
    assert_ne!(
        new_ex & custom_ex,
        0,
        "Ex bit {:#010x} should survive popup transform",
        custom_ex
    );
}

#[test]
fn frameless_window_return_type_is_tuple_i32() {
    let result: (i32, i32) = compute_frameless_window_styles(0, 0);
    let _ = result;
}

#[test]
fn frameless_popup_return_type_is_tuple_i32() {
    let result: (i32, i32) = compute_frameless_popup_window_styles(0, 0);
    let _ = result;
}

#[test]
fn frameless_window_caption_and_border_removed_independently() {
    let (s1, _) = compute_frameless_window_styles(WS_CAPTION, 0);
    let (s2, _) = compute_frameless_window_styles(WS_BORDER, 0);
    assert_eq!(s1 & WS_CAPTION, 0);
    assert_eq!(s2 & WS_BORDER, 0);
    // Each removal is independent
    assert_eq!(s1 & WS_BORDER, 0); // not set in input
    assert_eq!(s2 & WS_CAPTION, 0); // not set in input
}

#[test]
fn frameless_window_not_equal_to_input_when_frame_bits_set() {
    let style = WS_CAPTION | WS_THICKFRAME;
    let (new_style, _) = compute_frameless_window_styles(style, 0);
    assert_ne!(
        new_style, style,
        "Output should differ when frame bits are removed"
    );
}

#[test]
fn frameless_popup_not_equal_to_input() {
    let style = WS_CHILD | WS_CAPTION;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(
        new_style, style,
        "Output should differ after adding WS_POPUP and removing WS_CHILD"
    );
}

// ============================================================================
// R15 Extensions
// ============================================================================

#[test]
fn frameless_window_popup_bit_not_added() {
    // compute_frameless_window_styles should NOT add WS_POPUP
    let (new_style, _) = compute_frameless_window_styles(0, 0);
    assert_eq!(new_style & WS_POPUP, 0);
}

#[test]
fn frameless_popup_clears_sysmenu() {
    let style = WS_SYSMENU | WS_CHILD;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_eq!(new_style & WS_SYSMENU, 0);
}

#[test]
fn frameless_popup_clears_minimizebox() {
    let style = WS_MINIMIZEBOX | WS_CHILD;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_eq!(new_style & WS_MINIMIZEBOX, 0);
}

#[test]
fn frameless_popup_clears_maximizebox() {
    let style = WS_MAXIMIZEBOX | WS_CHILD;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_eq!(new_style & WS_MAXIMIZEBOX, 0);
}

#[rstest]
#[case(WS_CAPTION, WS_POPUP)]
#[case(WS_THICKFRAME, WS_POPUP)]
#[case(WS_CHILD, WS_POPUP)]
fn frameless_popup_always_sets_ws_popup(#[case] input_bit: i32, #[case] expected_bit: i32) {
    let (new_style, _) = compute_frameless_popup_window_styles(input_bit, 0);
    assert_ne!(new_style & expected_bit, 0);
}

#[test]
fn frameless_window_output_style_is_not_negative_when_no_high_bits() {
    let style: i32 = 0x00000010;
    let (new_style, _) = compute_frameless_window_styles(style, 0);
    // Custom low bit should still be set (not cleared)
    assert_ne!(new_style & style, 0);
}

#[test]
fn frameless_window_ex_style_only_frame_bits_all_cleared() {
    let ex = WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE;
    let (_, new_ex) = compute_frameless_window_styles(0, ex);
    assert_eq!(new_ex & WS_EX_DLGMODALFRAME, 0);
    assert_eq!(new_ex & WS_EX_WINDOWEDGE, 0);
    assert_eq!(new_ex & WS_EX_CLIENTEDGE, 0);
    assert_eq!(new_ex & WS_EX_STATICEDGE, 0);
}

#[test]
fn frameless_window_chain_with_popup_preserves_popup() {
    // Apply frameless then popup: popup bit must survive second call
    let style = WS_CAPTION | WS_CHILD;
    let (s1, e1) = compute_frameless_window_styles(style, 0);
    let (s2, _) = compute_frameless_popup_window_styles(s1, e1);
    assert_ne!(s2 & WS_POPUP, 0);
}

#[rstest]
#[case(WS_EX_DLGMODALFRAME, 0i32)]
#[case(0i32, WS_EX_WINDOWEDGE)]
#[case(WS_EX_CLIENTEDGE, WS_EX_STATICEDGE)]
fn frameless_window_combined_ex_bits_all_cleared(#[case] ex_a: i32, #[case] ex_b: i32) {
    let (_, new_ex) = compute_frameless_window_styles(0, ex_a | ex_b);
    assert_eq!(new_ex & ex_a, 0);
    assert_eq!(new_ex & ex_b, 0);
}

#[test]
fn frameless_popup_with_multiple_bits_sets_popup_clears_child() {
    let style = WS_CHILD | WS_CAPTION | WS_BORDER | 0x00000004;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_style & WS_CHILD, 0);
    assert_eq!(new_style & WS_CAPTION, 0);
    assert_eq!(new_style & WS_BORDER, 0);
    assert_ne!(new_style & 0x00000004, 0);
}

// ============================================================================
// fix_webview2_child_windows (public export)
// ============================================================================

// The function enumerates child windows of the given top-level HWND. A NULL (0)
// handle is special-cased: `EnumChildWindows(NULL, ...)` would otherwise iterate
// every top-level window on the desktop and mutate them, so the production guard
// must short-circuit and return immediately (no panic, no desktop-wide mutation).
#[test]
fn fix_webview2_child_windows_null_handle_returns_immediately() {
    fix_webview2_child_windows(0, false);
}

#[rstest]
#[case(0x1_isize)]
#[case(0xDEAD_BEEF_isize)]
#[case(-1_isize)]
fn fix_webview2_child_windows_arbitrary_handles_do_not_panic(#[case] hwnd: isize) {
    // Cross-platform note: the two builds verify different code paths under the
    // same "must not panic" contract. On Linux CI this hits the no-op stub
    // (window_style.rs), which trivially can't panic. On Windows these invalid
    // HWNDs reach the real `EnumChildWindows`, which Win32 tolerates by
    // returning FALSE rather than panicking — so this is where the contract is
    // genuinely exercised.
    fix_webview2_child_windows(hwnd, false);
}

// ============================================================================
// Non-Windows stub coverage
//
// Rust coverage is measured on Linux CI, where every `#[cfg(target_os =
// "windows")]` styling entry point is compiled out and its non-Windows stub
// takes its place. The real-window tests in the crate's internal module only
// run on Windows, so without these the stub bodies — the only versions of
// these functions that exist on the coverage platform — show up as uncovered.
//
// Each stub has a documented contract: the `Result`-returning ones must report
// the "Windows only" error, `apply_owner_window_style` must hand back an
// all-zero / false result, and the rest are no-ops that must simply not panic.
// ============================================================================
#[cfg(not(target_os = "windows"))]
mod non_windows_stubs {
    use auroraview_core::builder::{
        apply_child_window_style, apply_frameless_popup_window_style, apply_frameless_window_style,
        apply_owner_window_style, apply_tool_window_style, disable_window_shadow,
        extend_frame_into_client_area, optimize_transparent_window_resize,
        remove_clip_children_style, set_window_class_dark_background, subclass_for_zero_nc_area,
        ChildWindowStyleOptions,
    };

    #[test]
    fn apply_child_window_style_is_unsupported() {
        let err = apply_child_window_style(1, 2, ChildWindowStyleOptions::for_dcc_embedding())
            .expect_err("non-Windows stub must return Err");
        assert!(
            err.contains("only supported on Windows"),
            "unexpected error message: {err}"
        );
    }

    #[test]
    fn apply_frameless_window_style_is_unsupported() {
        let err = apply_frameless_window_style(1).expect_err("non-Windows stub must return Err");
        assert!(err.contains("only supported on Windows"), "{err}");
    }

    #[test]
    fn apply_frameless_popup_window_style_is_unsupported() {
        let err =
            apply_frameless_popup_window_style(1).expect_err("non-Windows stub must return Err");
        assert!(err.contains("only supported on Windows"), "{err}");
    }

    #[test]
    fn apply_owner_window_style_returns_zeroed_result() {
        // tool_window=true on the stub is still a no-op: the result is fixed.
        let res = apply_owner_window_style(1, 2, true);
        assert_eq!(res.old_ex_style, 0);
        assert_eq!(res.new_ex_style, 0);
        assert!(!res.tool_window);
    }

    #[test]
    fn noop_stubs_do_not_panic() {
        // Each of these is a `#[cfg(not(target_os = "windows"))]` no-op; calling
        // them exercises the stub body and proves the cross-platform surface
        // stays callable without a Windows handle.
        subclass_for_zero_nc_area(1);
        apply_tool_window_style(1);
        disable_window_shadow(1);
        set_window_class_dark_background(1);
        extend_frame_into_client_area(1);
        optimize_transparent_window_resize(1);
        remove_clip_children_style(1);
    }
}
