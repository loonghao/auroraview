// AuroraView WebSocket Server for Photoshop Integration
// This example demonstrates bidirectional communication between Photoshop UXP plugin and Rust

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{StreamExt, SinkExt, stream::TryStreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;
use serde::{Deserialize, Serialize};
use serde_json::Value;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<SocketAddr, Tx>>>;

#[derive(Debug, Serialize, Deserialize)]
struct WsMessage {
    #[serde(rename = "type")]
    msg_type: String,
    id: String,
    action: String,
    data: Value,
    timestamp: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    let addr = "127.0.0.1:9001";
    let listener = TcpListener::bind(addr).await?;
    println!("🚀 AuroraView WebSocket Server listening on: {}", addr);
    println!("📡 Waiting for Photoshop UXP plugin to connect...\n");

    // Shared state: map of peer addresses to their message channels
    let peer_map: PeerMap = Arc::new(Mutex::new(HashMap::new()));

    while let Ok((stream, addr)) = listener.accept().await {
        println!("✅ New connection from: {}", addr);
        tokio::spawn(handle_connection(peer_map.clone(), stream, addr));
    }

    Ok(())
}

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    // Accept WebSocket handshake
    let ws_stream = match accept_async(raw_stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("❌ WebSocket handshake error for {}: {}", addr, e);
            return;
        }
    };

    println!("🔗 WebSocket connection established with {}", addr);

    // Create channel for this peer to receive broadcasts
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(addr, tx);

    let (outgoing, incoming) = ws_stream.split();

    // Handle incoming messages from Photoshop
    let broadcast_incoming = incoming.try_for_each(|msg| {
        if let Message::Text(text) = &msg {
            println!("📨 Received from {}: {}", addr, text);
            
            // Parse and handle message
            if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(text) {
                handle_photoshop_message(&ws_msg, &peer_map, &addr);
            }
        }

        futures_util::future::ok(())
    });

    // Forward messages from other peers to this peer's outgoing stream
    let receive_from_others = rx.map(Ok).forward(outgoing);

    // Run both tasks concurrently
    futures_util::pin_mut!(broadcast_incoming, receive_from_others);
    futures_util::future::select(broadcast_incoming, receive_from_others).await;

    // Clean up when connection closes
    println!("🔌 {} disconnected", addr);
    peer_map.lock().unwrap().remove(&addr);
}

fn handle_photoshop_message(msg: &WsMessage, peer_map: &PeerMap, sender_addr: &SocketAddr) {
    match msg.action.as_str() {
        "handshake" => {
            println!("🤝 Handshake from Photoshop: {:?}", msg.data);
            
            // Send acknowledgment
            let response = WsMessage {
                msg_type: "response".to_string(),
                id: format!("ack_{}", msg.id),
                action: "handshake_ack".to_string(),
                data: serde_json::json!({
                    "server": "auroraview",
                    "version": "1.0.0",
                    "status": "connected"
                }),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            };
            
            send_to_peer(peer_map, sender_addr, &response);
        }
        
        "layer_created" => {
            println!("🎨 Layer created: {:?}", msg.data);
            
            // Broadcast to all other connected clients
            broadcast_message(peer_map, sender_addr, msg);
        }
        
        "selection_info" => {
            println!("🖱️  Selection info: {:?}", msg.data);
            
            // Process selection data
            // You can add custom logic here
        }
        
        "document_info" => {
            println!("📄 Document info: {:?}", msg.data);
            
            // Process document data
            // You can add custom logic here
        }
        
        "command_result" => {
            println!("✅ Command result: {:?}", msg.data);
        }
        
        _ => {
            println!("❓ Unknown action: {}", msg.action);
        }
    }
}

fn send_to_peer(peer_map: &PeerMap, addr: &SocketAddr, msg: &WsMessage) {
    let peers = peer_map.lock().unwrap();
    
    if let Some(tx) = peers.get(addr) {
        let json = serde_json::to_string(msg).unwrap();
        let _ = tx.unbounded_send(Message::Text(json));
        println!("📤 Sent to {}: {}", addr, msg.action);
    }
}

fn broadcast_message(peer_map: &PeerMap, sender_addr: &SocketAddr, msg: &WsMessage) {
    let peers = peer_map.lock().unwrap();
    let json = serde_json::to_string(msg).unwrap();
    
    for (addr, tx) in peers.iter() {
        if addr != sender_addr {
            let _ = tx.unbounded_send(Message::Text(json.clone()));
            println!("📡 Broadcast to {}: {}", addr, msg.action);
        }
    }
}

// Example: Send command to Photoshop
#[allow(dead_code)]
fn send_command_to_photoshop(peer_map: &PeerMap, addr: &SocketAddr, command: &str, params: Value) {
    let msg = WsMessage {
        msg_type: "request".to_string(),
        id: format!("cmd_{}", chrono::Utc::now().timestamp_millis()),
        action: "execute_command".to_string(),
        data: serde_json::json!({
            "command": command,
            "params": params
        }),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };
    
    send_to_peer(peer_map, addr, &msg);
}

