"""WebView2 extension management API handlers for AuroraView Gallery.

This module provides API handlers for:
- Installing extensions to WebView2's extensions directory
- Listing installed WebView2 extensions
- Removing WebView2 extensions
- Opening the extensions directory
- Installing extensions from Chrome Web Store URLs
"""

from __future__ import annotations

import json as json_module
import os
import platform
import re
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from auroraview import WebView


def get_webview_extensions_dir() -> Path:
    """Get the WebView2 extensions directory path."""
    if platform.system() == "Windows":
        local_app_data = os.environ.get("LOCALAPPDATA", os.path.expanduser("~"))
        return Path(local_app_data) / "AuroraView" / "Extensions"
    else:
        # On non-Windows, use ~/.local/share/AuroraView/Extensions
        return Path.home() / ".local" / "share" / "AuroraView" / "Extensions"


def get_extension_config_path() -> Path:
    """Get the extension configuration file path."""
    if platform.system() == "Windows":
        local_app_data = os.environ.get("LOCALAPPDATA", os.path.expanduser("~"))
        return Path(local_app_data) / "AuroraView" / "extension_config.json"
    else:
        return Path.home() / ".local" / "share" / "AuroraView" / "extension_config.json"


def load_extension_config() -> dict:
    """Load extension configuration from file."""
    config_path = get_extension_config_path()
    if config_path.exists():
        try:
            with open(config_path, "r", encoding="utf-8") as f:
                return json_module.load(f)
        except (json_module.JSONDecodeError, OSError) as e:
            print(f"[Python] Error loading extension config: {e}", file=sys.stderr)
    return {"disabled_extensions": []}


def save_extension_config(config: dict) -> bool:
    """Save extension configuration to file."""
    config_path = get_extension_config_path()
    try:
        config_path.parent.mkdir(parents=True, exist_ok=True)
        with open(config_path, "w", encoding="utf-8") as f:
            json_module.dump(config, f, indent=2)
        return True
    except OSError as e:
        print(f"[Python] Error saving extension config: {e}", file=sys.stderr)
        return False


def is_extension_enabled(extension_id: str) -> bool:
    """Check if an extension is enabled."""
    config = load_extension_config()
    return extension_id not in config.get("disabled_extensions", [])


def resolve_i18n_message(text: str, extension_path: Path, default_locale: str = "en") -> str:
    """Resolve Chrome extension i18n message placeholders.

    Chrome extensions use __MSG_key__ format for internationalized strings.
    The actual values are in _locales/<locale>/messages.json.

    Args:
        text: Text that may contain __MSG_key__ placeholders
        extension_path: Path to the extension directory
        default_locale: Default locale to use (default: "en")

    Returns:
        Resolved text with placeholders replaced
    """
    if not text or not text.startswith("__MSG_") or not text.endswith("__"):
        return text

    # Extract the message key (remove __MSG_ prefix and __ suffix)
    key = text[6:-2]

    # Try to find messages.json
    locales_dir = extension_path / "_locales"
    if not locales_dir.exists():
        return text

    # Try locales in order: default_locale, "en", first available
    locales_to_try = [default_locale, "en"]
    available_locales = [d.name for d in locales_dir.iterdir() if d.is_dir()]

    for locale in locales_to_try:
        if locale in available_locales:
            messages_file = locales_dir / locale / "messages.json"
            if messages_file.exists():
                try:
                    with open(messages_file, "r", encoding="utf-8") as f:
                        messages = json_module.load(f)
                    if key in messages:
                        return messages[key].get("message", text)
                except (json_module.JSONDecodeError, OSError):
                    pass

    # Try first available locale
    if available_locales:
        messages_file = locales_dir / available_locales[0] / "messages.json"
        if messages_file.exists():
            try:
                with open(messages_file, "r", encoding="utf-8") as f:
                    messages = json_module.load(f)
                if key in messages:
                    return messages[key].get("message", text)
            except (json_module.JSONDecodeError, OSError):
                pass

    return text


def register_webview_extension_apis(view: WebView) -> None:
    """Register all WebView2 extension management API handlers.

    Args:
        view: The WebView instance to register handlers on
    """

    @view.bind_call("api.install_to_webview")
    def install_to_webview(path: str = "", name: str = "") -> dict:
        """Install an unpacked extension to WebView2's extensions directory.

        This copies the extension folder to the WebView2 extensions directory.
        The extension will be loaded when the application restarts.

        Args:
            path: Path to the unpacked extension folder (must contain manifest.json)
            name: Optional custom name for the extension folder
        """
        print(f"[Python:install_to_webview] path={path}, name={name}", file=sys.stderr)

        if not path:
            return {"ok": False, "error": "No path provided"}

        source_path = Path(path)
        if not source_path.exists():
            return {"ok": False, "error": f"Extension path not found: {path}"}

        if not source_path.is_dir():
            return {
                "ok": False,
                "error": "Extension path must be a directory (unpacked extension)",
            }

        manifest_path = source_path / "manifest.json"
        if not manifest_path.exists():
            return {
                "ok": False,
                "error": "Extension folder must contain manifest.json",
            }

        try:
            # Read manifest to get extension info
            with open(manifest_path, "r", encoding="utf-8") as f:
                manifest = json_module.load(f)

            ext_name = manifest.get("name", "Unknown")
            ext_version = manifest.get("version", "0.0.0")

            # Determine target folder name
            folder_name = name if name else source_path.name

            # Create extensions directory
            extensions_dir = get_webview_extensions_dir()
            extensions_dir.mkdir(parents=True, exist_ok=True)

            target_path = extensions_dir / folder_name

            # Remove existing extension with same name
            if target_path.exists():
                shutil.rmtree(target_path)

            # Copy extension folder
            shutil.copytree(source_path, target_path)

            print(
                f"[Python:install_to_webview] SUCCESS: Installed '{ext_name}' v{ext_version} to {target_path}",
                file=sys.stderr,
            )

            return {
                "ok": True,
                "success": True,
                "id": folder_name,
                "name": ext_name,
                "version": ext_version,
                "path": str(target_path),
                "extensionsDir": str(extensions_dir),
                "message": f"Extension '{ext_name}' v{ext_version} installed. Restart the application to load it.",
                "requiresRestart": True,
            }
        except Exception as e:
            error_msg = f"Failed to install extension: {e}"
            print(f"[Python:install_to_webview] ERROR: {error_msg}", file=sys.stderr)
            import traceback

            traceback.print_exc(file=sys.stderr)
            return {"ok": False, "error": error_msg}

    @view.bind_call("api.list_webview_extensions")
    def list_webview_extensions() -> dict:
        """List extensions installed in WebView2's extensions directory."""
        extensions_dir = get_webview_extensions_dir()
        extensions = []

        print(
            f"[Python:list_webview_extensions] Scanning: {extensions_dir}",
            file=sys.stderr,
        )

        if extensions_dir.exists():
            for entry in extensions_dir.iterdir():
                if entry.is_dir():
                    manifest_path = entry / "manifest.json"
                    if manifest_path.exists():
                        try:
                            with open(manifest_path, "r", encoding="utf-8") as f:
                                manifest = json_module.load(f)

                            # Check for side panel
                            side_panel = manifest.get("side_panel", {})
                            has_side_panel = bool(side_panel.get("default_path"))
                            side_panel_path = side_panel.get("default_path", "")
                            # Normalize path: remove leading ./ and normalize slashes
                            if side_panel_path.startswith("./"):
                                side_panel_path = side_panel_path[2:]
                            side_panel_path = side_panel_path.replace("\\", "/")

                            # Check for popup (action.default_popup)
                            action = manifest.get("action", {})
                            has_popup = bool(action.get("default_popup"))
                            popup_path = action.get("default_popup", "")
                            # Normalize popup path as well
                            if popup_path.startswith("./"):
                                popup_path = popup_path[2:]
                            popup_path = popup_path.replace("\\", "/")

                            # Get options page
                            options_url = manifest.get("options_page", "")
                            if not options_url:
                                options_ui = manifest.get("options_ui", {})
                                options_url = options_ui.get("page", "")

                            # Get permissions
                            permissions = manifest.get("permissions", [])
                            host_permissions = manifest.get("host_permissions", [])

                            # Get homepage URL
                            homepage_url = manifest.get("homepage_url", "")

                            # Get icons from manifest
                            # Chrome extensions define icons in manifest.icons or action.default_icon
                            # Use custom protocol URL for packed mode compatibility
                            icons = []
                            ext_id = entry.name
                            manifest_icons = manifest.get("icons", {})
                            if manifest_icons:
                                for size, icon_path in manifest_icons.items():
                                    icon_file = entry / icon_path
                                    if icon_file.exists():
                                        # Use custom protocol: https://auroraview.localhost/extension/{id}/{path}
                                        # Normalize icon path (remove leading ./ and use forward slashes)
                                        normalized_path = icon_path.replace("\\", "/")
                                        if normalized_path.startswith("./"):
                                            normalized_path = normalized_path[2:]
                                        icons.append(
                                            {
                                                "size": int(size),
                                                "url": f"https://auroraview.localhost/extension/{ext_id}/{normalized_path}",
                                            }
                                        )

                            # Also check action.default_icon
                            action_icon = action.get("default_icon")
                            if action_icon and not icons:
                                if isinstance(action_icon, str):
                                    icon_file = entry / action_icon
                                    if icon_file.exists():
                                        normalized_path = action_icon.replace("\\", "/")
                                        if normalized_path.startswith("./"):
                                            normalized_path = normalized_path[2:]
                                        icons.append(
                                            {
                                                "size": 32,
                                                "url": f"https://auroraview.localhost/extension/{ext_id}/{normalized_path}",
                                            }
                                        )
                                elif isinstance(action_icon, dict):
                                    for size, icon_path in action_icon.items():
                                        icon_file = entry / icon_path
                                        if icon_file.exists():
                                            normalized_path = icon_path.replace("\\", "/")
                                            if normalized_path.startswith("./"):
                                                normalized_path = normalized_path[2:]
                                            icons.append(
                                                {
                                                    "size": int(size),
                                                    "url": f"https://auroraview.localhost/extension/{ext_id}/{normalized_path}",
                                                }
                                            )

                            # Sort icons by size (largest first)
                            icons.sort(key=lambda x: x["size"], reverse=True)

                            # Determine install type based on extension ID format
                            # Chrome Web Store extensions have 32-char lowercase IDs
                            # Local unpacked extensions use folder names
                            is_webstore_id = (
                                len(ext_id) == 32 and ext_id.isalpha() and ext_id.islower()
                            )
                            install_type = "normal" if is_webstore_id else "development"

                            # Get name and description, resolving i18n placeholders
                            raw_name = manifest.get("name", "Unknown")
                            raw_desc = manifest.get("description", "")
                            ext_name = resolve_i18n_message(raw_name, entry)
                            ext_desc = resolve_i18n_message(raw_desc, entry)

                            extensions.append(
                                {
                                    "id": entry.name,
                                    "name": ext_name,
                                    "version": manifest.get("version", "0.0.0"),
                                    "description": ext_desc,
                                    "path": str(entry),
                                    "hasSidePanel": has_side_panel,
                                    "sidePanelPath": side_panel_path,
                                    "hasPopup": has_popup,
                                    "popupPath": popup_path,
                                    "optionsUrl": options_url,
                                    "permissions": permissions,
                                    "hostPermissions": host_permissions,
                                    "homepageUrl": homepage_url,
                                    "installType": install_type,
                                    "enabled": is_extension_enabled(entry.name),
                                    "icons": icons,
                                }
                            )
                        except Exception as e:
                            print(
                                f"[Python:list_webview_extensions] Error reading {manifest_path}: {e}",
                                file=sys.stderr,
                            )

        print(
            f"[Python:list_webview_extensions] Found {len(extensions)} extensions",
            file=sys.stderr,
        )

        return {
            "ok": True,
            "extensions": extensions,
            "extensionsDir": str(extensions_dir),
            "count": len(extensions),
        }

    @view.bind_call("api.remove_webview_extension")
    def remove_webview_extension(id: str = "") -> dict:
        """Remove an extension from WebView2's extensions directory.

        Args:
            id: Extension folder name (ID)
        """
        print(f"[Python:remove_webview_extension] id={id}", file=sys.stderr)

        if not id:
            return {"ok": False, "error": "No extension ID provided"}

        extensions_dir = get_webview_extensions_dir()
        extension_path = extensions_dir / id

        if not extension_path.exists():
            return {"ok": False, "error": f"Extension '{id}' not found"}

        try:
            shutil.rmtree(extension_path)
            print(
                f"[Python:remove_webview_extension] SUCCESS: Removed '{id}'",
                file=sys.stderr,
            )

            return {
                "ok": True,
                "success": True,
                "id": id,
                "message": f"Extension '{id}' removed. Restart the application to apply changes.",
                "requiresRestart": True,
            }
        except Exception as e:
            error_msg = f"Failed to remove extension: {e}"
            print(f"[Python:remove_webview_extension] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

    @view.bind_call("api.open_extensions_dir")
    def open_extensions_dir() -> dict:
        """Open the WebView2 extensions directory in file explorer."""
        extensions_dir = get_webview_extensions_dir()
        extensions_dir.mkdir(parents=True, exist_ok=True)

        print(f"[Python:open_extensions_dir] Opening: {extensions_dir}", file=sys.stderr)

        try:
            if sys.platform == "win32":
                os.startfile(str(extensions_dir))
            elif sys.platform == "darwin":
                subprocess.run(["open", str(extensions_dir)], check=True)
            else:
                subprocess.run(["xdg-open", str(extensions_dir)], check=True)

            return {"ok": True, "success": True, "path": str(extensions_dir)}
        except Exception as e:
            error_msg = f"Failed to open directory: {e}"
            print(f"[Python:open_extensions_dir] ERROR: {error_msg}", file=sys.stderr)
            return {"ok": False, "error": error_msg}

    @view.bind_call("api.install_extension_from_url")
    def install_extension_from_url(url: str = "") -> dict:
        """Install an extension from Chrome Web Store or Edge Add-ons URL.

        This downloads the extension CRX file and extracts it to the extensions directory.

        Args:
            url: Chrome Web Store or Edge Add-ons URL
        """
        print(f"[Python:install_extension_from_url] url={url}", file=sys.stderr)

        if not url:
            return {"ok": False, "error": "No URL provided"}

        # Parse Chrome Web Store URL (support both old and new formats)
        # Old: https://chrome.google.com/webstore/detail/name/id
        # New: https://chromewebstore.google.com/detail/name/id
        chrome_pattern_old = r"https://chrome\.google\.com/webstore/detail/[^/]+/([a-z]{32})"
        chrome_pattern_new = r"https://chromewebstore\.google\.com/detail/[^/]+/([a-z]{32})"
        edge_pattern = r"https://microsoftedge\.microsoft\.com/addons/detail/[^/]+/([a-z]{32})"

        chrome_match_old = re.match(chrome_pattern_old, url, re.IGNORECASE)
        chrome_match_new = re.match(chrome_pattern_new, url, re.IGNORECASE)
        edge_match = re.match(edge_pattern, url, re.IGNORECASE)

        extension_id = None
        source = None

        if chrome_match_old:
            extension_id = chrome_match_old.group(1)
            source = "chrome"
        elif chrome_match_new:
            extension_id = chrome_match_new.group(1)
            source = "chrome"
        elif edge_match:
            extension_id = edge_match.group(1)
            source = "edge"
        else:
            return {
                "ok": False,
                "error": "Invalid URL. Please use a Chrome Web Store or Edge Add-ons URL.",
            }

        print(
            f"[Python:install_extension_from_url] Detected {source} extension: {extension_id}",
            file=sys.stderr,
        )

        try:
            # Create temp directory for download
            with tempfile.TemporaryDirectory() as temp_dir:
                temp_path = Path(temp_dir)

                if source == "chrome":
                    # Download CRX from Chrome Web Store
                    crx_url = f"https://clients2.google.com/service/update2/crx?response=redirect&prodversion=120.0.0.0&acceptformat=crx2,crx3&x=id%3D{extension_id}%26uc"
                    crx_path = temp_path / f"{extension_id}.crx"

                    print(
                        f"[Python:install_extension_from_url] Downloading from: {crx_url}",
                        file=sys.stderr,
                    )

                    # Download with proper headers
                    request = urllib.request.Request(
                        crx_url,
                        headers={
                            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
                            "Accept": "application/x-chrome-extension,*/*",
                        },
                    )

                    try:
                        with urllib.request.urlopen(request, timeout=60) as response:
                            with open(crx_path, "wb") as f:
                                f.write(response.read())
                    except urllib.error.HTTPError as e:
                        if e.code == 204:
                            return {
                                "ok": False,
                                "error": "Extension not found or not available for download. Try downloading manually from the Web Store.",
                            }
                        raise

                    print(
                        f"[Python:install_extension_from_url] Downloaded CRX: {crx_path.stat().st_size} bytes",
                        file=sys.stderr,
                    )

                    # Extract CRX (it's a ZIP with a header)
                    extract_path = temp_path / "extracted"
                    extract_path.mkdir()

                    # CRX3 format: magic (4) + version (4) + header_length (4) + header + zip
                    with open(crx_path, "rb") as f:
                        magic = f.read(4)
                        if magic != b"Cr24":
                            return {"ok": False, "error": "Invalid CRX file format"}

                        version = int.from_bytes(f.read(4), "little")
                        print(
                            f"[Python:install_extension_from_url] CRX version: {version}",
                            file=sys.stderr,
                        )

                        if version == 3:
                            # CRX3: header_length (4 bytes), then header, then zip
                            header_length = int.from_bytes(f.read(4), "little")
                            f.seek(12 + header_length)  # Skip to ZIP content
                        elif version == 2:
                            # CRX2: public_key_length (4), signature_length (4), then key, sig, zip
                            pub_key_len = int.from_bytes(f.read(4), "little")
                            sig_len = int.from_bytes(f.read(4), "little")
                            f.seek(16 + pub_key_len + sig_len)
                        else:
                            return {
                                "ok": False,
                                "error": f"Unsupported CRX version: {version}",
                            }

                        # Read ZIP content
                        zip_content = f.read()

                    # Write ZIP content and extract
                    zip_path = temp_path / f"{extension_id}.zip"
                    with open(zip_path, "wb") as f:
                        f.write(zip_content)

                    with zipfile.ZipFile(zip_path, "r") as zf:
                        zf.extractall(extract_path)

                else:  # edge
                    # Edge Add-ons doesn't have a direct download API
                    return {
                        "ok": False,
                        "error": "Edge Add-ons direct download is not supported. Please download the extension manually and use 'Load unpacked'.",
                    }

                # Read manifest to get extension info
                manifest_path = extract_path / "manifest.json"
                if not manifest_path.exists():
                    return {
                        "ok": False,
                        "error": "Invalid extension: manifest.json not found",
                    }

                with open(manifest_path, "r", encoding="utf-8") as f:
                    manifest = json_module.load(f)

                ext_name = manifest.get("name", "Unknown")
                ext_version = manifest.get("version", "0.0.0")

                # Copy to extensions directory
                extensions_dir = get_webview_extensions_dir()
                extensions_dir.mkdir(parents=True, exist_ok=True)

                # Use extension ID as folder name
                target_path = extensions_dir / extension_id

                if target_path.exists():
                    shutil.rmtree(target_path)

                shutil.copytree(extract_path, target_path)

                print(
                    f"[Python:install_extension_from_url] SUCCESS: Installed '{ext_name}' v{ext_version} to {target_path}",
                    file=sys.stderr,
                )

                return {
                    "ok": True,
                    "success": True,
                    "id": extension_id,
                    "name": ext_name,
                    "version": ext_version,
                    "path": str(target_path),
                    "extensionsDir": str(extensions_dir),
                    "message": f"Extension '{ext_name}' v{ext_version} installed. Restart the application to load it.",
                    "requiresRestart": True,
                }

        except urllib.error.URLError as e:
            error_msg = f"Network error: {e.reason}"
            print(
                f"[Python:install_extension_from_url] ERROR: {error_msg}",
                file=sys.stderr,
            )
            return {"ok": False, "error": error_msg}
        except Exception as e:
            error_msg = f"Failed to install extension: {e}"
            print(
                f"[Python:install_extension_from_url] ERROR: {error_msg}",
                file=sys.stderr,
            )
            import traceback

            traceback.print_exc(file=sys.stderr)
            return {"ok": False, "error": error_msg}

    @view.bind_call("api.set_extension_enabled")
    def set_extension_enabled(id: str = "", enabled: bool = True) -> dict:
        """Enable or disable an extension.

        This persists the enabled state to a config file. Disabled extensions
        will not be loaded on the next application restart.

        Args:
            id: Extension ID (folder name)
            enabled: True to enable, False to disable
        """
        print(
            f"[Python:set_extension_enabled] id={id}, enabled={enabled}",
            file=sys.stderr,
        )

        if not id:
            return {"ok": False, "error": "No extension ID provided"}

        config = load_extension_config()
        disabled_list = config.get("disabled_extensions", [])

        if enabled:
            # Remove from disabled list
            if id in disabled_list:
                disabled_list.remove(id)
        else:
            # Add to disabled list
            if id not in disabled_list:
                disabled_list.append(id)

        config["disabled_extensions"] = disabled_list

        if save_extension_config(config):
            print(
                f"[Python:set_extension_enabled] SUCCESS: {id} -> enabled={enabled}",
                file=sys.stderr,
            )
            return {
                "ok": True,
                "id": id,
                "enabled": enabled,
                "requiresRestart": True,
                "message": f"Extension will be {'enabled' if enabled else 'disabled'} on next restart.",
            }
        else:
            return {"ok": False, "error": "Failed to save configuration"}

    @view.bind_call("api.get_extension_config")
    def get_extension_config() -> dict:
        """Get the extension configuration including disabled extensions list."""
        config = load_extension_config()
        return {
            "ok": True,
            "config": config,
            "configPath": str(get_extension_config_path()),
        }
