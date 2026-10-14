//! Console attachment for the packed headless CLI mode (RFC 0018 §3).
//!
//! Packed executables have their PE subsystem rewritten to GUI (see
//! `auroraview-pack` `resource_editor::set_subsystem`) so a double-click never
//! flashes a console window. The downside is that a GUI-subsystem process does
//! **not** inherit the parent shell's stdin/stdout/stderr, so anything printed
//! from `app.exe -h` / `app.exe list` would otherwise vanish.
//!
//! [`attach_parent_console`] reconnects the standard streams to the launching
//! console (when there is one) and switches the output code page to UTF-8 so
//! that Chinese text and emoji render correctly. It is only meant to run on the
//! CLI path: double-click launches have no parent console, `AttachConsole`
//! returns an error, and we leave the GUI path untouched (§3.3).
//!
//! On non-Windows targets the whole thing is a no-op — `.app` / AppImage
//! bundles don't have the GUI-subsystem stdio isolation problem (§9.4).

/// Attach the current process to its parent console and route the standard
/// streams there, using UTF-8 for output.
///
/// Returns `true` when a parent console was attached and the streams were
/// reopened (i.e. we were launched from a terminal), `false` otherwise (e.g.
/// double-click, where there is no parent console). On non-Windows platforms
/// this always returns `false`.
#[cfg(target_os = "windows")]
pub fn attach_parent_console() -> bool {
    use windows::Win32::System::Console::{
        AttachConsole, SetConsoleOutputCP, ATTACH_PARENT_PROCESS,
    };

    // SAFETY: All calls are plain Win32 FFI with no shared-state invariants.
    // `AttachConsole` is idempotent-safe to probe; on failure we bail out
    // before touching any std handles.
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
            // No parent console (double-click / detached launch). Leave stdio
            // as-is so the GUI path is unaffected (§3.3).
            return false;
        }

        reopen_std_streams();

        // CP_UTF8 = 65001. Without this the console mangles multi-byte text.
        let _ = SetConsoleOutputCP(65001);
    }

    true
}

/// Re-point the process standard handles at the freshly attached console.
///
/// After `AttachConsole`, the `STD_*_HANDLE` slots may still reference the
/// (invalid) handles inherited at startup. We open the console's own
/// `CONOUT$` / `CONIN$` pseudo-files and install them via `SetStdHandle`.
/// Rust's `std::io::{stdout, stderr, stdin}` re-query `GetStdHandle` on each
/// access, so this is sufficient for `println!` / `eprintln!` to land in the
/// parent console.
#[cfg(target_os = "windows")]
unsafe fn reopen_std_streams() {
    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{GENERIC_READ, GENERIC_WRITE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows::Win32::System::Console::{
        SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
    };

    // UTF-16, NUL-terminated names for the console pseudo-files.
    let conout: Vec<u16> = "CONOUT$\0".encode_utf16().collect();
    let conin: Vec<u16> = "CONIN$\0".encode_utf16().collect();

    // Output side: CONOUT$ feeds both stdout and stderr.
    if let Ok(out) = CreateFileW(
        PCWSTR(conout.as_ptr()),
        (GENERIC_READ | GENERIC_WRITE).0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        Default::default(),
        None,
    ) {
        let _ = SetStdHandle(STD_OUTPUT_HANDLE, out);
        let _ = SetStdHandle(STD_ERROR_HANDLE, out);
    }

    // Input side: CONIN$ feeds stdin.
    if let Ok(inp) = CreateFileW(
        PCWSTR(conin.as_ptr()),
        (GENERIC_READ | GENERIC_WRITE).0,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        None,
        OPEN_EXISTING,
        Default::default(),
        None,
    ) {
        let _ = SetStdHandle(STD_INPUT_HANDLE, inp);
    }
}

/// Non-Windows stub: there is no GUI-subsystem stdio isolation to undo, so the
/// standard streams already work. Always reports "no console attached".
#[cfg(not(target_os = "windows"))]
pub fn attach_parent_console() -> bool {
    false
}
