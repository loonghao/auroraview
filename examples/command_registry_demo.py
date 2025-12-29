"""Command Registry Demo - AuroraView Command System.

This example demonstrates the CommandRegistry system for defining
Python functions callable from JavaScript, inspired by Tauri's #[command] macro.

Usage:
    python examples/command_registry_demo.py

Features demonstrated:
    - CommandRegistry for centralized command management
    - @commands.register decorator patterns
    - Command error handling with CommandError
    - Async command support
    - Command listing and introspection
    - Direct Python invocation of commands

JavaScript side:
    // Invoke commands
    const result = await auroraview.invoke("greet", {name: "Alice"});
    const sum = await auroraview.invoke("calculate", {a: 5, b: 3, op: "add"});
"""

from __future__ import annotations

from typing import Any, Dict, List

from auroraview import WebView
from auroraview.core.commands import CommandError, CommandErrorCode, CommandRegistry


def main():
    """Run the command registry demo."""
    html_content = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Command Registry Demo</title>
        <style>
            body {
                font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                max-width: 900px;
                margin: 50px auto;
                padding: 20px;
                background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
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
            h3 { color: #666; margin-bottom: 10px; }
            button {
                background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
                color: white;
                border: none;
                padding: 12px 24px;
                border-radius: 6px;
                cursor: pointer;
                font-size: 14px;
                margin: 5px;
                transition: transform 0.1s, box-shadow 0.1s;
            }
            button:hover {
                transform: translateY(-2px);
                box-shadow: 0 5px 15px rgba(240, 147, 251, 0.4);
            }
            button:active { transform: translateY(0); }
            .input-group {
                display: flex;
                gap: 10px;
                margin: 10px 0;
                flex-wrap: wrap;
            }
            input, select {
                padding: 10px 15px;
                border: 2px solid #e0e0e0;
                border-radius: 6px;
                font-size: 14px;
                transition: border-color 0.2s;
            }
            input:focus, select:focus {
                outline: none;
                border-color: #f093fb;
            }
            #output {
                background: #1e1e1e;
                color: #f8f8f2;
                border-radius: 8px;
                padding: 16px;
                font-family: 'Consolas', monospace;
                font-size: 13px;
                max-height: 300px;
                overflow-y: auto;
                white-space: pre-wrap;
            }
            .success { color: #50fa7b; }
            .error { color: #ff5555; }
            .info { color: #8be9fd; }
            .command-list {
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
                gap: 10px;
                margin-top: 15px;
            }
            .command-item {
                background: #f5f5f5;
                padding: 12px;
                border-radius: 8px;
                font-family: monospace;
                font-size: 13px;
            }
            .command-name { font-weight: bold; color: #f5576c; }
            .command-params { color: #666; font-size: 11px; }
        </style>
    </head>
    <body>
        <div class="card">
            <h1>Command Registry Demo</h1>
            <p>Demonstrates the Tauri-inspired command system for Python-JavaScript communication.</p>

            <h3>Greet Command</h3>
            <div class="input-group">
                <input type="text" id="greetName" placeholder="Enter name" value="World">
                <button onclick="invokeGreet()">Greet</button>
            </div>

            <h3>Calculator Command</h3>
            <div class="input-group">
                <input type="number" id="calcA" placeholder="A" value="10" style="width: 80px">
                <select id="calcOp">
                    <option value="add">+</option>
                    <option value="subtract">-</option>
                    <option value="multiply">*</option>
                    <option value="divide">/</option>
                </select>
                <input type="number" id="calcB" placeholder="B" value="5" style="width: 80px">
                <button onclick="invokeCalculate()">Calculate</button>
            </div>

            <h3>Data Operations</h3>
            <div class="input-group">
                <button onclick="invokeGetUsers()">Get Users</button>
                <button onclick="invokeAddUser()">Add User</button>
                <button onclick="invokeValidateEmail()">Validate Email</button>
                <button onclick="invokeErrorDemo()">Error Demo</button>
            </div>

            <h3>Command Introspection</h3>
            <div class="input-group">
                <button onclick="listCommands()">List All Commands</button>
            </div>
        </div>

        <div class="card">
            <h3>Output</h3>
            <div id="output">Ready to invoke commands...</div>
        </div>

        <div class="card">
            <h3>Registered Commands</h3>
            <div id="commandList" class="command-list">Loading...</div>
        </div>

        <script>
            function log(msg, type = 'info') {
                const output = document.getElementById('output');
                const timestamp = new Date().toLocaleTimeString();
                const formatted = typeof msg === 'object' ? JSON.stringify(msg, null, 2) : msg;
                output.innerHTML = `<span class="${type}">[${timestamp}] ${formatted}</span>`;
            }

            async function invokeGreet() {
                const name = document.getElementById('greetName').value;
                try {
                    // Using the invoke pattern (Tauri-style)
                    const result = await auroraview.api.greet({name});
                    log(result, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function invokeCalculate() {
                const a = parseFloat(document.getElementById('calcA').value);
                const b = parseFloat(document.getElementById('calcB').value);
                const op = document.getElementById('calcOp').value;
                try {
                    const result = await auroraview.api.calculate({a, b, op});
                    log(`Result: ${a} ${op} ${b} = ${result}`, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function invokeGetUsers() {
                try {
                    const result = await auroraview.api.get_users();
                    log(result, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function invokeAddUser() {
                try {
                    const result = await auroraview.api.add_user({
                        name: "New User",
                        email: "new@example.com"
                    });
                    log(result, 'success');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function invokeValidateEmail() {
                const email = prompt("Enter email to validate:", "test@example.com");
                if (!email) return;
                try {
                    const result = await auroraview.api.validate_email({email});
                    log(result, result.valid ? 'success' : 'error');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            async function invokeErrorDemo() {
                try {
                    await auroraview.api.error_demo();
                } catch (e) {
                    log(`Caught error: ${JSON.stringify(e)}`, 'error');
                }
            }

            async function listCommands() {
                try {
                    const result = await auroraview.api.list_commands();
                    log(result, 'info');

                    // Update command list display
                    const listEl = document.getElementById('commandList');
                    listEl.innerHTML = result.commands.map(cmd => `
                        <div class="command-item">
                            <div class="command-name">${cmd}</div>
                        </div>
                    `).join('');
                } catch (e) {
                    log(`Error: ${e.message}`, 'error');
                }
            }

            // Load commands on startup
            setTimeout(listCommands, 500);
        </script>
    </body>
    </html>
    """

    view = WebView(title="Command Registry Demo", html=html_content, width=950, height=850)

    # Create a CommandRegistry instance
    commands = CommandRegistry()

    # In-memory data store for demo
    users_db: List[Dict[str, Any]] = [
        {"id": 1, "name": "Alice", "email": "alice@example.com"},
        {"id": 2, "name": "Bob", "email": "bob@example.com"},
    ]

    # ─────────────────────────────────────────────────────────────────
    # Register commands using different patterns
    # ─────────────────────────────────────────────────────────────────

    # Pattern 1: Simple decorator
    @commands.register
    def greet(name: str = "World") -> str:
        """Greet someone by name."""
        return f"Hello, {name}! Welcome to AuroraView."

    # Pattern 2: Decorator with custom name
    @commands.register("calculate")
    def do_calculation(a: float, b: float, op: str = "add") -> float:
        """Perform a calculation."""
        operations = {
            "add": lambda x, y: x + y,
            "subtract": lambda x, y: x - y,
            "multiply": lambda x, y: x * y,
            "divide": lambda x, y: x / y if y != 0 else float("inf"),
        }
        if op not in operations:
            raise CommandError(
                CommandErrorCode.INVALID_ARGUMENTS,
                f"Unknown operation: {op}",
                {"valid_operations": list(operations.keys())},
            )
        return operations[op](a, b)

    # Pattern 3: Data access commands
    @commands.register
    def get_users() -> Dict[str, Any]:
        """Get all users from the database."""
        return {"users": users_db, "count": len(users_db)}

    @commands.register
    def add_user(name: str, email: str) -> Dict[str, Any]:
        """Add a new user to the database."""
        new_id = max(u["id"] for u in users_db) + 1 if users_db else 1
        new_user = {"id": new_id, "name": name, "email": email}
        users_db.append(new_user)
        return {"ok": True, "user": new_user, "message": f"User {name} added successfully"}

    # Pattern 4: Validation command with error handling
    @commands.register
    def validate_email(email: str) -> Dict[str, Any]:
        """Validate an email address."""
        import re

        pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"
        is_valid = bool(re.match(pattern, email))
        return {
            "email": email,
            "valid": is_valid,
            "message": "Valid email" if is_valid else "Invalid email format",
        }

    # Pattern 5: Command that raises errors
    @commands.register
    def error_demo() -> None:
        """Demonstrate error handling."""
        raise CommandError(
            CommandErrorCode.PERMISSION_DENIED,
            "This is a demo error to show error handling",
            {"demo": True, "hint": "This error was intentionally raised"},
        )

    # Pattern 6: Introspection command
    @commands.register
    def list_commands() -> Dict[str, Any]:
        """List all registered commands."""
        return {"commands": commands.list_commands(), "count": len(commands)}

    # ─────────────────────────────────────────────────────────────────
    # Bind commands to WebView using bind_call
    # ─────────────────────────────────────────────────────────────────

    # Expose all registered commands via the API
    for cmd_name in commands.list_commands():

        def make_handler(name):
            def handler(**kwargs):
                return commands.invoke(name, **kwargs)

            return handler

        view.bind_call(f"api.{cmd_name}", make_handler(cmd_name))

    print("Starting Command Registry Demo...")
    print(f"Registered commands: {commands.list_commands()}")
    print(f"Total commands: {len(commands)}")

    # Demonstrate direct Python invocation
    print("\nDirect Python invocation test:")
    print(f"  greet('Test') = {commands.invoke('greet', name='Test')}")
    print(f"  calculate(10, 5, 'add') = {commands.invoke('calculate', a=10, b=5, op='add')}")

    view.show()


if __name__ == "__main__":
    main()
