# -*- coding: utf-8 -*-
"""Packed Headless CLI Demo - AuroraView (RFC 0018).

This example demonstrates AuroraView's *packed headless CLI* mode: the same
packed ``app.exe`` opens its GUI window when double-clicked, yet can also run
registered commands straight from a terminal - no window, JSON to stdout, exit.

Commands are exposed with ``@webview.command(cli=...)``. The ``cli`` argument is
both the CLI on/off switch and the alias declaration (RFC 0018 §6.2):

    cli=False           # default: front-end only, hidden from the CLI
    cli=True            # CLI-enabled, no alias
    cli="imp"           # CLI-enabled, single alias
    cli=["wt", "txt"]   # CLI-enabled, multiple aliases

Run as a normal script (opens a window):
    python examples/packed_cli_demo.py

Pack it into a standalone exe:
    av pack examples/pack/python/packed-cli-demo.pack.toml
    # or: python -m auroraview.cli pack examples/pack/python/packed-cli-demo.pack.toml

Then call it from the command line (no window opens):
    # List CLI-enabled commands (reads the embedded table, zero Python startup)
    packed-cli-demo.exe -h
    packed-cli-demo.exe list
    packed-cli-demo.exe list --json

    # 2.1 Import an image (returns success message)
    packed-cli-demo.exe run import-image --path C:/pics/logo.png
    packed-cli-demo.exe run imp C:/pics/logo.png            # via alias + positional

    # 2.2 Process an image (copies original with a "_processed" suffix beside it)
    packed-cli-demo.exe run process-image --path C:/pics/logo.png
    packed-cli-demo.exe run process-image C:/pics/logo.png --suffix _v2

    # 2.3 Write a text file
    packed-cli-demo.exe run write-text --path C:/out/note.txt --content "hello"
    packed-cli-demo.exe run wt C:/out/note.txt "hello world"

    # 2.4 Get the current date and time
    packed-cli-demo.exe run get-time
    packed-cli-demo.exe run now --utc          # alias + boolean flag

Exit codes (RFC 0018 §4.4):
    0  success                     1  command raised an exception
    2  command not found / bad arguments

Note: ``get_app_status`` below uses the default ``cli=False`` - it stays
callable from the front-end but is hidden from ``-h``/``list`` and reports
"command not found" if invoked via ``run``. This is the safe default: commands
must opt in to the command line explicitly.
"""

from __future__ import annotations

import datetime
import os
import shutil
from typing import Any, Dict

from auroraview import WebView
from auroraview.core.commands import CommandError, CommandErrorCode


def register_commands(view: WebView) -> None:
    """Register every demo command on ``view``.

    Kept separate from :func:`main` so the pack-time metadata dump and the
    headless invoke path (which build the WebView without calling ``show()``)
    register the exact same commands the GUI does.
    """

    # 2.1 - Import an image. Takes a local image path, returns a success result.
    @view.command(
        name="import-image",
        cli="imp",
        help="Import a local image",
        args_help={"path": "Absolute local path to the image"},
    )
    def import_image(path: str) -> Dict[str, Any]:
        """Import a local image and return its info on success."""
        if not os.path.isfile(path):
            raise CommandError(
                CommandErrorCode.INVALID_ARGUMENTS,
                f"Image not found: {path}",
                {"path": path},
            )
        size = os.path.getsize(path)
        return {
            "ok": True,
            "message": "Imported successfully",
            "path": os.path.abspath(path),
            "size_bytes": size,
        }

    # 2.2 - Process an image: copy the original with a name suffix, beside it.
    @view.command(
        name="process-image",
        cli="proc",
        help="Process an image: copy it beside the original with a name suffix",
        args_help={
            "path": "Absolute local path to the source image",
            "suffix": "Suffix appended to the file name (before the extension)",
        },
    )
    def process_image(path: str, suffix: str = "_processed") -> Dict[str, Any]:
        """Copy the source image with a name suffix, beside the original."""
        if not os.path.isfile(path):
            raise CommandError(
                CommandErrorCode.INVALID_ARGUMENTS,
                f"Image not found: {path}",
                {"path": path},
            )
        root, ext = os.path.splitext(os.path.abspath(path))
        dest = f"{root}{suffix}{ext}"
        shutil.copy2(path, dest)
        return {
            "ok": True,
            "message": "Processed successfully",
            "source": os.path.abspath(path),
            "output": dest,
        }

    # 2.3 - Write a text file. Params: output path, content.
    @view.command(
        name="write-text",
        cli=["wt", "txt"],
        help="Write text content to a local txt file",
        args_help={"path": "Output file path", "content": "Text content to write"},
    )
    def write_text(path: str, content: str) -> Dict[str, Any]:
        """Write a txt file locally."""
        out = os.path.abspath(path)
        parent = os.path.dirname(out)
        if parent:
            os.makedirs(parent, exist_ok=True)
        with open(out, "w", encoding="utf-8") as fh:
            fh.write(content)
        return {
            "ok": True,
            "message": "Written successfully",
            "path": out,
            "bytes_written": len(content.encode("utf-8")),
        }

    # 2.4 - Get the current date and time.
    @view.command(
        name="get-time",
        cli=["now"],
        help="Get the current date and time",
        args_help={"utc": "Return UTC time when true, otherwise local time"},
    )
    def get_time(utc: bool = False) -> Dict[str, Any]:
        """Return the current date and time."""
        now = datetime.datetime.utcnow() if utc else datetime.datetime.now()
        return {
            "date": now.strftime("%Y-%m-%d"),
            "time": now.strftime("%H:%M:%S"),
            "iso": now.isoformat(timespec="seconds"),
            "tz": "UTC" if utc else "local",
        }

    # Default cli=False: front-end callable, hidden from the CLI (safe default).
    @view.command(name="get-app-status")
    def get_app_status() -> Dict[str, Any]:
        """Return app status; front-end only, not exposed to the CLI."""
        return {"status": "running", "cli_exposed": False}


# Minimal front-end: proves the same commands are reachable from JavaScript.
# Note get-app-status (cli=False) is callable here but NOT from the terminal.
_HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>Packed CLI Demo</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 720px;
            margin: 40px auto;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }
        .card {
            background: white;
            border-radius: 12px;
            padding: 24px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
            margin-bottom: 20px;
        }
        h1 { color: #333; margin-top: 0; }
        p  { color: #666; }
        button {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white; border: none; padding: 12px 20px;
            border-radius: 6px; cursor: pointer; font-size: 14px; margin: 5px;
        }
        button:hover { opacity: 0.9; }
        #output {
            background: #1e1e1e; color: #f8f8f2; border-radius: 8px;
            padding: 16px; font-family: 'Consolas', monospace; font-size: 13px;
            white-space: pre-wrap; margin-top: 12px;
        }
        code { background: #f0f0f0; padding: 2px 6px; border-radius: 4px; }
    </style>
</head>
<body>
    <div class="card">
        <h1>Packed CLI Demo</h1>
        <p>The same commands below are callable from a packed exe's command line,
           e.g. <code>app.exe run get-time</code>. Try the buttons here, then pack
           and call them from a terminal.</p>
        <button onclick="callStatus()">get-app-status (cli=False, GUI only)</button>
        <button onclick="callTime()">get-time (CLI-enabled)</button>
        <div id="output">Ready.</div>
    </div>

    <script>
        function show(obj) {
            document.getElementById('output').textContent =
                typeof obj === 'object' ? JSON.stringify(obj, null, 2) : String(obj);
        }
        async function callStatus() {
            try { show(await auroraview.invoke('get-app-status')); }
            catch (e) { show('Error: ' + e.message); }
        }
        async function callTime() {
            try { show(await auroraview.invoke('get-time', {})); }
            catch (e) { show('Error: ' + e.message); }
        }
    </script>
</body>
</html>
"""


def main() -> None:
    """Run the demo as a GUI window (also the packed entry point)."""
    view = WebView(title="Packed CLI Demo", html=_HTML, width=760, height=560)
    register_commands(view)

    # In packed mode show() transparently switches to API-server / headless-CLI
    # mode based on AURORAVIEW_* env vars set by the Rust launcher; as a plain
    # script it just opens the window.
    view.show()


if __name__ == "__main__":
    main()
