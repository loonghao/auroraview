//! Window style utilities tests

use auroraview_core::builder::{
    compute_frameless_popup_window_styles, compute_frameless_window_styles,
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
    let style = WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_DLGFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX;
    let (new_style, _) = compute_frameless_window_styles(style, 0);
    assert_eq!(new_style, 0, "All frame bits should be cleared");
}

#[test]
fn frameless_window_all_ex_bits_removed_simultaneously() {
    let ex_style = WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE;
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
    let style = WS_CAPTION | WS_THICKFRAME | WS_BORDER | WS_DLGFRAME | WS_SYSMENU | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_CHILD;
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
#[case(0x00000010_i32, 0)]
#[case(0x00000020_i32, 0)]
#[case(0x00000040_i32, 0)]
fn frameless_window_unrelated_bits_preserved_parametric(#[case] custom_bit: i32, #[case] _zero: i32) {
    let (new_style, _) = compute_frameless_window_styles(custom_bit, 0);
    assert_ne!(new_style & custom_bit, 0, "Custom bit {:#010x} should be preserved", custom_bit);
}

#[rstest]
#[case(0x00000010_i32)]
#[case(0x00000004_i32)]
#[case(0x00000002_i32)]
fn frameless_popup_unrelated_ex_bits_preserved(#[case] custom_ex: i32) {
    let (_, new_ex) = compute_frameless_popup_window_styles(0, custom_ex);
    assert_ne!(new_ex & custom_ex, 0, "Ex bit {:#010x} should survive popup transform", custom_ex);
}

#[test]
fn frameless_window_multiple_applications_identical() {
    let style = WS_CAPTION | WS_THICKFRAME | 0x00000010;
    let ex = WS_EX_WINDOWEDGE | 0x00000008;
    let (s1, e1) = compute_frameless_window_styles(style, ex);
    let (s2, e2) = compute_frameless_window_styles(style, ex);
    assert_eq!(s1, s2);
    assert_eq!(e1, e2);
}

#[test]
fn frameless_popup_multiple_applications_identical() {
    let style = WS_CHILD | WS_CAPTION;
    let (s1, e1) = compute_frameless_popup_window_styles(style, 0);
    let (s2, e2) = compute_frameless_popup_window_styles(style, 0);
    assert_eq!(s1, s2);
    assert_eq!(e1, e2);
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
    assert_ne!(new_style, style, "Output should differ when frame bits are removed");
}

#[test]
fn frameless_popup_not_equal_to_input() {
    let style = WS_CHILD | WS_CAPTION;
    let (new_style, _) = compute_frameless_popup_window_styles(style, 0);
    assert_ne!(new_style, style, "Output should differ after adding WS_POPUP and removing WS_CHILD");
}
