//! Window style utilities tests

use auroraview_core::builder::{
    compute_frameless_popup_window_styles, compute_frameless_window_styles,
};

#[test]
fn test_compute_frameless_window_styles_removes_caption_and_frame_bits() {
    // Construct a fake style/ex_style with bits that should be removed.
    // These bit values match WinUser.h.
    const WS_CAPTION: i32 = 0x00C00000;
    const WS_THICKFRAME: i32 = 0x00040000;
    const WS_BORDER: i32 = 0x00800000;
    const WS_DLGFRAME: i32 = 0x00400000;
    const WS_SYSMENU: i32 = 0x00080000;
    const WS_MINIMIZEBOX: i32 = 0x00020000;
    const WS_MAXIMIZEBOX: i32 = 0x00010000;

    const WS_EX_DLGMODALFRAME: i32 = 0x00000001;
    const WS_EX_WINDOWEDGE: i32 = 0x00000100;
    const WS_EX_CLIENTEDGE: i32 = 0x00000200;
    const WS_EX_STATICEDGE: i32 = 0x00020000;

    let style = WS_CAPTION
        | WS_THICKFRAME
        | WS_BORDER
        | WS_DLGFRAME
        | WS_SYSMENU
        | WS_MINIMIZEBOX
        | WS_MAXIMIZEBOX
        | 0x00000010; // keep some unrelated bit

    let ex_style =
        WS_EX_DLGMODALFRAME | WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE | 0x00000008;

    let (new_style, new_ex_style) = compute_frameless_window_styles(style, ex_style);

    // Removed bits
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
fn test_compute_frameless_popup_window_styles_sets_ws_popup_and_clears_ws_child() {
    // WinUser.h constants
    const WS_POPUP: i32 = 0x80000000u32 as i32;
    const WS_CHILD: i32 = 0x40000000;
    const WS_CAPTION: i32 = 0x00C00000;

    let style = WS_CHILD | WS_CAPTION | 0x00000010;
    let ex_style = 0x00000008;

    let (new_style, new_ex_style) = compute_frameless_popup_window_styles(style, ex_style);

    assert_ne!(new_style & WS_POPUP, 0);
    assert_eq!(new_style & WS_CHILD, 0);
    assert_eq!(new_style & WS_CAPTION, 0);

    // ex_style is preserved except for the frameless edge removals; we didn't set those bits here.
    assert_eq!(new_ex_style, ex_style);
}
