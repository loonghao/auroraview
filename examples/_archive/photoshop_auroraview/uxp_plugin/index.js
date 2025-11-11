// Minimal UXP Plugin - Bridge to Python Backend
const { app } = require('photoshop');

let socket = null;
let statusEl = null;
let logEl = null;
let connectBtn = null;

function log(message) {
    if (!logEl) return;
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    entry.textContent = `[${new Date().toLocaleTimeString()}] ${message}`;
    logEl.appendChild(entry);
    logEl.scrollTop = logEl.scrollHeight;
    console.log(message);
}

function updateStatus(connected) {
    if (!statusEl || !connectBtn) return;
    statusEl.className = `status ${connected ? 'connected' : 'disconnected'}`;
    statusEl.textContent = connected ? 'Connected to Python' : 'Disconnected';
    connectBtn.textContent = connected ? 'Disconnect' : 'Connect to Python';
}

function connect() {
    if (socket) {
        socket.close();
        return;
    }
    
    log('Connecting to Python backend...');
    socket = new WebSocket('ws://localhost:9001');
    
    socket.onopen = () => {
        updateStatus(true);
        log('✅ Connected to Python backend');
        
        // Send handshake
        sendMessage('handshake', {
            client: 'photoshop',
            app: app.name,
            version: app.version
        });
    };
    
    socket.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            log(`📨 Received: ${data.action}`);
            handleMessage(data);
        } catch (error) {
            log(`❌ Error: ${error.message}`);
        }
    };
    
    socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        log(`❌ WebSocket error: ${error.message || 'Connection failed'}`);
        log(`   Check if Python backend is running on port 9001`);
    };

    socket.onclose = (event) => {
        updateStatus(false);
        const reason = event.reason || 'Unknown reason';
        const code = event.code || 'Unknown code';
        log(`🔌 Disconnected from Python (Code: ${code}, Reason: ${reason})`);
        socket = null;
    };
}

function sendMessage(action, data) {
    if (!socket || socket.readyState !== WebSocket.OPEN) {
        log('❌ Not connected');
        return;
    }
    
    const message = {
        type: 'request',
        action: action,
        data: data,
        timestamp: Date.now()
    };
    
    socket.send(JSON.stringify(message));
    log(`📤 Sent: ${action}`);
}

async function handleMessage(message) {
    const action = message.action;
    
    switch (action) {
        case 'handshake_ack':
            log('🤝 Handshake acknowledged');
            break;
            
        case 'execute_command':
            await executeCommand(message.data);
            break;
            
        default:
            log(`❓ Unknown action: ${action}`);
    }
}

async function executeCommand(data) {
    const { command, params } = data;
    log(`⚙️  Executing: ${command}`);

    try {
        switch (command) {
            case 'get_active_layer_image':
                await getActiveLayerImage();
                break;
            case 'create_layer':
                await createLayer(params);
                break;
            case 'get_layers':
                await getLayers();
                break;
            case 'delete_layer':
                await deleteLayer(params);
                break;
            case 'rename_layer':
                await renameLayer(params);
                break;
            case 'get_document_info':
                await getDocumentInfo();
                break;
            default:
                log(`❓ Unknown command: ${command}`);
        }
    } catch (error) {
        log(`❌ Command failed: ${error.message}`);
        sendMessage('command_error', {
            command: command,
            error: error.message
        });
    }
}

async function getActiveLayerImage() {
    try {
        const doc = app.activeDocument;
        const layer = doc.activeLayers[0];
        
        // Get layer bounds
        const bounds = layer.bounds;
        
        // For now, send layer info (actual image export would require more complex code)
        sendMessage('image_data', {
            layerName: layer.name,
            bounds: {
                left: bounds.left,
                top: bounds.top,
                right: bounds.right,
                bottom: bounds.bottom
            },
            // TODO: Add actual image data export
            image: null
        });
        
        log('📷 Layer info sent');
    } catch (error) {
        log(`❌ Error getting layer: ${error.message}`);
    }
}

async function createLayer(params) {
    try {
        await app.batchPlay([
            {
                _obj: 'make',
                _target: [{ _ref: 'layer' }],
                using: {
                    _obj: 'layer',
                    name: params.name || `Layer ${Date.now()}`
                }
            }
        ], {});

        const layer = app.activeDocument.activeLayers[0];

        sendMessage('layer_created', {
            name: layer.name,
            id: layer.id,
            bounds: {
                left: layer.bounds.left,
                top: layer.bounds.top,
                right: layer.bounds.right,
                bottom: layer.bounds.bottom
            }
        });

        log(`🎨 Layer created: ${layer.name}`);
    } catch (error) {
        log(`❌ Error creating layer: ${error.message}`);
    }
}

async function getLayers() {
    try {
        const doc = app.activeDocument;
        const layers = [];

        // Get all layers
        for (const layer of doc.layers) {
            layers.push({
                id: layer.id,
                name: layer.name,
                visible: layer.visible,
                opacity: layer.opacity,
                kind: layer.kind,
                bounds: {
                    left: layer.bounds.left,
                    top: layer.bounds.top,
                    right: layer.bounds.right,
                    bottom: layer.bounds.bottom
                }
            });
        }

        sendMessage('layers_list', {
            count: layers.length,
            layers: layers
        });

        log(`📋 Sent ${layers.length} layers`);
    } catch (error) {
        log(`❌ Error getting layers: ${error.message}`);
    }
}

async function deleteLayer(params) {
    try {
        const doc = app.activeDocument;
        const layer = doc.layers.find(l => l.id === params.id);

        if (!layer) {
            throw new Error(`Layer not found: ${params.id}`);
        }

        const layerName = layer.name;
        await layer.delete();

        sendMessage('layer_deleted', {
            id: params.id,
            name: layerName
        });

        log(`🗑️  Layer deleted: ${layerName}`);
    } catch (error) {
        log(`❌ Error deleting layer: ${error.message}`);
    }
}

async function renameLayer(params) {
    try {
        const doc = app.activeDocument;
        const layer = doc.layers.find(l => l.id === params.id);

        if (!layer) {
            throw new Error(`Layer not found: ${params.id}`);
        }

        const oldName = layer.name;
        layer.name = params.newName;

        sendMessage('layer_renamed', {
            id: params.id,
            oldName: oldName,
            newName: params.newName
        });

        log(`✏️  Layer renamed: ${oldName} → ${params.newName}`);
    } catch (error) {
        log(`❌ Error renaming layer: ${error.message}`);
    }
}

async function getDocumentInfo() {
    try {
        const doc = app.activeDocument;

        sendMessage('document_info', {
            name: doc.name,
            width: doc.width,
            height: doc.height,
            resolution: doc.resolution,
            colorMode: doc.mode,
            layerCount: doc.layers.length
        });

        log(`📄 Document info sent: ${doc.name}`);
    } catch (error) {
        log(`❌ Error getting document info: ${error.message}`);
    }
}

// Initialize function
function initialize() {
    // Get DOM elements
    statusEl = document.getElementById('status');
    logEl = document.getElementById('log');
    connectBtn = document.getElementById('connectBtn');

    // Check if elements exist
    if (!statusEl || !logEl || !connectBtn) {
        console.error('Failed to get DOM elements');
        console.log('statusEl:', statusEl);
        console.log('logEl:', logEl);
        console.log('connectBtn:', connectBtn);
        return;
    }

    // Event listeners
    connectBtn.addEventListener('click', connect);

    // Initialize
    log('AuroraView Bridge initialized');
    updateStatus(false);
}

// Try multiple initialization methods for UXP compatibility
if (document.readyState === 'loading') {
    // DOM is still loading
    document.addEventListener('DOMContentLoaded', initialize);
} else {
    // DOM is already loaded
    initialize();
}

