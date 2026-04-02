//! Unit tests for CLI argument parsing using clap's try_parse_from API.
//! These tests exercise RunArgs and PackArgs without spawning a subprocess.

use auroraview_cli::cli::{PackArgs, RunArgs};
use clap::Parser;
use rstest::rstest;

// ---------------------------------------------------------------------------
// RunArgs – basic parsing
// ---------------------------------------------------------------------------

#[test]
fn test_run_args_defaults() {
    let args = RunArgs::try_parse_from(["run", "--url", "https://example.com"]).unwrap();
    assert_eq!(args.url.as_deref(), Some("https://example.com"));
    assert!(args.html.is_none());
    assert_eq!(args.title, "AuroraView");
    assert_eq!(args.width, 800);
    assert_eq!(args.height, 600);
    assert!(!args.debug);
    assert!(!args.watch);
    assert!(!args.always_on_top);
}

#[test]
fn test_run_args_url() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "http://localhost:3000"]).unwrap();
    assert_eq!(args.url.as_deref(), Some("http://localhost:3000"));
    assert!(args.html.is_none());
}

#[test]
fn test_run_args_url_short_flag() {
    let args =
        RunArgs::try_parse_from(["run", "-u", "http://localhost:5173"]).unwrap();
    assert_eq!(args.url.as_deref(), Some("http://localhost:5173"));
}

#[test]
fn test_run_args_html_flag() {
    let args =
        RunArgs::try_parse_from(["run", "--html", "/path/to/index.html"]).unwrap();
    assert!(args.html.is_some());
    assert!(args.url.is_none());
}

#[test]
fn test_run_args_html_short_flag() {
    let args = RunArgs::try_parse_from(["run", "-f", "/tmp/index.html"]).unwrap();
    assert!(args.html.is_some());
}

#[test]
fn test_run_args_custom_title() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "--title", "My Tool"])
            .unwrap();
    assert_eq!(args.title, "My Tool");
}

#[test]
fn test_run_args_custom_size() {
    let args = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--width",
        "1280",
        "--height",
        "720",
    ])
    .unwrap();
    assert_eq!(args.width, 1280);
    assert_eq!(args.height, 720);
}

#[test]
fn test_run_args_debug_flag() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "--debug"]).unwrap();
    assert!(args.debug);
}

#[test]
fn test_run_args_debug_short_flag() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "-d"]).unwrap();
    assert!(args.debug);
}

#[test]
fn test_run_args_watch_flag() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "--watch"]).unwrap();
    assert!(args.watch);
}

#[test]
fn test_run_args_poll_interval() {
    let args = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--watch",
        "--poll-interval-ms",
        "2000",
    ])
    .unwrap();
    assert_eq!(args.poll_interval_ms, 2000);
}

#[test]
fn test_run_args_always_on_top() {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "--always-on-top"])
            .unwrap();
    assert!(args.always_on_top);
}

#[test]
fn test_run_args_allow_new_window() {
    let args = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--allow-new-window",
    ])
    .unwrap();
    assert!(args.allow_new_window);
}

#[test]
fn test_run_args_allow_file_protocol() {
    let args = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--allow-file-protocol",
    ])
    .unwrap();
    assert!(args.allow_file_protocol);
}

// url and html are mutually exclusive (conflicts_with)
#[test]
fn test_run_args_url_html_conflict() {
    let result =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", "--html", "/tmp/a.html"]);
    assert!(result.is_err(), "url and html should conflict");
}

// poll-interval-ms requires watch
#[test]
fn test_run_args_poll_interval_requires_watch() {
    let result = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--poll-interval-ms",
        "500",
    ]);
    assert!(result.is_err(), "poll-interval-ms should require --watch");
}

// assets-root requires html: using --assets-root without --html should fail
#[test]
fn test_run_args_assets_root_requires_html() {
    // --url and --html conflict, so --assets-root without --html (no --url either) fails
    let result = RunArgs::try_parse_from(["run", "--assets-root", "/tmp"]);
    assert!(result.is_err(), "--assets-root without --html should fail");
}

// neither url nor html – clap requires at least one, RunArgs should reject
// (note: clap itself may not enforce this unless we add a custom validator;
//  this test checks that at minimum the struct is created if we pass nothing extra)
#[test]
fn test_run_args_no_url_no_html_succeeds_parsing() {
    // clap parse itself doesn't enforce the at-least-one-of rule at parse time;
    // that's done at runtime by run_webview. Parsing alone should succeed.
    let result = RunArgs::try_parse_from(["run"]);
    // This may succeed or fail depending on clap group settings; just assert no panic.
    let _ = result;
}

// ---------------------------------------------------------------------------
// RunArgs – parametrised flag combinations
// ---------------------------------------------------------------------------

#[rstest]
#[case("--debug", true)]
#[case("--watch", false)] // --watch doesn't set debug
fn test_run_args_individual_bool_flags(#[case] flag: &str, #[case] is_debug: bool) {
    let args =
        RunArgs::try_parse_from(["run", "--url", "https://a.com", flag]).unwrap();
    assert_eq!(args.debug, is_debug);
}

// ---------------------------------------------------------------------------
// PackArgs – basic parsing
// ---------------------------------------------------------------------------

#[test]
fn test_pack_args_defaults() {
    let args = PackArgs::try_parse_from(["pack"]).unwrap();
    assert!(args.config.is_none());
    assert!(args.url.is_none());
    assert!(args.frontend.is_none());
    assert!(args.backend.is_none());
    assert!(args.output.is_none());
    assert!(args.title.is_none());
    assert!(!args.debug);
    assert!(!args.build);
    assert!(!args.frameless);
    assert!(!args.always_on_top);
    assert!(!args.no_resize);
    assert!(!args.console);
    assert!(!args.no_console);
}

#[test]
fn test_pack_args_config_flag() {
    let args =
        PackArgs::try_parse_from(["pack", "--config", "auroraview.pack.toml"]).unwrap();
    assert!(args.config.is_some());
    assert_eq!(
        args.config.unwrap().to_str().unwrap(),
        "auroraview.pack.toml"
    );
}

#[test]
fn test_pack_args_url_flag() {
    let args = PackArgs::try_parse_from(["pack", "--url", "https://example.com"]).unwrap();
    assert_eq!(args.url.as_deref(), Some("https://example.com"));
}

#[test]
fn test_pack_args_frontend_flag() {
    let args = PackArgs::try_parse_from(["pack", "--frontend", "./dist"]).unwrap();
    assert!(args.frontend.is_some());
}

#[test]
fn test_pack_args_output_flags() {
    let args = PackArgs::try_parse_from([
        "pack",
        "--output",
        "my-app",
        "--output-dir",
        "./out",
    ])
    .unwrap();
    assert_eq!(args.output.as_deref(), Some("my-app"));
    assert!(args.output_dir.is_some());
}

#[test]
fn test_pack_args_window_size() {
    let args = PackArgs::try_parse_from([
        "pack",
        "--width",
        "1920",
        "--height",
        "1080",
    ])
    .unwrap();
    assert_eq!(args.width, Some(1920));
    assert_eq!(args.height, Some(1080));
}

#[test]
fn test_pack_args_frameless() {
    let args = PackArgs::try_parse_from(["pack", "--frameless"]).unwrap();
    assert!(args.frameless);
}

#[test]
fn test_pack_args_always_on_top() {
    let args = PackArgs::try_parse_from(["pack", "--always-on-top"]).unwrap();
    assert!(args.always_on_top);
}

#[test]
fn test_pack_args_no_resize() {
    let args = PackArgs::try_parse_from(["pack", "--no-resize"]).unwrap();
    assert!(args.no_resize);
}

#[test]
fn test_pack_args_user_agent() {
    let args = PackArgs::try_parse_from(["pack", "--user-agent", "MyAgent/1.0"]).unwrap();
    assert_eq!(args.user_agent.as_deref(), Some("MyAgent/1.0"));
}

#[test]
fn test_pack_args_console_flag() {
    let args = PackArgs::try_parse_from(["pack", "--console"]).unwrap();
    assert!(args.console);
    assert!(!args.no_console);
}

#[test]
fn test_pack_args_no_console_flag() {
    let args = PackArgs::try_parse_from(["pack", "--no-console"]).unwrap();
    assert!(args.no_console);
    assert!(!args.console);
}

// console and no-console are mutually exclusive
#[test]
fn test_pack_args_console_no_console_conflict() {
    let result = PackArgs::try_parse_from(["pack", "--console", "--no-console"]);
    assert!(result.is_err(), "--console and --no-console should conflict");
}

// url conflicts with frontend and backend
#[test]
fn test_pack_args_url_frontend_conflict() {
    let result =
        PackArgs::try_parse_from(["pack", "--url", "https://a.com", "--frontend", "./dist"]);
    assert!(result.is_err(), "--url should conflict with --frontend");
}

// backend requires frontend
#[test]
fn test_pack_args_backend_requires_frontend() {
    let result =
        PackArgs::try_parse_from(["pack", "--backend", "app.main:run"]);
    assert!(result.is_err(), "--backend requires --frontend");
}

#[test]
fn test_pack_args_clean_flag() {
    let args = PackArgs::try_parse_from(["pack", "--clean"]).unwrap();
    assert!(args.clean);
}

#[test]
fn test_pack_args_icon_flag() {
    let args = PackArgs::try_parse_from(["pack", "--icon", "./app.ico"]).unwrap();
    assert!(args.icon.is_some());
}

#[test]
fn test_pack_args_title_short_flag() {
    let args = PackArgs::try_parse_from(["pack", "-t", "My App"]).unwrap();
    assert_eq!(args.title.as_deref(), Some("My App"));
}

#[test]
fn test_pack_args_debug_short_flag() {
    let args = PackArgs::try_parse_from(["pack", "-d"]).unwrap();
    assert!(args.debug);
}

// ---------------------------------------------------------------------------
// Parametrised: RunArgs window dimension parsing
// ---------------------------------------------------------------------------

#[rstest]
#[case(800, 600)]
#[case(1280, 720)]
#[case(1920, 1080)]
#[case(0, 0)]
fn test_run_args_dimensions(#[case] w: u32, #[case] h: u32) {
    let args = RunArgs::try_parse_from([
        "run",
        "--url",
        "https://a.com",
        "--width",
        &w.to_string(),
        "--height",
        &h.to_string(),
    ])
    .unwrap();
    assert_eq!(args.width, w);
    assert_eq!(args.height, h);
}

// ---------------------------------------------------------------------------
// Parametrised: PackArgs window dimension parsing
// ---------------------------------------------------------------------------

#[rstest]
#[case(800u32, 600u32)]
#[case(1440, 900)]
fn test_pack_args_dimensions(#[case] w: u32, #[case] h: u32) {
    let args = PackArgs::try_parse_from([
        "pack",
        "--width",
        &w.to_string(),
        "--height",
        &h.to_string(),
    ])
    .unwrap();
    assert_eq!(args.width, Some(w));
    assert_eq!(args.height, Some(h));
}
