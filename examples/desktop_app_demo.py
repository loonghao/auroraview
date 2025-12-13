"""
AuroraView Desktop Application Demo

Demonstrates desktop application capabilities:
- File dialogs (open, save, folder selection)
- File system operations (read, write, list)
- Shell commands and script execution
- Environment variables

This example shows how to build a desktop-like application with
full file system access and native dialogs.
"""

import json
import tempfile
from pathlib import Path

import auroraview


def create_demo_html():
    """Create the demo HTML interface."""
    return """
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AuroraView Desktop App Demo</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: #e4e4e4;
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1000px;
            margin: 0 auto;
        }
        h1 {
            text-align: center;
            margin-bottom: 30px;
            color: #00d4ff;
            font-size: 2em;
        }
        .section {
            background: rgba(255, 255, 255, 0.05);
            border-radius: 12px;
            padding: 20px;
            margin-bottom: 20px;
            border: 1px solid rgba(255, 255, 255, 0.1);
        }
        .section h2 {
            color: #00d4ff;
            margin-bottom: 15px;
            font-size: 1.2em;
            display: flex;
            align-items: center;
            gap: 10px;
        }
        .section h2::before {
            content: '';
            width: 4px;
            height: 20px;
            background: #00d4ff;
            border-radius: 2px;
        }
        .btn-group {
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
            margin-bottom: 15px;
        }
        button {
            background: linear-gradient(135deg, #0066cc 0%, #0099ff 100%);
            color: white;
            border: none;
            padding: 10px 20px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.3s ease;
        }
        button:hover {
            transform: translateY(-2px);
            box-shadow: 0 4px 15px rgba(0, 153, 255, 0.4);
        }
        button:active {
            transform: translateY(0);
        }
        button.secondary {
            background: linear-gradient(135deg, #444 0%, #666 100%);
        }
        button.success {
            background: linear-gradient(135deg, #00aa55 0%, #00cc66 100%);
        }
        button.warning {
            background: linear-gradient(135deg, #cc6600 0%, #ff8800 100%);
        }
        .output {
            background: #0d1117;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 15px;
            font-family: 'Consolas', 'Monaco', monospace;
            font-size: 13px;
            max-height: 200px;
            overflow-y: auto;
            white-space: pre-wrap;
            word-break: break-all;
        }
        .output.success { border-color: #00aa55; }
        .output.error { border-color: #ff4444; color: #ff6666; }
        .input-group {
            display: flex;
            gap: 10px;
            margin-bottom: 10px;
        }
        input[type="text"], textarea {
            flex: 1;
            background: #0d1117;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 10px 15px;
            color: #e4e4e4;
            font-size: 14px;
        }
        input[type="text"]:focus, textarea:focus {
            outline: none;
            border-color: #0099ff;
        }
        textarea {
            min-height: 100px;
            font-family: 'Consolas', 'Monaco', monospace;
            resize: vertical;
        }
        .grid-2 {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
        }
        @media (max-width: 768px) {
            .grid-2 { grid-template-columns: 1fr; }
        }
        .status {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 12px;
            font-size: 12px;
            margin-left: 10px;
        }
        .status.ready { background: #00aa55; }
        .status.loading { background: #cc6600; }
        .file-list {
            max-height: 150px;
            overflow-y: auto;
        }
        .file-item {
            padding: 8px 12px;
            border-bottom: 1px solid rgba(255,255,255,0.1);
            display: flex;
            justify-content: space-between;
            align-items: center;
        }
        .file-item:last-child { border-bottom: none; }
        .file-item .name { color: #00d4ff; }
        .file-item .size { color: #888; font-size: 12px; }
    </style>
</head>
<body>
    <div class="container">
        <h1>AuroraView Desktop App Demo</h1>
        
        <div class="grid-2">
            <!-- File Dialogs Section -->
            <div class="section">
                <h2>File Dialogs</h2>
                <div class="btn-group">
                    <button onclick="openFile()">Open File</button>
                    <button onclick="openFiles()">Open Multiple</button>
                    <button onclick="openFolder()">Open Folder</button>
                    <button onclick="saveFile()">Save File</button>
                </div>
                <div id="dialogOutput" class="output">Click a button to open a dialog...</div>
            </div>
            
            <!-- File Operations Section -->
            <div class="section">
                <h2>File Operations</h2>
                <div class="input-group">
                    <input type="text" id="filePath" placeholder="Enter file path...">
                    <button onclick="readFile()">Read</button>
                    <button onclick="checkExists()">Exists?</button>
                </div>
                <div class="input-group">
                    <input type="text" id="dirPath" placeholder="Enter directory path...">
                    <button onclick="listDir()">List Dir</button>
                </div>
                <div id="fileOutput" class="output">File operation results will appear here...</div>
            </div>
        </div>
        
        <!-- Write File Section -->
        <div class="section">
            <h2>Write File</h2>
            <div class="input-group">
                <input type="text" id="writeFilePath" placeholder="File path to write...">
                <button class="success" onclick="writeFile()">Write File</button>
                <button class="secondary" onclick="appendFile()">Append</button>
            </div>
            <textarea id="writeContent" placeholder="Content to write..."></textarea>
            <div id="writeOutput" class="output" style="margin-top: 10px;">Write results will appear here...</div>
        </div>
        
        <!-- Shell Commands Section -->
        <div class="section">
            <h2>Shell Commands & Scripts</h2>
            <div class="input-group">
                <input type="text" id="command" placeholder="Command (e.g., python, node, git)">
                <input type="text" id="args" placeholder="Arguments (comma separated)">
                <button class="warning" onclick="executeCommand()">Execute</button>
            </div>
            <div class="btn-group">
                <button class="secondary" onclick="runPythonScript()">Run Python Script</button>
                <button class="secondary" onclick="getSystemInfo()">System Info</button>
                <button class="secondary" onclick="whichCommand()">Which Command</button>
            </div>
            <div id="shellOutput" class="output">Shell command results will appear here...</div>
        </div>
        
        <!-- Environment Variables Section -->
        <div class="grid-2">
            <div class="section">
                <h2>Environment Variables</h2>
                <div class="input-group">
                    <input type="text" id="envName" placeholder="Variable name (e.g., PATH)">
                    <button onclick="getEnvVar()">Get</button>
                    <button class="secondary" onclick="getAllEnv()">Get All</button>
                </div>
                <div id="envOutput" class="output">Environment variable results...</div>
            </div>
            
            <div class="section">
                <h2>Open & Reveal</h2>
                <div class="input-group">
                    <input type="text" id="openPath" placeholder="Path or URL to open...">
                </div>
                <div class="btn-group">
                    <button onclick="openUrl()">Open URL</button>
                    <button onclick="openFilePath()">Open File</button>
                    <button onclick="showInFolder()">Show in Folder</button>
                </div>
                <div id="openOutput" class="output">Open results...</div>
            </div>
        </div>
        
        <!-- Message Dialogs Section -->
        <div class="section">
            <h2>Message Dialogs</h2>
            <div class="btn-group">
                <button onclick="showInfo()">Info</button>
                <button class="warning" onclick="showWarning()">Warning</button>
                <button style="background: #cc4444" onclick="showError()">Error</button>
                <button class="secondary" onclick="showConfirm()">Confirm</button>
                <button class="secondary" onclick="askQuestion()">Ask</button>
            </div>
            <div id="messageOutput" class="output">Message dialog results...</div>
        </div>
    </div>
    
    <script>
        // Wait for AuroraView to be ready
        window.addEventListener('auroraviewready', function() {
            console.log('[Demo] AuroraView ready');
        });
        
        function log(elementId, message, isError = false) {
            const el = document.getElementById(elementId);
            el.textContent = typeof message === 'object' ? JSON.stringify(message, null, 2) : message;
            el.className = 'output' + (isError ? ' error' : ' success');
        }
        
        // File Dialogs
        async function openFile() {
            try {
                const result = await auroraview.dialog.openFile({
                    title: 'Select a File',
                    filters: [
                        { name: 'Text Files', extensions: ['txt', 'md', 'json'] },
                        { name: 'Python Files', extensions: ['py'] },
                        { name: 'All Files', extensions: ['*'] }
                    ]
                });
                log('dialogOutput', result);
            } catch (e) {
                log('dialogOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function openFiles() {
            try {
                const result = await auroraview.dialog.openFiles({
                    title: 'Select Multiple Files'
                });
                log('dialogOutput', result);
            } catch (e) {
                log('dialogOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function openFolder() {
            try {
                const result = await auroraview.dialog.openFolder({
                    title: 'Select a Folder'
                });
                log('dialogOutput', result);
            } catch (e) {
                log('dialogOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function saveFile() {
            try {
                const result = await auroraview.dialog.saveFile({
                    title: 'Save File As',
                    defaultName: 'document.txt',
                    filters: [
                        { name: 'Text Files', extensions: ['txt'] },
                        { name: 'All Files', extensions: ['*'] }
                    ]
                });
                log('dialogOutput', result);
            } catch (e) {
                log('dialogOutput', 'Error: ' + e.message, true);
            }
        }
        
        // File Operations
        async function readFile() {
            const path = document.getElementById('filePath').value;
            if (!path) {
                log('fileOutput', 'Please enter a file path', true);
                return;
            }
            try {
                const content = await auroraview.fs.readFile(path);
                log('fileOutput', content);
            } catch (e) {
                log('fileOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function checkExists() {
            const path = document.getElementById('filePath').value;
            if (!path) {
                log('fileOutput', 'Please enter a path', true);
                return;
            }
            try {
                const exists = await auroraview.fs.exists(path);
                log('fileOutput', 'Exists: ' + exists);
            } catch (e) {
                log('fileOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function listDir() {
            const path = document.getElementById('dirPath').value;
            if (!path) {
                log('fileOutput', 'Please enter a directory path', true);
                return;
            }
            try {
                const entries = await auroraview.fs.readDir(path);
                log('fileOutput', entries);
            } catch (e) {
                log('fileOutput', 'Error: ' + e.message, true);
            }
        }
        
        // Write File
        async function writeFile() {
            const path = document.getElementById('writeFilePath').value;
            const content = document.getElementById('writeContent').value;
            if (!path) {
                log('writeOutput', 'Please enter a file path', true);
                return;
            }
            try {
                await auroraview.fs.writeFile(path, content);
                log('writeOutput', 'File written successfully to: ' + path);
            } catch (e) {
                log('writeOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function appendFile() {
            const path = document.getElementById('writeFilePath').value;
            const content = document.getElementById('writeContent').value;
            if (!path) {
                log('writeOutput', 'Please enter a file path', true);
                return;
            }
            try {
                await auroraview.fs.writeFile(path, content, true);
                log('writeOutput', 'Content appended to: ' + path);
            } catch (e) {
                log('writeOutput', 'Error: ' + e.message, true);
            }
        }
        
        // Shell Commands
        async function executeCommand() {
            const command = document.getElementById('command').value;
            const argsStr = document.getElementById('args').value;
            const args = argsStr ? argsStr.split(',').map(s => s.trim()) : [];
            
            if (!command) {
                log('shellOutput', 'Please enter a command', true);
                return;
            }
            try {
                const result = await auroraview.shell.execute(command, args);
                log('shellOutput', result);
            } catch (e) {
                log('shellOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function runPythonScript() {
            try {
                const result = await auroraview.shell.execute('python', ['-c', 'print("Hello from Python!")']);
                log('shellOutput', result);
            } catch (e) {
                log('shellOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function getSystemInfo() {
            try {
                let result;
                // Try Windows first
                try {
                    result = await auroraview.shell.execute('cmd', ['/c', 'ver']);
                } catch {
                    // Try Unix
                    result = await auroraview.shell.execute('uname', ['-a']);
                }
                log('shellOutput', result);
            } catch (e) {
                log('shellOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function whichCommand() {
            const command = document.getElementById('command').value || 'python';
            try {
                const path = await auroraview.shell.which(command);
                log('shellOutput', 'Path: ' + (path || 'Not found'));
            } catch (e) {
                log('shellOutput', 'Error: ' + e.message, true);
            }
        }
        
        // Environment Variables
        async function getEnvVar() {
            const name = document.getElementById('envName').value || 'PATH';
            try {
                const value = await auroraview.shell.getEnv(name);
                log('envOutput', name + ' = ' + (value || '(not set)'));
            } catch (e) {
                log('envOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function getAllEnv() {
            try {
                const env = await auroraview.shell.getEnvAll();
                log('envOutput', env);
            } catch (e) {
                log('envOutput', 'Error: ' + e.message, true);
            }
        }
        
        // Open & Reveal
        async function openUrl() {
            const path = document.getElementById('openPath').value || 'https://github.com';
            try {
                await auroraview.shell.open(path);
                log('openOutput', 'Opened: ' + path);
            } catch (e) {
                log('openOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function openFilePath() {
            const path = document.getElementById('openPath').value;
            if (!path) {
                log('openOutput', 'Please enter a file path', true);
                return;
            }
            try {
                await auroraview.shell.openPath(path);
                log('openOutput', 'Opened: ' + path);
            } catch (e) {
                log('openOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function showInFolder() {
            const path = document.getElementById('openPath').value;
            if (!path) {
                log('openOutput', 'Please enter a file path', true);
                return;
            }
            try {
                await auroraview.shell.showInFolder(path);
                log('openOutput', 'Revealed: ' + path);
            } catch (e) {
                log('openOutput', 'Error: ' + e.message, true);
            }
        }
        
        // Message Dialogs
        async function showInfo() {
            try {
                const result = await auroraview.dialog.info('This is an info message.', 'Information');
                log('messageOutput', result);
            } catch (e) {
                log('messageOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function showWarning() {
            try {
                const result = await auroraview.dialog.warning('This is a warning message.', 'Warning');
                log('messageOutput', result);
            } catch (e) {
                log('messageOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function showError() {
            try {
                const result = await auroraview.dialog.error('This is an error message.', 'Error');
                log('messageOutput', result);
            } catch (e) {
                log('messageOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function showConfirm() {
            try {
                const result = await auroraview.dialog.confirm({
                    title: 'Confirm Action',
                    message: 'Are you sure you want to proceed?'
                });
                log('messageOutput', result);
            } catch (e) {
                log('messageOutput', 'Error: ' + e.message, true);
            }
        }
        
        async function askQuestion() {
            try {
                const confirmed = await auroraview.dialog.ask('Do you want to save changes?', 'Save Changes');
                log('messageOutput', 'User confirmed: ' + confirmed);
            } catch (e) {
                log('messageOutput', 'Error: ' + e.message, true);
            }
        }
    </script>
</body>
</html>
"""


def main():
    """Run the desktop app demo."""
    # Create webview
    webview = auroraview.WebView(
        title="AuroraView Desktop App Demo",
        width=1100,
        height=900,
        html=create_demo_html(),
        debug=True,
    )

    print("Desktop App Demo")
    print("================")
    print("This demo showcases desktop application capabilities:")
    print("- File dialogs (open, save, folder selection)")
    print("- File system operations (read, write, list)")
    print("- Shell commands and script execution")
    print("- Environment variables")
    print()
    print("Starting webview...")

    webview.show()


if __name__ == "__main__":
    main()
