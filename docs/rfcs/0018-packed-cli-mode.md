# RFC 0018: Packed EXE Headless CLI Mode

- Number: 0018
- Title: Headless CLI for packed apps — register commands via Python decorators and invoke them directly without opening a window
- Status: Draft
- Created: 2026-06-15
- Authors: AuroraView Core Team
- Affected code: `crates/auroraview-cli`, `crates/auroraview-pack`, `python/auroraview/core`

---

## 0. Purpose

This RFC proposes adding a **headless CLI mode** to AuroraView packaged artifacts (the standalone exe):

```bash
my-app.exe                          # double-click / no args: open the window as today (unchanged)
my-app.exe -h                       # print help: list every command exposed to the CLI
my-app.exe run export --path ./out  # invoke a registered command, print the result, exit — no window
my-app.exe list                     # list available commands (machine- or human-readable)
```

Command registration and exposure are driven by **Python decorators**, reusing the existing `@webview.command` / `bind_call` machinery and only extending the metadata:

```python
@webview.command(name="export", help="Export the current project to a given format", cli=True)
def export(path: str, format: str = "json") -> dict:
    ...
```

This document covers motivation, feasibility evidence, architecture, the key technical obstacle (the Windows subsystem), the implementation work breakdown, and the trade-offs, for review and confirmation.

---

## 1. Motivation

Today the packaged artifact **can only run as a GUI app**: once `main.rs:127` detects the overlay it calls `run_packed_app()` directly, and that function never reads command-line arguments — it unconditionally creates a window and enters the event loop.

This imposes several limits:

1. **No scripting / automation.** CI, batch jobs, and other programs cannot drive the business logic already inside the app; they can only open the window and operate it by hand.
2. **Duplicated logic.** Developers have already written "export", "validate", "sync" commands in Python and exposed them to the front-end, but a command-line scenario forces them to write a second standalone script.
3. **Limited DCC integration.** Maya/Houdini batch workflows (mayabatch, hython) want to drive app commands directly rather than spin up a full GUI.

Goal: **one exe, one set of decorator-registered commands** — callable both from the front-end inside the window and directly from the command line — without breaking the existing GUI double-click behavior.

---

## 2. Current State and Feasibility Evidence

Investigation conclusion: most of the underlying machinery is already in place; there is no architectural blocker.

### 2.1 The command registry already exists

- The decorator `@webview.command` / `webview.commands.register(name)` adds the function to `CommandRegistry._commands` (`python/auroraview/core/commands.py:131-302`).
- `bind_call` / `bind_api` add the function to `WebViewApiMixin._bound_functions` (`python/auroraview/core/mixins/api.py:37,180-405`).
- At runtime the packed app's `run_api_server` reads from `_bound_functions` and sends a ready signal to Rust carrying the handler-name list (`python/auroraview/core/packed.py:218,236`).

So "register and expose a command via a decorator" is **already done** by the framework — it is just exposed only to the front-end `window.auroraview.call`, never to the command line.

### 2.2 Command invocation does not depend on the window

- `CommandRegistry.invoke(name, **kwargs)` (`commands.py:338`) itself does not depend on a window or an event loop.
- Only `webview.show()` enters the persistent JSON-RPC loop (`packed.py:263-300`). Headless mode just **skips `show()`** and can invoke a command, print, and exit.

### 2.3 The entry_point resolver already supports function-level calls

- `backend.rs:447-454` already supports the `"module:function"` form — it can import a module and call the named function; it also supports `runpy.run_path` for running a script.

### 2.4 Console allocation already has precedent

- In debug mode `allocate_console()` (`packed/mod.rs:140-143` plus the Windows implementation) already demonstrates operating a console from a packed GUI process — a useful starting point for the CLI output work.

**Conclusion**: (a) the command registry exists; (b) it is invoked once at entry_point startup; (c) the backend is a persistent JSON-RPC server, but headless can bypass it; (d) there is no fundamental blocker to invoking a single command, printing the result, and exiting.

The new work concentrates in three places: **entry-point argv routing**, **Windows console output**, and **decorator + pack-time metadata written into the overlay**.

---

## 3. Core Technical Obstacle: the Windows Subsystem (must be solved first)

This is the make-or-break item of this RFC, not a corner-case detail.

### 3.1 The problem

At pack time, to eliminate the double-click black console window, `WinBuilder` rewrites the PE-header subsystem field to **GUI(2)** (`resource_editor.rs`, written directly at PE+68). On Windows a **GUI-subsystem process is not attached to the parent console's stdin/stdout/stderr by default**.

Consequence: running `my-app.exe -h` in `cmd` / PowerShell returns the prompt immediately and the help text has nowhere to go — the CLI mode is effectively dead.

This is the classic "dual-mode exe" GUI/CLI problem (VS Code, Node, and the py launcher all hit it).

### 3.2 Candidate solutions

| Option | Approach | Assessment |
|------|------|------|
| **A. Runtime AttachConsole (recommended)** | Early in startup call `AttachConsole(ATTACH_PARENT_PROCESS)` and redirect stdout/stderr/stdin back to the parent console; if it fails (no parent console, e.g. double-click) skip it | Single file, no black window on double-click, output on the command line. The mainstream industry solution |
| B. Two exes | Pack out `app.exe` (GUI) + `app-cli.exe` (console) | Works but violates the single-file principle and doubles the size |
| C. Pack with console=true | Keep subsystem = console | Double-click pops a black window — unacceptable for most desktop scenarios |

**Option A is selected.** The existing `allocate_console()` uses `AllocConsole` (opens a new window, for debug); we need a new `attach_parent_console()` path:

```rust
// Windows-only, pseudocode
fn attach_parent_console() -> bool {
    unsafe {
        if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
            // redirect CRT/std handles to CONOUT$/CONIN$
            reopen_std_streams();
            true
        } else {
            false // double-click launch, no parent console; CLI output falls back to a log file
        }
    }
}
```

### 3.3 Boundaries

- On double-click launch (no parent console) `AttachConsole` returns 0, and the CLI path should not be triggered anyway (see the §4 trigger convention), so there is no impact.
- When launched from `cmd` but in GUI mode (no CLI arguments), **do not attach** — keep current behavior and avoid bolting a console onto a window app for no reason.
- Output encoding: the Windows console must handle UTF-8 (`SetConsoleOutputCP(CP_UTF8)`, or explicit wide chars), otherwise non-ASCII text is garbled.

---

## 4. Entry-Point argv Routing Design

### 4.1 Current state

`main.rs:125-129`:

```rust
fn main() -> Result<()> {
    if is_packed() {
        return packed::run_packed_app();   // argv ignored entirely
    }
    let cli = Cli::parse();
    ...
}
```

`run_packed_app()` never reads argv and unconditionally opens a window.

### 4.2 The change

Parse argv inside the packed branch and decide GUI vs CLI:

```rust
if is_packed() {
    return match classify_packed_invocation(std::env::args()) {
        PackedInvocation::Gui              => packed::run_packed_app(),
        PackedInvocation::Cli(cli_args)    => packed::run_packed_cli(cli_args),
    };
}
```

### 4.3 Trigger convention (critical — avoid confusion with normal startup)

A packed app may legitimately start with arguments (file associations, dropped file paths, protocol activation). **A bare `--flag` or a bare path must never trigger the CLI**, or double-clicking an associated file would be misread as a command.

We use an **explicit subcommand verb** as the sole trigger:

| Invocation | Verdict | Behavior |
|------|------|------|
| `app.exe` | GUI | open window |
| `app.exe some/file.proj` | GUI | open window with the path as an open argument (current/future file association) |
| `app.exe run <cmd> [--k v ...]` | CLI | invoke command, print, exit |
| `app.exe list [--json]` | CLI | list commands |
| `app.exe -h` / `--help` | CLI | print help |
| `app.exe -V` / `--version` | CLI | print version |

Rule: enter CLI **only when the first argument is a reserved verb (`run`/`list`) or a reserved flag (`-h`/`--help`/`-V`/`--version`)**; everything else is GUI. The reserved-verb set's prefix is configurable in the overlay config to lower the chance of collision with business file names.

### 4.4 Exit-code convention

- Success: `0`
- Command not found: `2`
- Argument error: `2`
- Exception thrown inside the command: `1`, with a structured error on stderr (reusing `CommandError`'s code/message)

---

## 5. Source of the Command List for `-h` / `list`

`-h` and `list` need to know "which commands exist, their help, aliases, and parameters".

### 5.1 Static embedding (the chosen design, zero latency)

**Review decision 3: `-h` must have zero startup latency.** The command list is therefore collected **at pack time** and written into the overlay config; at runtime `-h`/`list` **read the overlay only and never start Python**, returning in milliseconds.

Collection method (to avoid the side effects / missing-dependency risk of `import`ing the user module directly on the packaging machine): the pack flow runs the entry_point once as a subprocess **inside the already-bundled target Python environment** with `AURORAVIEW_CLI_DUMP=1`, letting Python build `webview`/`commands` (**without calling `show()`**) and serialize the command metadata table back to the packager, which writes it into the overlay config's `cli_commands` field (see §13.2 for the structure).

- This subprocess runs **once at pack time**; the runtime is never involved.
- The target environment already has all dependencies, so `import` is safe.
- At runtime `-h`/`list` have zero latency and zero Python startup, satisfying §10 acceptance.

### 5.2 Dynamic (only for the actual `run` execution)

`run <cmd>` does need to start Python anyway, and it `invoke`s in-process following the unified lookup order (see §15.2).

**Conclusion**: `-h`/`list` use static embedding (5.1: collect at pack time → overlay → zero-latency read at runtime); `run` uses dynamic execution (5.2).

---

## 6. Python Decorator Extension

### 6.1 Current state

`@webview.command(name=...)` records only name + callable.

### 6.2 Extension

Add optional CLI metadata (backward compatible; old usage is unaffected). **`cli` carries both the "switch" and the "alias" roles** (review decision 3 — alias and CLI switch are merged):

```python
@webview.command(
    name="export-document-image",
    help="Export the current document as an image",   # used by -h
    cli="exi",                                          # enable CLI + alias exi
    args_help={                                         # per-parameter description (used by -h)
        "path": "Output directory",
        "dpi": "Resolution (DPI)",
    },
)
def export_document_image(path: str, dpi: int = 300) -> dict:
    """Export the current document as an image."""     # help falls back to the docstring's first line
    return {"written": path, "dpi": dpi}
```

The semantics of the `cli` argument's value (a single argument; no separate `aliases=`):

| `cli` value | Meaning |
|-----------|------|
| `False` (default) | Not exposed to the command line; front-end only |
| `True` | Exposed to the command line, no alias |
| `"exi"` (str) | Exposed to the command line, alias `exi` |
| `["exi", "edi"]` (list[str]) | Exposed to the command line, multiple aliases |

- Default `False`, **explicit opt-in**, to avoid accidentally exposing sensitive/dangerous commands to the command line (§9.3, §14).
- `help`: help text; falls back to the docstring's first line.
- `args_help`: per-parameter description used by `-h` (parameter name, type, and default are auto-filled from `inspect.signature`).

### 6.3 Parameter mapping

Support **both `--key value` and positional parameters, mixable** (review decision 5):

```bash
app.exe run export-document-image --path ./out --dpi 600   # keyword
app.exe run exi ./out 600                                  # positional (in signature order)
app.exe run exi ./out --dpi 600                            # mixed: positional first, keyword override
```

Mapping rules:

- **Positional parameters**: filled in order following `inspect.signature`.
- **Keyword parameters**: `--key value` overrides/supplements the matching formal parameter; specifying the same formal parameter both positionally and by keyword → argument error (exit code 2).
- Type conversion follows the annotation: `int`/`float`/`bool`/`str`; complex types go through JSON (`--config '{"a":1}'`).
- Defaults come from the signature's default values; no default and not provided → argument error (exit code 2).
- Booleans: `--flag` means `True`, `--no-flag` means `False`; in positional form a boolean is parsed as `true`/`false`/`1`/`0`.

---

## 7. Headless Execution Flow

The flow of `run_packed_cli` (the CLI path, corrected per §15 to a one-shot in-process call):

```
attach_parent_console() + reopen_std_streams()    # §3.2: attach exists, add reopen of CONOUT$/CONIN$
SetConsoleOutputCP(CP_UTF8)
read_overlay()                                     # reuse existing overlay read
if -h / list:
    read overlay.config.cli_commands -> render -> exit(0)    # zero Python startup (review decision 4)
if run <name|alias>:
    extract_standalone_python()                    # reuse existing extraction (latency on first run)
    alias -> canonical-name normalization (using the alias table in overlay.cli_commands)
    launch entry_point with AURORAVIEW_CLI_INVOKE=<name> + AURORAVIEW_CLI_PARAMS=<json>
        -> Python builds webview/commands (without calling show())
        -> in-process invoke following the §15.2 lookup order
        -> result JSON to stdout / structured error to stderr
        -> exit (per the §4.4 exit code)
```

Reuse points: overlay read, Python extraction, and entry_point process launch (`backend.rs:447`) **already exist**. Headless and the GUI's persistent JSON-RPC server are **two independent paths**; they do not share a process (§9.5).

A new one-shot Python entry that does not call `show()` (§15.2) invokes following the "`CommandRegistry.invoke` → `_bound_functions` fallback" order and serializes the result to stdout.

---

## 8. Development Steps (to fix the implementation process)

> Review has decided **not** to deliver incrementally as P1–P5. The following is an **implementation-level work breakdown**, ordered by dependency; each step notes "files changed / verification". Parallelizable items are flagged.

### Step 1 — Windows console output (hard prerequisite)

- Changes: `crates/auroraview-cli/src/packed/mod.rs`. Add `attach_parent_console()`, reusing the existing `AttachConsole(ATTACH_PARENT_PROCESS)` (`mod.rs:133`), **adding `reopen_std_streams()`** (freopen `CONOUT$`/`CONIN$`, or `SetStdHandle` + CRT fd rebind), and add `SetConsoleOutputCP(CP_UTF8)`. No-op on non-Windows.
- Verify: temporarily print a line of non-ASCII + emoji early in the packed entry, run from cmd / PowerShell to confirm no garbling and no black window on double-click.

### Step 2 — Entry-point argv routing (independent of Python)

- Changes: `crates/auroraview-cli/src/main.rs:127`, `packed/mod.rs`. Add `classify_packed_invocation(args)` and `PackedInvocation::{Gui, Cli}`; route inside `run_packed_app`; the CLI branch calls `run_packed_cli(cli_args)`. Implement the §4.3 trigger convention (only `run`/`list`/`-h`/`--help`/`-V`/`--version` enter CLI).
- Implement the `-V`/`--version` and `-h` skeleton first (`-h` prints a placeholder for now).
- Verify: `app.exe`, `app.exe foo.proj` still open the window; `app.exe -V` prints the version; no GUI regression.

### Step 3 — Decorator `cli` argument (switch + alias merged) + `args_help`

- Changes: `python/auroraview/core/mixins/commands.py`, `python/auroraview/core/commands.py`.
  - `command()` / `register()` gain `cli` (`False`/`True`/`str`/`list[str]`), `help`, `args_help`. Store the metadata on the handler (e.g. `func.__av_cli__`) or in a parallel dict on `CommandRegistry`.
  - Add `CommandRegistry.enable_cli(*names | mapping)` (§14.2).
  - Registration-time conflict detection (§12.4): alias vs canonical name / alias / reserved verb.
- Verify: Python unit tests covering the four `cli` values, alias-conflict errors, `args_help` reading, docstring fallback.

### Step 4 — Command metadata collection (unify the two registries)

- Changes: `python/auroraview/core/packed.py`. Add `iter_cli_commands(webview)` (walk `CommandRegistry._commands` + `_bound_functions`, take `cli != False`, normalize aliases), `collect_cli_commands()`, and `dump_cli_metadata()` (take this path when `AURORAVIEW_CLI_DUMP=1`, serialize the §13.2 structure then exit).
- The entry must detect `AURORAVIEW_CLI_DUMP` before `show()` and short-circuit (do not enter GUI/server).
- Verify: locally set `AURORAVIEW_CLI_DUMP=1` and run the demo app's entry_point, confirm stdout is the expected JSON and no window opens.

### Step 5 — Pack-time overlay embedding (zero runtime latency)

- Changes:
  - `crates/auroraview-pack/src/config.rs`: `PackConfig` gains `cli_commands: Vec<CliCommandMeta>` (default empty); define `CliCommandMeta`/`CliParamMeta` (serde).
  - Pack flow (FullStack/Python branch): once the bundled Python environment is ready, run the entry_point as an `AURORAVIEW_CLI_DUMP=1` subprocess, parse the stdout JSON, do §12.4 conflict detection (pack fails on conflict), write `cli_commands`.
- Verify: after `app pack`, dump the overlay via the `inspect` subcommand (`main.rs:115`) and confirm it contains `cli_commands`.

### Step 6 — `-h` / `list` rendering (overlay read only)

- Changes: `packed/mod.rs` or a new `packed/cli.rs`. In `run_packed_cli`, render `-h`/`list`/`list --json` from `overlay.config.cli_commands` (§13.4). **Do not start Python.**
- Verify: `app.exe -h`, `app.exe list`, `app.exe list --json` output correctly, include aliases and parameters, zero latency; redirecting to a file is consistent.

### Step 7 — `run` headless execution (in-process invoke)

- Changes:
  - Rust (`packed/cli.rs`): alias normalization → extract Python → launch entry_point with `AURORAVIEW_CLI_INVOKE` + `AURORAVIEW_CLI_PARAMS` → pass through stdout/stderr → map exit codes (§4.4).
  - Python (`packed.py`): the entry detects `AURORAVIEW_CLI_INVOKE`, invokes following the §15.2 order, serializes the result/error then exits, **without calling `show()`**.
  - Parameter parsing (§6.3): positional + `--key value` mixing, type conversion, booleans, required-field validation. Positional parsing can live in Rust or Python; **the Python side** is recommended (it holds `inspect.signature`).
- Verify: `run <name> --k v`, `run <alias> <pos>`, mixing, missing required (exit code 2), command not found (2), command throws (1).

### Step 8 — Wrap-up

- Structured errors (reuse `CommandError` code/message), end-to-end exit codes, timeouts;
- Docs and examples (README / `docs/`);
- Cross-platform verification: attach is a no-op on macOS/Linux, but `-h`/`run` stdout behavior is verified on each.

### Dependencies

```
Step1 ─┐
Step2 ─┼─→ Step6 (-h/list render) ─┐
Step3 ─→ Step4 ─→ Step5 ───────────┴─→ Step7 (run) ─→ Step8
```

- Steps 1, 2, 3 can start in parallel (mutually independent).
- Step 6 depends on 2 (routing) + 5 (overlay has data).
- Step 7 depends on 5 (alias table) + 4 (Python invoke entry).

---

## 9. Trade-offs and Risks

1. **Robustness of the trigger convention**: explicit verbs (`run`/`list`) + opt-in `cli` (default `False`) provide double insurance, avoiding file-association/dropped-path misreads as commands. Conflicts between reserved verbs and business command names/aliases are detected and errored at pack-time collection (§12.4).
2. **First-run latency**: under the standalone strategy `run`'s first launch must extract Python (a few seconds); mitigated by content_hash cache reuse on a hit, and the docs must explain this. `-h`/`list` use pack-time static embedding (§13.3) — zero startup latency, unaffected.
3. **Security boundary**: CLI-exposed commands are off by default (`cli=False`); developers must explicitly opt in. Commands that write/delete files or touch the network should note in the docs that auth and confirmation responsibility lies with the integrator.
4. **Cross-platform consistency**: this RFC focuses on Windows (the subsystem problem is most acute there). macOS/Linux .app/AppImage have no GUI-subsystem isolation problem; the attach logic is a cross-platform no-op fallback, but stdout behavior must be verified on each.
5. **Relationship to the persistent backend**: headless is "invoke once and exit", a separate path from the GUI's persistent JSON-RPC server; they do not share a process, avoiding state pollution.

---

## 10. Acceptance Criteria

- `app.exe` (double-click / no args) behaves exactly as today — no black window, no regression.
- `app.exe -h` correctly prints the command list (with help, aliases, parameters) under cmd / PowerShell, **reading the overlay only, zero Python startup**, UTF-8 without garbling.
- `app.exe run <cmd|alias> --k v` and `app.exe run <cmd|alias> <pos...>` both correctly invoke the command, output the result JSON to stdout, with exit codes per §4.4.
- Commands with CLI disabled (`cli=False`) do not appear in `list`/`-h`, and a `run` invocation reports "command not found".
- Output is consistent across cmd / PowerShell / redirect-to-file (`app.exe list > out.txt`).

---

## 11. Review Decisions (confirmed)

| # | Topic | Decision |
|---|------|------|
| 1 | Trigger verb | Use `run` / `list` (keep the §4.3 trigger convention) |
| 2 | `cli` default | Off by default (`cli=False`), explicit opt-in |
| 3 | Alias and switch | **Merged into a single `cli` argument**: `cli=True` (on, no alias) / `cli="exi"` (on + alias) / `cli=["exi","edi"]` (multiple aliases). No separate `aliases=` |
| 4 | `-h` latency | **Must be zero latency**: the command list is collected at pack time and embedded into the overlay; the runtime reads it directly, never starting Python |
| 5 | Positional parameters | **Supported**, mixable with `--key value` (§6.3) |

---

## 12. Add-on Feature A: Command Aliases (merged into the `cli` argument)

### 12.1 Requirement

Allow declaring a short alias for a long command name, e.g. `export-document-image` can be invoked as `exi`.

### 12.2 Design: alias and CLI switch merged into a single `cli` argument (review decision 3)

No separate `aliases=` is introduced; the alias is written directly on `cli` (value semantics in §6.2):

```python
@webview.command(name="export-document-image", cli="exi")          # alias exi
def export_document_image(path: str, dpi: int = 300) -> dict: ...

@webview.command(name="validate", cli=["val", "v"])                # multiple aliases
def validate(...): ...

@webview.command(name="sync", cli=True)                            # enabled but no alias
def sync(...): ...
```

- Alias and canonical name share the same handler and same metadata, shown in `list`/`-h` as `export-document-image (exi)`.
- The front-end JS side still uses only the canonical name (`window.auroraview.invoke("export-document-image", ...)`); aliases apply to the command line only, to avoid namespace bloat.

### 12.3 Where aliases are resolved (to avoid clashing with the trigger convention)

§4.3 states CLI is entered only when the first token is a reserved verb `run`/`list` or a reserved flag `-h`/`-V`. Therefore **the alias is resolved as an argument to `run`**, not as a top-level bare token:

```bash
app.exe run export-document-image --path ./out   # canonical name
app.exe run exi --path ./out                      # alias, equivalent
app.exe run exi ./out                             # alias + positional
```

We do not make `-exi` (a bare short flag with a leading `-`) a top-level trigger: it conflicts with §4.3 ("a bare `--flag`/bare path is always GUI" — file association/dropped paths would be misread), and a leading `-` would collide with clap's top-level flag parsing. **The alias form is fixed as a short word with no leading `-`, resolved uniformly under `run <name|alias>`.**

### 12.4 Conflict detection

Validated during pack-time collection (§5.1); a duplicate is an immediate error (fail-fast):

- An alias duplicates any canonical name;
- An alias duplicates another alias;
- An alias hits the reserved-verb set (`run`/`list`/`help`/`version`, etc.).

---

## 13. Add-on Feature B: `-h` Auto-Collects the Command List (with descriptions and parameters)

### 13.1 Requirement

`app.exe -h` automatically lists **all CLI-enabled (`cli != False`)** commands and shows: command name, aliases, command description, and each parameter (name, type, default, whether required, parameter description). **Zero Python startup** (review decision 4).

### 13.2 Metadata structure

The metadata collected for each command (written into the overlay `cli_commands` field):

```jsonc
{
  "name": "export-document-image",
  "aliases": ["exi"],               // from cli="exi" / cli=["exi",...]
  "help": "Export the current document as an image",   // command(help=); falls back to docstring first line
  "params": [
    {"name": "path", "type": "str", "required": true,  "default": null, "help": "Output directory"},
    {"name": "dpi",  "type": "int", "required": false, "default": 300,  "help": "Resolution (DPI)"}
  ]
}
```

- `aliases`: `cli` is str → single element; list → multiple elements; `True` → empty.
- `help`: `command(help=)`, falling back to the docstring's first line.
- `params`: `inspect.signature` auto-fills `type` (annotation name, `"any"` if unannotated), `required` (required if no default), and `default`; `help` comes from `args_help`.

### 13.3 Collection mechanism: pack-time static embedding (zero runtime latency)

**Do not dump at runtime.** The command list is collected once in the pack flow and written into the overlay; at runtime `-h`/`list` read the overlay only:

```
Pack time (inside the pack flow, target bundled Python environment is ready):
    run the entry_point as an AURORAVIEW_CLI_DUMP=1 subprocess
        -> Python builds webview/commands (without calling show())
        -> collect_cli_commands(webview) serializes the command metadata table -> stdout -> exit(0)
    the packager receives the JSON, does §12.4 conflict detection, writes PackConfig.cli_commands

Runtime (every app.exe -h / list):
    read overlay.config.cli_commands -> render -> exit(0)    # zero Python startup, milliseconds
```

- Collection happens only at pack time; the target environment already has the dependencies, so `import` is safe.
- The runtime never extracts/starts Python, satisfying the §10 zero-latency acceptance.

A new lightweight Python entry (not entering the persistent loop):

```python
# new in packed.py
def dump_cli_metadata(webview: "WebView") -> None:
    table = collect_cli_commands(webview)   # only cli != False commands + parameter introspection
    _get_writer().write_json({"type": "cli_metadata", "commands": table})
```

`collect_cli_commands` must walk **both** `CommandRegistry._commands` and `_bound_functions` (see §15) and take their `cli != False` subset.

### 13.4 Rendering

Human-readable (`-h`) and machine-readable (`list --json` emits the §13.2 JSON array directly) share the same metadata. `-h` example:

```
USAGE:
    app.exe run <command> [--key value ...]

COMMANDS:
    export-document-image (exi)   Export the current document as an image
        --path <str>              [required] Output directory
        --dpi  <int>              [default 300] Resolution (DPI)
```

---

## 14. Add-on Feature C: CLI Off by Default, Explicit Opt-In Required

### 14.1 Requirement

Registered commands **do not enable** CLI support by default; CLI support must be added manually before a command can be invoked from the command line.

### 14.2 Design (consistent with §6.2, formalized here)

- The `cli` argument of `@webview.command` defaults to `False` (review decision 2).
- Only `cli != False` commands enter the §13 list and can be invoked by `run` (canonical/alias); the rest return "command not found" (exit code 2) on a `run` invocation and do not appear in `list`/`-h`.
- Two opt-in granularities are provided:

```python
# Granularity 1: enable a single command (can also give an alias)
@webview.command(name="export", cli=True)        # enabled, no alias
@webview.command(name="export", cli="exp")       # enabled + alias

# Granularity 2: bulk enable (registry level, convenient for projects with many existing commands)
webview.commands.enable_cli("export", "validate")               # enable only
webview.commands.enable_cli({"export": "exp", "validate": "v"}) # enable + alias
```

- **No** "enable everything by default" global switch is provided, to avoid accidentally exposing file-write/delete/network commands to the command line (a secure default, echoing §9.3).

### 14.3 Default behavior

A command without a declared `cli` (i.e. `cli=False`) is entirely invisible to the command line and cannot be invoked by `run` — this is the default and only secure state. Developers must explicitly declare `cli` for each command they want to expose.

---

## 15. Defect Fix: Bridge the Two Command Registries (added in review)

### 15.1 The problem

The code has two independent registries:

- `@webview.command` / `commands.register` → `CommandRegistry._commands` (`commands.py:281`), exposed via the `__invoke__` IPC callback (`commands.py:159`);
- `bind_call` / `bind_api` → `_bound_functions` (`api.py:243`), exposed via the JSON-RPC `_handle_request` (`packed.py:345`).

`run_api_server` reads only `_bound_functions` (`packed.py:218`), and `_handle_request` looks up only `bound_functions.get(method)` (`packed.py:345`). So if §7 reused the JSON-RPC server, commands registered by `@webview.command` would be **entirely unreachable**, contradicting the §1/§6 examples.

### 15.2 The fix

Headless `run` **does not reuse the persistent JSON-RPC server loop**; it is a one-shot in-process call that resolves the command following a unified lookup order:

```
1. CommandRegistry.invoke(name, **params)   # commands registered by @webview.command (§2.2 confirmed window-independent)
2. fall back to _bound_functions[name](**params)   # commands registered by bind_call/bind_api
3. neither -> exit code 2 "command not found"
```

The §7 flow is correspondingly corrected to:

```
attach_parent_console() + reopen_std_streams()   # §3.2 fix: attach exists, add reopen
read_overlay()
if -h / list:
    read the §13 cache (dump once if needed) -> render -> exit(0)
if run <name|alias>:
    extract_standalone_python()
    launch entry_point with AURORAVIEW_CLI_INVOKE=<name> + params(JSON)
        -> Python builds webview/commands, [no show()]
        -> invoke following the §15.2 lookup order, result JSON to stdout / error to stderr
        -> exit (per the corresponding exit code)
```

Aliases (§12) are normalized alias→canonical before the step 1/2 lookup (the normalization table likewise comes from pack-time-collected metadata, read alongside the overlay at runtime).

### 15.3 Consistency with metadata collection

`collect_cli_commands` (§13.3) and headless `run` (§15.2) must share the same "walk both registries + alias normalization" logic, so the commands listed by `-h` match the commands `run` can actually invoke. Extracting a single `iter_cli_commands(webview)` generator for both is recommended.
