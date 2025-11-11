// Import Photoshop APIs
const { app } = require('photoshop');
const { action } = require('photoshop');

// WebSocket connection
let socket = null;
let reconnectTimer = null;
let messageId = 0;

// DOM elements
const statusEl = document.getElementById('status');
const logEl = document.getElementById('log');
const connectBtn = document.getElementById('connectBtn');
const disconnectBtn = document.getElementById('disconnectBtn');
const serverUrlInput = document.getElementById('serverUrl');
const createLayerBtn = document.getElementById('createLayerBtn');
const getSelectionBtn = document.getElementById('getSelectionBtn');
const getDocInfoBtn = document.getElementById('getDocInfoBtn');

// Logging utility
function log(message, type = 'info') {
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logEl.appendChild(entry);
    logEl.scrollTop = logEl.scrollHeight;
    console.log(message);
}

// Update connection status
function updateStatus(status) {
    statusEl.className = `status ${status}`;
    statusEl.textContent = status.charAt(0).toUpperCase() + status.slice(1);
    
    const isConnected = status === 'connected';
    connectBtn.disabled = isConnected;
    disconnectBtn.disabled = !isConnected;
    createLayerBtn.disabled = !isConnected;
    getSelectionBtn.disabled = !isConnected;
    getDocInfoBtn.disabled = !isConnected;
}

// Send message to server
function sendMessage(action, data = {}) {
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        log('Not connected to server', 'error');
        return;
    }
    
    const message = {
        type: 'request',
        id: `msg_${++messageId}`,
        action: action,
        data: data,
        timestamp: Date.now()
    };
    
    socket.send(JSON.stringify(message));
    log(`Sent: ${action}`, 'info');
}

// Connect to WebSocket server
function connect() {
    const url = serverUrlInput.value.trim();
    
    if (!url) {
        log('Please enter server URL', 'error');
        return;
    }
    
    updateStatus('connecting');
    log(`Connecting to ${url}...`);
    
    try {
        socket = new WebSocket(url);
        
        socket.onopen = () => {
            updateStatus('connected');
            log('Connected to AuroraView server', 'success');
            
            // Send initial handshake
            sendMessage('handshake', {
                client: 'photoshop',
                version: '1.0.0',
                app: app.name,
                appVersion: app.version
            });
        };
        
        socket.onmessage = (event) => {
            try {
                const message = JSON.parse(event.data);
                log(`Received: ${message.action || message.type}`, 'success');
                handleMessage(message);
            } catch (error) {
                log(`Failed to parse message: ${error.message}`, 'error');
            }
        };
        
        socket.onerror = (error) => {
            log(`WebSocket error: ${error}`, 'error');
        };
        
        socket.onclose = () => {
            updateStatus('disconnected');
            log('Disconnected from server', 'error');
            socket = null;
            
            // Auto-reconnect after 5 seconds
            if (reconnectTimer) clearTimeout(reconnectTimer);
            reconnectTimer = setTimeout(() => {
                log('Attempting to reconnect...');
                connect();
            }, 5000);
        };
        
    } catch (error) {
        log(`Connection failed: ${error.message}`, 'error');
        updateStatus('disconnected');
    }
}

// Disconnect from server
function disconnect() {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
    
    if (socket) {
        socket.close();
        socket = null;
    }
    
    updateStatus('disconnected');
    log('Disconnected');
}

// Handle incoming messages
function handleMessage(message) {
    switch (message.action) {
        case 'handshake_ack':
            log('Handshake acknowledged', 'success');
            break;
            
        case 'execute_command':
            executePhotoshopCommand(message.data);
            break;
            
        default:
            log(`Unknown action: ${message.action}`);
    }
}

// Execute Photoshop commands from server
async function executePhotoshopCommand(data) {
    try {
        const { command, params } = data;
        log(`Executing: ${command}`);
        
        // Execute command based on type
        // This is a placeholder - actual implementation would use Photoshop APIs
        
        sendMessage('command_result', {
            command: command,
            success: true,
            result: 'Command executed successfully'
        });
        
    } catch (error) {
        log(`Command failed: ${error.message}`, 'error');
        sendMessage('command_result', {
            command: data.command,
            success: false,
            error: error.message
        });
    }
}

// Create a new layer
async function createLayer() {
    try {
        await app.batchPlay([
            {
                _obj: 'make',
                _target: [{ _ref: 'layer' }],
                using: {
                    _obj: 'layer',
                    name: `AuroraView Layer ${Date.now()}`
                }
            }
        ], {});
        
        log('Layer created successfully', 'success');
        
        // Send layer info to server
        const layerInfo = {
            name: app.activeDocument.activeLayers[0].name,
            id: app.activeDocument.activeLayers[0].id
        };
        
        sendMessage('layer_created', layerInfo);
        
    } catch (error) {
        log(`Failed to create layer: ${error.message}`, 'error');
    }
}

// Get selection information
async function getSelection() {
    try {
        const doc = app.activeDocument;
        const selection = doc.selection;
        
        const selectionInfo = {
            hasSelection: selection.bounds !== undefined,
            bounds: selection.bounds || null,
            documentName: doc.name,
            documentWidth: doc.width,
            documentHeight: doc.height
        };
        
        log(`Selection info retrieved`, 'success');
        sendMessage('selection_info', selectionInfo);
        
    } catch (error) {
        log(`Failed to get selection: ${error.message}`, 'error');
    }
}

// Get document information
async function getDocumentInfo() {
    try {
        const doc = app.activeDocument;
        
        const docInfo = {
            name: doc.name,
            width: doc.width,
            height: doc.height,
            resolution: doc.resolution,
            colorMode: doc.mode,
            layerCount: doc.layers.length,
            activeLayer: doc.activeLayers[0]?.name || 'None'
        };
        
        log('Document info retrieved', 'success');
        sendMessage('document_info', docInfo);
        
    } catch (error) {
        log(`Failed to get document info: ${error.message}`, 'error');
    }
}

// Event listeners
connectBtn.addEventListener('click', connect);
disconnectBtn.addEventListener('click', disconnect);
createLayerBtn.addEventListener('click', createLayer);
getSelectionBtn.addEventListener('click', getSelection);
getDocInfoBtn.addEventListener('click', getDocumentInfo);

// Initialize
log('AuroraView Bridge initialized');
updateStatus('disconnected');

