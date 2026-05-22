# RFC 0014: Centralize `wry` / `tao` Into `[workspace.dependencies]`

- Number: 0014
- Title: Centralize `wry` / `tao` versions via `[workspace.dependencies]`
- Status: Draft
- Created: 2026-05-21
- Authors: AuroraView Core Team
- Split from: RFC 0013 §4.1.5 D9 revision
- Affected files:
  - `Cargo.toml` (workspace root, adds `[workspace.dependencies]`)
  - `crates/auroraview-core/Cargo.toml`
  - `crates/auroraview-desktop/Cargo.toml`
  - `crates/auroraview-browser/Cargo.toml`
  - `crates/auroraview-cli/Cargo.toml`

---

## 1. Summary

Move the `wry` / `tao` version pins from each crate's `[dependencies]` into the workspace root `Cargo.toml`'s `[workspace.dependencies]`. All affected crates switch to `{ workspace = true }` references, eliminating the version-drift risk.

We do **not** add `pub use wry`, to avoid pinning the upstream crate's types into `auroraview-core`'s public API surface.

---

## 2. Motivation

The repo root `Cargo.toml` currently writes:

```toml
[dependencies]
wry = "0.54.4"
tao = "0.34.6"
```

Each crate (`auroraview-core` / `auroraview-desktop` / `auroraview-browser` / `auroraview-cli` / root crate) writes its own version pin independently, with the following risks:

1. **Version drift**: missing one crate when bumping wry causes two wry versions to be pulled in at compile time, leading to symbol-link conflicts or undefined runtime behavior.
2. **Cross-crate type incompatibility**: a `wry::WebViewBuilder` exposed by `auroraview-core` and a `wry::WebViewBuilder` referenced in `auroraview-cli` come from different versions, so calling `auroraview_core::attach_drag_drop_handler` produces a compile-time type mismatch.
3. **CI noise**: every wry bump PR has to touch 5 `Cargo.toml` files, bloating PR descriptions.

The `attach_drag_drop_handler` helper introduced by RFC 0015 exposes `wry::WebViewBuilder<'a>` in its signature, which requires all callers to use the same wry version. This RFC is a hard prerequisite for 0015.

---

## 3. Design

### 3.1 Workspace Root `Cargo.toml`

Add a `[workspace.dependencies]` section:

```toml
[workspace.dependencies]
wry = "0.54.4"
tao = "0.34.6"
# Other shared webview-related deps can be folded in here as needed.
```

The root `Cargo.toml`'s own `[dependencies]` section changes:

```toml
wry = "0.54.4"
tao = "0.34.6"
```

into:

```toml
wry = { workspace = true }
tao = { workspace = true }
```

### 3.2 Per-crate `Cargo.toml`

`auroraview-core` / `auroraview-desktop` / `auroraview-browser` / `auroraview-cli` — each crate's `Cargo.toml` switches its `wry` / `tao` references to `{ workspace = true }`. 1–2 lines per crate, no source changes.

### 3.3 No `pub use wry`

`auroraview-core`'s public API (such as the future RFC 0015 `DragDropIpcSink` trait and `attach_drag_drop_handler` function) **will** mention `wry::WebViewBuilder` in its signatures — that is unavoidable, because the helper's job is to operate on the wry builder.

But this is distinct from "`pub use wry` to re-export the entire wry module":

- **Current approach**: `auroraview-core` **references** wry types in signatures (unavoidable) but does **not** actively re-export them. Downstream crates use wry directly via the workspace dep. When wry releases a minor/patch bump, the workspace dep is bumped in one place and the entire workspace recompiles cleanly.
- **If wry ships a breaking release** (e.g. `0.54 → 0.55` changing `WebViewBuilder` signatures), `auroraview-core`'s helper signatures will be affected too — but a single workspace-dep bump synchronizes the entire workspace, equivalent in effect to `pub use wry` while avoiding the design commitment that "`auroraview-core` actively exposes an upstream crate". This leaves more naming room in the future to swap wry for another webview crate.

---

## 4. Implementation Steps

1. **Step 1**: add the `[workspace.dependencies]` section to the root `Cargo.toml`, populated with `wry` / `tao` versions.
2. **Step 2**: change the root `Cargo.toml`'s own `[dependencies]` and the four crate `Cargo.toml` files to `{ workspace = true }`.
3. **Step 3**: `vx just build` to verify compilation; `vx just test` to verify no regressions.

Total change is ~10 lines and fits in a standalone PR.

---

## 5. Compatibility

- **Fully non-breaking**: only `Cargo.toml` files change; no public API or runtime behavior is affected.
- **Backward compatible**: existing caller code requires no adjustment after the bump.

---

## 6. Risks

| Risk | Assessment | Mitigation |
|---|---|---|
| Bumping wry still requires editing root `[workspace.dependencies]` | Low | Editing one place is less error-prone than editing five; CI compile failure catches regressions immediately |
| wry adds a new generic parameter (e.g. `WebViewBuilder<'a, T>`) | Low | The workspace dep does not eliminate the helper-signature sync cost, but ensures one bump propagates everywhere |

---

## 7. Follow-up Dependencies

- **RFC 0015** exposes `attach_drag_drop_handler` from `auroraview-core` (signature includes `wry::WebViewBuilder<'a>`); this RFC must land first to ensure cross-crate compile-time type consistency.
