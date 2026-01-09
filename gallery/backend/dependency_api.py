"""Dependency installation API for AuroraView Gallery.

This module provides API handlers for:
- Checking sample dependencies before running
- Installing missing dependencies with progress streaming
- Reporting installation status via WebView events

The workflow is:
1. Frontend calls api.check_dependencies(sample_id)
2. If missing dependencies found, frontend shows installation UI
3. Frontend calls api.install_dependencies(sample_id) 
4. Progress events are streamed to frontend: dep:progress, dep:complete, dep:error
5. Once installed, frontend can run the sample

Signed-off-by: Hal Long <hal.long@outlook.com>
"""

from __future__ import annotations

import threading
from pathlib import Path
from typing import TYPE_CHECKING

from .config import EXAMPLES_DIR
from .dependency_installer import (
    DependencyInstaller,
    parse_requirements_from_docstring,
    get_missing_requirements,
)
from .samples import get_sample_by_id, extract_docstring

if TYPE_CHECKING:
    from auroraview import WebView


def register_dependency_apis(view: WebView):
    """Register dependency installation API handlers.

    Args:
        view: The WebView instance to register handlers on.
    """
    installer = DependencyInstaller()
    _install_lock = threading.Lock()
    _cancel_event = threading.Event()

    @view.bind_call("api.check_dependencies")
    def check_dependencies(sample_id: str = "") -> dict:
        """Check if a sample has missing dependencies.

        Args:
            sample_id: The ID of the sample to check.

        Returns:
            dict with:
            - ok: True if check succeeded
            - sample_id: The sample ID
            - requirements: List of all requirements
            - missing: List of missing requirements
            - needs_install: True if installation needed
        """
        sample = get_sample_by_id(sample_id)
        if not sample:
            return {"ok": False, "error": f"Sample not found: {sample_id}"}

        sample_path = EXAMPLES_DIR / sample["source_file"]
        if not sample_path.exists():
            return {"ok": False, "error": f"File not found: {sample['source_file']}"}

        # Extract and parse requirements from docstring
        docstring = extract_docstring(sample_path) or ""
        requirements = parse_requirements_from_docstring(docstring)
        missing = get_missing_requirements(requirements)

        return {
            "ok": True,
            "sample_id": sample_id,
            "requirements": requirements,
            "missing": missing,
            "needs_install": len(missing) > 0,
        }

    @view.bind_call("api.install_dependencies")
    def install_dependencies(sample_id: str = "") -> dict:
        """Install missing dependencies for a sample.

        This starts installation in a background thread and streams
        progress events to the frontend:
        - dep:start - Installation started
        - dep:progress - Package installation progress
        - dep:complete - All packages installed
        - dep:error - Installation failed

        Args:
            sample_id: The ID of the sample to install dependencies for.

        Returns:
            dict with:
            - ok: True if installation started
            - message: Status message
        """
        # Check dependencies first
        check_result = check_dependencies(sample_id)
        if not check_result.get("ok"):
            return check_result

        missing = check_result.get("missing", [])
        if not missing:
            return {
                "ok": True,
                "message": "All dependencies already installed",
                "already_installed": True,
            }

        # Check if already installing
        if not _install_lock.acquire(blocking=False):
            return {
                "ok": False,
                "error": "Installation already in progress",
            }

        _cancel_event.clear()
        try:
            # Create emitter on the main thread to avoid PyO3 unsendable panic
            emitter = view.create_emitter()
        except Exception as exc:  # pragma: no cover - defensive
            _install_lock.release()
            return {
                "ok": False,
                "error": f"Failed to create emitter: {exc}",
            }

        def run_installation():
            """Run installation in background thread."""
            try:
                import sys
                print(f"[DependencyAPI] Starting installation thread for sample_id={sample_id}", file=sys.stderr)
                print(f"[DependencyAPI] Missing packages: {missing}", file=sys.stderr)
                
                # Emit start event
                emitter.emit("dep:start", {
                    "sample_id": sample_id,
                    "packages": missing,
                    "total": len(missing),
                })
                print(f"[DependencyAPI] Emitted dep:start event", file=sys.stderr)

                def on_progress(progress: dict):
                    """Handle progress updates."""
                    event_type = progress.get("type", "")
                    print(f"[DependencyAPI] Progress update: type={event_type}, package={progress.get('package')}", file=sys.stderr)
                    
                    if event_type == "start":
                        emitter.emit("dep:progress", {
                            "sample_id": sample_id,
                            "package": progress.get("package"),
                            "index": progress.get("index", 0),
                            "total": progress.get("total", len(missing)),
                            "message": progress.get("message", ""),
                            "phase": "starting",
                        })
                    elif event_type == "output":
                        emitter.emit("dep:progress", {
                            "sample_id": sample_id,
                            "package": progress.get("package"),
                            "line": progress.get("line", ""),
                            "phase": "installing",
                        })
                    elif event_type == "complete":
                        emitter.emit("dep:progress", {
                            "sample_id": sample_id,
                            "package": progress.get("package"),
                            "success": True,
                            "message": progress.get("message", ""),
                            "phase": "complete",
                        })
                    elif event_type == "error":
                        print(f"[DependencyAPI] Error in progress: {progress}", file=sys.stderr)
                        emitter.emit("dep:error", {
                            "sample_id": sample_id,
                            "package": progress.get("package"),
                            "error": progress.get("message", "Installation failed"),
                        })

                print(f"[DependencyAPI] Starting installer.install_missing()", file=sys.stderr)
                # Run installation
                result = installer.install_missing(missing, on_progress, cancel_event=_cancel_event)
                print(f"[DependencyAPI] Installation result: {result}", file=sys.stderr)

                if result.get("cancelled"):
                    print(f"[DependencyAPI] Installation was cancelled", file=sys.stderr)
                    emitter.emit("dep:error", {
                        "sample_id": sample_id,
                        "error": "Installation cancelled by user",
                        "cancelled": True,
                    })
                elif result.get("success"):
                    print(f"[DependencyAPI] Installation succeeded", file=sys.stderr)
                    emitter.emit("dep:complete", {
                        "sample_id": sample_id,
                        "installed": result.get("installed", []),
                        "message": "All dependencies installed successfully",
                    })
                else:
                    print(f"[DependencyAPI] Installation failed: {result.get('failed', [])}", file=sys.stderr)
                    emitter.emit("dep:error", {
                        "sample_id": sample_id,
                        "failed": result.get("failed", []),
                        "error": result.get("error", "Some dependencies failed to install"),
                    })
            except Exception as e:
                print(f"[DependencyAPI] Exception in installation thread: {e}", file=sys.stderr)
                import traceback
                traceback.print_exc(file=sys.stderr)
                emitter.emit("dep:error", {
                    "sample_id": sample_id,
                    "error": f"Installation thread exception: {e}",
                })
            finally:
                print(f"[DependencyAPI] Installation thread finished, releasing lock", file=sys.stderr)
                _install_lock.release()

        # Start installation in background
        thread = threading.Thread(target=run_installation, daemon=True)
        thread.start()

        return {
            "ok": True,
            "message": f"Installing {len(missing)} package(s)...",
            "packages": missing,
        }

    @view.bind_call("api.cancel_installation")
    def cancel_installation() -> dict:
        """Cancel the ongoing dependency installation.

        Returns:
            dict with ok=True if cancellation signal sent.
        """
        _cancel_event.set()
        return {"ok": True, "message": "Cancellation signal sent"}

    @view.bind_call("api.get_all_sample_dependencies")
    def get_all_sample_dependencies() -> dict:
        """Get dependency info for all samples.

        Returns a map of sample_id -> dependency info for samples
        that have any requirements defined.
        """
        from .samples import get_samples_list

        result = {}
        for sample in get_samples_list():
            sample_id = sample["id"]
            sample_path = EXAMPLES_DIR / sample["source_file"]

            if sample_path.exists():
                docstring = extract_docstring(sample_path) or ""
                requirements = parse_requirements_from_docstring(docstring)
                if requirements:
                    missing = get_missing_requirements(requirements)
                    result[sample_id] = {
                        "requirements": requirements,
                        "missing": missing,
                        "needs_install": len(missing) > 0,
                    }

        return {"ok": True, "samples": result}

