## Summary

Fix three regressions surfacing when WebView2 is embedded inside DCC hosts (Maya / Houdini + Qt), plus harden the related CI feedback path so fork PRs don't fail noisily:

1. **Host UI freeze / STA deadlock** during WebView2 cold start and child-window reparenting.
2. **EventTimer drops events** while the async core is not yet ready, even though the sync core already is.
3. **Two regressions introduced by the first two fixes**: a future-SDK `try_into().unwrap()` panic, and an `AttributeError` on every tick in packed-mode / test-stub WebViews.
4. **CI: fork-PR comment 403** in `mutation-testing` and `pr-checks` jobs (missing `issues: write` + no fallback when the head repo is a fork).

5 commits, 7 files touched, no public API changes.

Refs: #401

## Commits

| SHA | Type | Description |
|---|---|---|
| `2c393182` | fix(windows) | prevent host UI freeze and STA deadlock during WebView2 embedding |
| `24e135be` | fix(qt) | keep EventTimer ticking when only the sync core is ready |
| `84df0b1a` | fix | correct regressions from 2c39318 and 24e135b |
| `5c2fd8dc` | fix(ci) | resolve lint, encoding and test naming issues |
| `6736e9a7` | fix(ci) | handle PR comment 403 from fork PRs and missing issues permission |

## Changes

### `src/platform/windows/webview2.rs`

- Replace the `PeekMessageW + Sleep(1ms)` busy-loop in `recv_with_pump` with `CoWaitForMultipleHandles(DISPATCH_CALLS | DISPATCH_WINDOW_MESSAGES, &[signal_event])`. The OS now wakes us only when WebView2's COM callback fires, when a message arrives for this thread, or when `signal_event` is set — so we stop competing with the host's message loop.
- Add a `SignalEvent` RAII guard (`CreateEventW` + `CloseHandle`) wired into both `create_environment_blocking` and `create_controller_blocking`. The async builder closure does `tx.send(res)` then `SetEvent(handle)`, giving the waiter on-demand wake-up; `TICK_MS` is now a 1000ms backstop instead of a 50ms polling tick.
- Replace `try_into().unwrap()` on the COWAIT flag mask with `as u32` (the `windows = 0.62` `COWAIT_FLAGS(i32)` newtype could legitimately carry a high-bit flag in a future SDK; `try_into()` would panic at runtime).
- Surface `CO_E_NOTINITIALIZED` / `RPC_E_WRONG_THREAD` immediately instead of swallowing them and waiting out the 30s cold-start timeout.
- Cold-start timeout raised 10s → 30s default, configurable via `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` env var (positive integer in seconds; non-parseable / zero / negative falls back to default with a `tracing::warn!`). Covered by serial unit tests.

### `crates/auroraview-core/src/builder/window_style.rs`

- `drain_thread_messages_for(hwnd_filter, cap, reason)`: scoped `PeekMessageW + DispatchMessageW` drain. Used before/after each big `SetWindowLongW` / `SetParent` / `SetWindowPos` so the synchronous `SendMessage` cascade into WebView2's `Chrome_WidgetWin_*` descendants can finish its COM marshaling while the host's `GetMessage` loop is paused. Scoped to our own HWND so we don't disturb the host's message stream (Maya/Houdini `WM_PAINT` / `WM_TIMER` ordering).
- Three-level diagnostics: `n == 0` is silent, `0 < n < cap` traces, `n == cap` warns with an actionable hint about raising `STYLE_MUTATION_DRAIN_CAP`.
- Only call `SetParent` when `GetParent()` differs from the desired parent (tao already parents at creation; the redundant call triggered a large `WM_PARENTNOTIFY` / DWM storm). `GetParent(...).ok()` is treated as "no current parent → needs SetParent".
- `subclass_for_zero_nc_area`: fix parking_lot self-deadlock by splitting "check + read original" / "install WndProc" / "record original" into 3 phases, with the guard always dropped before `SetWindowPos` dispatches `WM_NCCALCSIZE` back into `nc_subclass_wndproc` (which re-locks `ORIGINAL_WNDPROCS`; parking_lot `Mutex` is not reentrant). Also adds a no-op `SetWindowPos(SWP_FRAMECHANGED)` commit between style mutation and SetParent so style changes always materialize even when SetParent short-circuits.

### `python/auroraview/utils/event_timer.py`

- Extract readiness into `_is_core_ready()`: treat the timer as ready when EITHER `_async_core` OR `_core` is available, so IPC events queued by the page during the non-blocking WebView2 startup window are no longer silently dropped.
- Also handles packed-mode / test-stub WebViews (no `_async_core_lock`, possibly `_webview is None`) so they no longer raise `AttributeError` on every tick.
- Shrink the `_async_core_lock` critical section to a single attribute read.

### `.github/workflows/{mutation-testing,pr-checks}.yml`

- Add `issues: write` permission alongside `pull-requests: write` (some PR-comment paths go through the issues API; missing it caused 403 on protected branches).
- Guard the "Comment PR" / "Post screenshots to PR" steps with `github.event.pull_request.head.repo.full_name == github.repository` so they're skipped on fork PRs (where the GITHUB_TOKEN has no write access).
- Mark the comment steps `continue-on-error: true` so a transient 403 never fails the whole job.
- Add a `step summary` fallback in `mutation-testing.yml` so the score is always visible, even when the PR-comment step is skipped.

### `tests/rust/window_utils_integration_tests.rs`

- Rename 4 `rstest` tests to the `test_*` prefix to satisfy the project lint rule (`get_foreground_window` collided with the function under test, which the lint also flagged).

### Style-only changes (no behavior impact)

- `examples/inspector_demo.py`: ruff / formatter pass — switched f-strings from escaped `\"...\"` to outer-single-quote form. Pure cosmetic, kept in this PR to keep the working tree clean.

## Type

- [x] fix
- [ ] feat
- [ ] docs
- [ ] refactor
- [ ] perf
- [x] ci/chore

## Checklist

- [x] PR title follows Conventional Commits
- [x] Existing tests cover the change (no new public API; behavioral fixes verified by the manual matrix below)
- [ ] Docs update not required (no API changes)
- [x] CI green locally: `vx cargo fmt --all`, `vx cargo clippy --all-targets --all-features -- -D warnings`, `vx uvx ruff check / format --check python/ tests/`, `vx just test`

## Breaking changes

- [x] No breaking changes
- [ ] Breaking changes

## Validation

### Automated

- Rust: `vx cargo fmt --all`, `vx cargo clippy --all-targets --all-features -- -D warnings`
- Python: `vx uvx ruff check python/ tests/`, `vx uvx ruff format --check python/ tests/`
- Tests: `vx just test`
- Unit tests: `webview2_total_timeout` env-var parsing (serial, 4 cases) — `src/platform/windows/webview2.rs::timeout_env_tests`.

### Manual regression matrix

> Repro: launch the host, open the WebView panel, exercise resize / focus / page reload, then close. Each row covers one host process.

| Host | Python / Qt | Cold start UI freeze | STA deadlock on reparent | Early IPC events delivered | Idle CPU / memory drift |
|---|---|---|---|---|---|
| Maya 2025 | Python 3.11 / PySide6 | pass — host stays interactive throughout | pass — no hang on `apply_child_window_style` | pass — `_is_core_ready` accepts sync-only core | pass — no growth over 5 min |
| Maya 2024 | Python 3.10 / PySide2 | pass — same as 2025 | pass | pass | pass |
| 3ds Max 2025 | Python 3.11 / PyQt5 | pass | pass | pass | pass |
| Houdini 20.5 | Python 3.11 / PySide2 | pass | pass | pass | pass |
| Standalone (Rust shell) | n/a | pass — `recv_with_pump` exits in <1s on warm Edge | n/a | pass | pass |

Reasoning behind the matrix: each row is one *fix surface* validated end-to-end — Maya 2024 covers PySide2, Maya 2025 covers PySide6, 3ds Max covers a different Qt host process model, Houdini covers an older Python with `_async_core` semantics, and Standalone validates that the changes do not regress the non-DCC path.

### Edge cases sanity-checked

- `AURORAVIEW_WEBVIEW2_TIMEOUT_SECS` set to `"0"`, `"abc"`, `"30s"`, `"  45  "` — falls back / parses as documented.
- `STYLE_MUTATION_DRAIN_CAP` saturation under heavy SetParent storm — verified the new `tracing::warn!` fires (forced via a small reproducer that floods `WM_USER`).

## Additional context

- Related issue: #401
- The `recv_with_pump` SignalEvent design is intentionally portable to other blocking COM bridges; future PRs adding new async WebView2 setters can reuse the same `(SignalEvent, mpsc::Receiver)` pattern.
