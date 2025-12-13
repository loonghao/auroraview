"""Cookie Management Demo - Session and persistent cookies.

This example demonstrates AuroraView's cookie management capabilities,
including creating, reading, and managing cookies for session persistence.

Features demonstrated:
- Creating session cookies
- Creating persistent cookies with expiration
- Cookie attributes (secure, httpOnly, sameSite)
- Reading and displaying cookies
- Deleting cookies
- Cookie validation
"""

from __future__ import annotations

import datetime
import json

# WebView import is done in main() to avoid circular imports
from auroraview.core.cookies import Cookie

HTML = """
<!DOCTYPE html>
<html>
<head>
    <title>Cookie Management Demo</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #2c3e50 0%, #1a1a2e 100%);
            color: #ecf0f1;
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 900px;
            margin: 0 auto;
        }
        h1 {
            text-align: center;
            margin-bottom: 10px;
            background: linear-gradient(90deg, #f1c40f, #e67e22);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
        }
        .subtitle {
            text-align: center;
            color: #7f8c8d;
            margin-bottom: 30px;
        }
        .grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 20px;
        }
        .card {
            background: rgba(255,255,255,0.05);
            border-radius: 12px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        .card h2 {
            font-size: 16px;
            color: #f1c40f;
            margin-bottom: 15px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .form-group {
            margin-bottom: 15px;
        }
        .form-group label {
            display: block;
            margin-bottom: 5px;
            color: #bdc3c7;
            font-size: 13px;
        }
        .form-group input, .form-group select {
            width: 100%;
            padding: 10px;
            border: 1px solid rgba(255,255,255,0.2);
            border-radius: 6px;
            background: rgba(0,0,0,0.2);
            color: white;
            font-size: 14px;
        }
        .form-group input:focus, .form-group select:focus {
            outline: none;
            border-color: #f1c40f;
        }
        .form-row {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 15px;
        }
        .checkbox-group {
            display: flex;
            gap: 20px;
            margin-bottom: 15px;
        }
        .checkbox-group label {
            display: flex;
            align-items: center;
            gap: 8px;
            cursor: pointer;
            font-size: 13px;
        }
        .checkbox-group input[type="checkbox"] {
            width: 18px;
            height: 18px;
            accent-color: #f1c40f;
        }
        .btn-group {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }
        button {
            padding: 10px 20px;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s;
            background: #f1c40f;
            color: #2c3e50;
            font-weight: 500;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 12px rgba(241,196,15,0.3);
        }
        button.secondary {
            background: #34495e;
            color: white;
        }
        button.danger {
            background: #e74c3c;
            color: white;
        }
        .cookie-list {
            list-style: none;
            max-height: 300px;
            overflow-y: auto;
        }
        .cookie-item {
            background: rgba(0,0,0,0.2);
            border-radius: 8px;
            padding: 15px;
            margin-bottom: 10px;
            border-left: 3px solid #f1c40f;
        }
        .cookie-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 10px;
        }
        .cookie-name {
            font-weight: 600;
            color: #f1c40f;
        }
        .cookie-actions {
            display: flex;
            gap: 5px;
        }
        .cookie-actions button {
            padding: 4px 10px;
            font-size: 12px;
        }
        .cookie-details {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 8px;
            font-size: 12px;
        }
        .cookie-detail {
            display: flex;
            justify-content: space-between;
            padding: 4px 8px;
            background: rgba(255,255,255,0.05);
            border-radius: 4px;
        }
        .cookie-detail .label { color: #7f8c8d; }
        .cookie-detail .value { color: #ecf0f1; }
        .cookie-badges {
            display: flex;
            gap: 5px;
            margin-top: 8px;
        }
        .badge {
            padding: 2px 8px;
            border-radius: 10px;
            font-size: 10px;
            font-weight: 500;
        }
        .badge-secure { background: #27ae60; color: white; }
        .badge-httponly { background: #3498db; color: white; }
        .badge-session { background: #9b59b6; color: white; }
        .badge-persistent { background: #e67e22; color: white; }
        .empty-state {
            text-align: center;
            padding: 40px;
            color: #7f8c8d;
        }
        .full-width { grid-column: 1 / -1; }
        .status-bar {
            padding: 10px 15px;
            background: rgba(0,0,0,0.3);
            border-radius: 6px;
            font-family: monospace;
            font-size: 13px;
            margin-top: 15px;
        }
        .status-bar.success { border-left: 3px solid #27ae60; }
        .status-bar.error { border-left: 3px solid #e74c3c; }
        .status-bar.info { border-left: 3px solid #3498db; }
    </style>
</head>
<body>
    <div class="container">
        <h1>Cookie Management Demo</h1>
        <p class="subtitle">Create, manage, and inspect HTTP cookies</p>

        <div class="grid">
            <!-- Create Cookie Form -->
            <div class="card">
                <h2>Create Cookie</h2>
                <div class="form-group">
                    <label for="cookie-name">Name</label>
                    <input type="text" id="cookie-name" placeholder="session_id">
                </div>
                <div class="form-group">
                    <label for="cookie-value">Value</label>
                    <input type="text" id="cookie-value" placeholder="abc123xyz">
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="cookie-domain">Domain</label>
                        <input type="text" id="cookie-domain" placeholder="example.com">
                    </div>
                    <div class="form-group">
                        <label for="cookie-path">Path</label>
                        <input type="text" id="cookie-path" value="/">
                    </div>
                </div>
                <div class="form-row">
                    <div class="form-group">
                        <label for="cookie-expires">Expires (days)</label>
                        <input type="number" id="cookie-expires" placeholder="Leave empty for session">
                    </div>
                    <div class="form-group">
                        <label for="cookie-samesite">SameSite</label>
                        <select id="cookie-samesite">
                            <option value="">None</option>
                            <option value="Strict">Strict</option>
                            <option value="Lax">Lax</option>
                            <option value="None">None (requires Secure)</option>
                        </select>
                    </div>
                </div>
                <div class="checkbox-group">
                    <label>
                        <input type="checkbox" id="cookie-secure">
                        Secure
                    </label>
                    <label>
                        <input type="checkbox" id="cookie-httponly">
                        HttpOnly
                    </label>
                </div>
                <div class="btn-group">
                    <button onclick="createCookie()">Create Cookie</button>
                    <button onclick="clearForm()" class="secondary">Clear</button>
                </div>
                <div id="create-status" class="status-bar info" style="display: none;"></div>
            </div>

            <!-- Quick Actions -->
            <div class="card">
                <h2>Quick Actions</h2>
                <p style="color: #7f8c8d; font-size: 13px; margin-bottom: 15px;">
                    Create common cookie types with one click
                </p>
                <div class="btn-group" style="flex-direction: column;">
                    <button onclick="createSessionCookie()">
                        Create Session Cookie
                    </button>
                    <button onclick="createPersistentCookie()" class="secondary">
                        Create 7-Day Cookie
                    </button>
                    <button onclick="createSecureCookie()" class="secondary">
                        Create Secure Cookie
                    </button>
                    <button onclick="createAuthCookie()" class="secondary">
                        Create Auth Cookie (HttpOnly)
                    </button>
                </div>
                <div style="margin-top: 20px;">
                    <h3 style="font-size: 14px; color: #f1c40f; margin-bottom: 10px;">Bulk Operations</h3>
                    <div class="btn-group">
                        <button onclick="refreshCookies()" class="secondary">Refresh List</button>
                        <button onclick="deleteAllCookies()" class="danger">Delete All</button>
                    </div>
                </div>
            </div>

            <!-- Cookie List -->
            <div class="card full-width">
                <h2>Active Cookies</h2>
                <ul class="cookie-list" id="cookie-list">
                    <li class="empty-state">
                        No cookies yet. Create one to get started!
                    </li>
                </ul>
            </div>
        </div>
    </div>

    <script>
        let cookies = [];

        function showStatus(message, type = 'info') {
            const status = document.getElementById('create-status');
            status.textContent = message;
            status.className = 'status-bar ' + type;
            status.style.display = 'block';
            setTimeout(() => status.style.display = 'none', 3000);
        }

        function createCookie() {
            const name = document.getElementById('cookie-name').value.trim();
            const value = document.getElementById('cookie-value').value.trim();
            const domain = document.getElementById('cookie-domain').value.trim();
            const path = document.getElementById('cookie-path').value.trim() || '/';
            const expiresDays = document.getElementById('cookie-expires').value;
            const sameSite = document.getElementById('cookie-samesite').value;
            const secure = document.getElementById('cookie-secure').checked;
            const httpOnly = document.getElementById('cookie-httponly').checked;

            if (!name || !value) {
                showStatus('Name and value are required', 'error');
                return;
            }

            window.auroraview.api.create_cookie({
                name, value, domain, path,
                expires_days: expiresDays ? parseInt(expiresDays) : null,
                same_site: sameSite || null,
                secure, http_only: httpOnly
            });
        }

        function clearForm() {
            document.getElementById('cookie-name').value = '';
            document.getElementById('cookie-value').value = '';
            document.getElementById('cookie-domain').value = '';
            document.getElementById('cookie-path').value = '/';
            document.getElementById('cookie-expires').value = '';
            document.getElementById('cookie-samesite').value = '';
            document.getElementById('cookie-secure').checked = false;
            document.getElementById('cookie-httponly').checked = false;
        }

        function createSessionCookie() {
            window.auroraview.api.create_quick_cookie({ type: 'session' });
        }

        function createPersistentCookie() {
            window.auroraview.api.create_quick_cookie({ type: 'persistent' });
        }

        function createSecureCookie() {
            window.auroraview.api.create_quick_cookie({ type: 'secure' });
        }

        function createAuthCookie() {
            window.auroraview.api.create_quick_cookie({ type: 'auth' });
        }

        function refreshCookies() {
            window.auroraview.api.get_cookies();
        }

        function deleteAllCookies() {
            if (confirm('Delete all cookies?')) {
                window.auroraview.api.delete_all_cookies();
            }
        }

        function deleteCookie(name) {
            window.auroraview.api.delete_cookie({ name });
        }

        function copyCookie(name) {
            const cookie = cookies.find(c => c.name === name);
            if (cookie) {
                navigator.clipboard.writeText(JSON.stringify(cookie, null, 2));
                showStatus('Cookie copied to clipboard', 'success');
            }
        }

        function renderCookies(cookieList) {
            cookies = cookieList;
            const list = document.getElementById('cookie-list');

            if (cookieList.length === 0) {
                list.innerHTML = '<li class="empty-state">No cookies yet. Create one to get started!</li>';
                return;
            }

            list.innerHTML = cookieList.map(cookie => {
                const isSession = !cookie.expires;
                const badges = [];
                if (cookie.secure) badges.push('<span class="badge badge-secure">Secure</span>');
                if (cookie.http_only) badges.push('<span class="badge badge-httponly">HttpOnly</span>');
                badges.push(isSession 
                    ? '<span class="badge badge-session">Session</span>'
                    : '<span class="badge badge-persistent">Persistent</span>'
                );

                return `
                    <li class="cookie-item">
                        <div class="cookie-header">
                            <span class="cookie-name">${cookie.name}</span>
                            <div class="cookie-actions">
                                <button onclick="copyCookie('${cookie.name}')" class="secondary">Copy</button>
                                <button onclick="deleteCookie('${cookie.name}')" class="danger">Delete</button>
                            </div>
                        </div>
                        <div class="cookie-details">
                            <div class="cookie-detail">
                                <span class="label">Value</span>
                                <span class="value">${cookie.value.substring(0, 20)}${cookie.value.length > 20 ? '...' : ''}</span>
                            </div>
                            <div class="cookie-detail">
                                <span class="label">Domain</span>
                                <span class="value">${cookie.domain || '(current)'}</span>
                            </div>
                            <div class="cookie-detail">
                                <span class="label">Path</span>
                                <span class="value">${cookie.path}</span>
                            </div>
                            <div class="cookie-detail">
                                <span class="label">Expires</span>
                                <span class="value">${cookie.expires || 'Session'}</span>
                            </div>
                            ${cookie.same_site ? `
                            <div class="cookie-detail">
                                <span class="label">SameSite</span>
                                <span class="value">${cookie.same_site}</span>
                            </div>
                            ` : ''}
                        </div>
                        <div class="cookie-badges">${badges.join('')}</div>
                    </li>
                `;
            }).join('');
        }

        // Listen for updates
        window.addEventListener('auroraviewready', () => {
            window.auroraview.on('cookies_updated', (data) => {
                renderCookies(data.cookies);
            });

            window.auroraview.on('cookie_created', (data) => {
                showStatus(`Cookie "${data.name}" created successfully`, 'success');
                refreshCookies();
            });

            window.auroraview.on('cookie_deleted', (data) => {
                showStatus(`Cookie "${data.name}" deleted`, 'info');
                refreshCookies();
            });

            window.auroraview.on('cookie_error', (data) => {
                showStatus(data.message, 'error');
            });

            // Initial load
            refreshCookies();
        });
    </script>
</body>
</html>
"""


class CookieManager:
    """Manages cookies for the demo."""

    def __init__(self, view):
        self.view = view
        self.cookies = []
        self.cookie_counter = 0

    def create_cookie(
        self,
        name: str,
        value: str,
        domain: str = None,
        path: str = "/",
        expires_days: int = None,
        same_site: str = None,
        secure: bool = False,
        http_only: bool = False,
    ) -> None:
        """Create a new cookie."""
        try:
            expires = None
            if expires_days:
                expires = datetime.datetime.now() + datetime.timedelta(days=expires_days)

            cookie = Cookie(
                name=name,
                value=value,
                domain=domain if domain else None,
                path=path,
                expires=expires,
                secure=secure,
                http_only=http_only,
                same_site=same_site if same_site else None,
            )

            # Add to our list (in a real app, this would set the cookie in WebView)
            # Remove existing cookie with same name
            self.cookies = [c for c in self.cookies if c.name != name]
            self.cookies.append(cookie)

            self.view.emit("cookie_created", {"name": name})
        except ValueError as e:
            self.view.emit("cookie_error", {"message": str(e)})

    def create_quick_cookie(self, type: str) -> None:
        """Create a quick cookie of a specific type."""
        self.cookie_counter += 1
        timestamp = datetime.datetime.now().strftime("%H%M%S")

        if type == "session":
            self.create_cookie(
                name=f"session_{self.cookie_counter}",
                value=f"sess_{timestamp}",
            )
        elif type == "persistent":
            self.create_cookie(
                name=f"remember_{self.cookie_counter}",
                value=f"rem_{timestamp}",
                expires_days=7,
            )
        elif type == "secure":
            self.create_cookie(
                name=f"secure_{self.cookie_counter}",
                value=f"sec_{timestamp}",
                secure=True,
                same_site="Strict",
            )
        elif type == "auth":
            self.create_cookie(
                name=f"auth_token_{self.cookie_counter}",
                value=f"auth_{timestamp}",
                http_only=True,
                secure=True,
                expires_days=1,
            )

    def get_cookies(self) -> None:
        """Get all cookies and send to frontend."""
        cookie_list = [c.to_dict() for c in self.cookies]
        self.view.emit("cookies_updated", {"cookies": cookie_list})

    def delete_cookie(self, name: str) -> None:
        """Delete a cookie by name."""
        self.cookies = [c for c in self.cookies if c.name != name]
        self.view.emit("cookie_deleted", {"name": name})

    def delete_all_cookies(self) -> None:
        """Delete all cookies."""
        self.cookies = []
        self.view.emit("cookies_updated", {"cookies": []})


def main():
    """Run the cookie management demo."""
    from auroraview import WebView

    view = WebView(
        html=HTML,
        title="Cookie Management Demo",
        width=950,
        height=750,
    )

    manager = CookieManager(view)

    @view.bind_call("api.create_cookie")
    def create_cookie(
        name: str,
        value: str,
        domain: str = None,
        path: str = "/",
        expires_days: int = None,
        same_site: str = None,
        secure: bool = False,
        http_only: bool = False,
    ):
        manager.create_cookie(
            name=name,
            value=value,
            domain=domain,
            path=path,
            expires_days=expires_days,
            same_site=same_site,
            secure=secure,
            http_only=http_only,
        )

    @view.bind_call("api.create_quick_cookie")
    def create_quick_cookie(type: str):
        manager.create_quick_cookie(type)

    @view.bind_call("api.get_cookies")
    def get_cookies():
        manager.get_cookies()

    @view.bind_call("api.delete_cookie")
    def delete_cookie(name: str):
        manager.delete_cookie(name)

    @view.bind_call("api.delete_all_cookies")
    def delete_all_cookies():
        manager.delete_all_cookies()

    view.show()


if __name__ == "__main__":
    main()
