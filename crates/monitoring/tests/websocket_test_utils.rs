use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::protocol::Message;
use serde_json::{json, Value};
use tracing::{info, warn, debug};

type WsStream = WebSocketStream<TcpStream>;

/// Mock WebSocket server for testing
pub struct MockWebSocketServer {
    address: SocketAddr,
    shutdown_tx: Option<oneshot::Sender<()>>,
    mock_data: Arc<Mutex<Vec<Value>>>,
}

impl MockWebSocketServer {
    /// Create a new mock server
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Bind to localhost with a random port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let address = listener.local_addr()?;
        
        let mock_data = Arc::new(Mutex::new(vec![
            json!({
                "componentId": "system_cpu",
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "data": {
                    "usage": 45.7,
                    "cores": 8,
                    "processes": 120
                }
            }),
            json!({
                "componentId": "system_memory",
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "data": {
                    "total": 16384,
                    "used": 8192,
                    "free": 8192
                }
            }),
            json!({
                "componentId": "network_traffic",
                "timestamp": chrono::Utc::now().timestamp_millis(),
                "data": {
                    "rx_bytes": 1500000,
                    "tx_bytes": 500000,
                    "connections": 42
                }
            })
        ]));
        
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        
        let mock_data_clone = mock_data.clone();
        tokio::spawn(async move {
            Self::run_server(listener, mock_data_clone, shutdown_rx).await;
        });
        
        Ok(Self {
            address,
            shutdown_tx: Some(shutdown_tx),
            mock_data,
        })
    }
    
    /// Get the server address
    pub fn address(&self) -> String {
        format!("127.0.0.1:{}", self.address.port())
    }
    
    /// Add mock data to the server
    pub async fn add_mock_data(&self, data: Value) {
        let mut mock_data = self.mock_data.lock().await;
        mock_data.push(data);
    }
    
    /// Run the mock server
    async fn run_server(
        listener: TcpListener,
        mock_data: Arc<Mutex<Vec<Value>>>,
        mut shutdown_rx: oneshot::Receiver<()>
    ) {
        info!("Mock WebSocket server running on {:?}", listener.local_addr().unwrap());
        
        // Track active connections with proper type annotation
        let connections = Arc::new(Mutex::new(Vec::<mpsc::Sender<()>>::new()));
        
        loop {
            tokio::select! {
                // Accept new connections
                Ok((stream, _)) = listener.accept() => {
                    let peer = stream.peer_addr().unwrap();
                    debug!("New WebSocket connection from {}", peer);
                    
                    let mock_data = mock_data.clone();
                    let connections = connections.clone();
                    
                    let (conn_tx, _) = mpsc::channel(1);
                    
                    // Store connection sender
                    {
                        let mut conns = connections.lock().await;
                        conns.push(conn_tx);
                    }
                    
                    // Handle connection
                    tokio::spawn(async move {
                        if let Ok(ws_stream) = accept_async(stream).await {
                            if let Err(e) = Self::handle_connection(ws_stream, mock_data).await {
                                warn!("Error handling WebSocket connection from {}: {}", peer, e);
                            }
                        }
                    });
                }
                
                // Check for shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Mock WebSocket server shutting down");
                    break;
                }
            }
        }
    }
    
    /// Handle a WebSocket connection
    async fn handle_connection(
        mut ws_stream: WsStream,
        mock_data: Arc<Mutex<Vec<Value>>>
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Track subscribed components
        let mut subscribed = Vec::new();
        
        // Process incoming messages
        while let Some(result) = ws_stream.next().await {
            match result {
                Ok(Message::Text(text)) => {
                    if let Ok(msg) = serde_json::from_str::<Value>(&text) {
                        if let Some(msg_type) = msg["type"].as_str() {
                            match msg_type {
                                "subscribe" => {
                                    if let Some(component_id) = msg["componentId"].as_str() {
                                        debug!("Client subscribed to component: {}", component_id);
                                        subscribed.push(component_id.to_string());
                                        
                                        // Send initial data for this component
                                        let mock_data = mock_data.lock().await;
                                        for data in mock_data.iter() {
                                            if let Some(id) = data["componentId"].as_str() {
                                                if id == component_id {
                                                    ws_stream.send(Message::Text(data.to_string())).await?;
                                                }
                                            }
                                        }
                                    }
                                }
                                "request_batch" => {
                                    debug!("Client requested batch data");
                                    let response = json!({
                                        "type": "batch",
                                        "timestamp": chrono::Utc::now().timestamp_millis(),
                                        "components": subscribed
                                    });
                                    
                                    // Send batch response
                                    ws_stream.send(Message::Text(response.to_string())).await?;
                                    
                                    // Check if we should send compressed data
                                    if let Some(components) = msg["components"].as_array() {
                                        if components.len() > 5 {
                                            // Send compressed response (simulated)
                                            let compressed = json!({
                                                "type": "compressed",
                                                "compressed": true,
                                                "compressed_data": "SGVsbG8gV29ybGQ=", // Base64 "Hello World"
                                                "original_size": 1024,
                                                "compressed_size": 256
                                            });
                                            ws_stream.send(Message::Text(compressed.to_string())).await?;
                                        }
                                    }
                                }
                                "ping" => {
                                    // Send pong
                                    ws_stream.send(Message::Text(json!({"type": "pong"}).to_string())).await?;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Message::Ping(data)) => {
                    ws_stream.send(Message::Pong(data)).await?;
                }
                Ok(Message::Close(_)) => break,
                _ => {}
            }
        }
        
        Ok(())
    }
}

impl Drop for MockWebSocketServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Try to connect to the main server or fall back to a mock server
pub async fn get_test_server_addr(
    main_server: &str,
    timeout_duration: Duration
) -> (String, Option<MockWebSocketServer>) {
    use tokio::time::timeout;
    use std::net::ToSocketAddrs;
    
    // First try to resolve the main server address
    let addr_resolved = main_server.to_socket_addrs().map(|mut addrs| addrs.next().is_some()).unwrap_or(false);
    
    if addr_resolved {
        // Try to connect to the main server with timeout
        let connection_result = timeout(
            timeout_duration,
            tokio_tungstenite::connect_async(format!("ws://{}/ws", main_server))
        ).await;
        
        // If connection was successful, return the main server address
        if connection_result.is_ok() {
            if let Ok((_, _)) = connection_result.unwrap() {
                info!("Using main server at {}", main_server);
                return (main_server.to_string(), None);
            }
        }
    }
    
    // Fall back to mock server
    warn!("Main server not available, falling back to mock server");
    match MockWebSocketServer::new().await {
        Ok(mock_server) => {
            let addr = mock_server.address();
            info!("Started mock server at {}", addr);
            (addr, Some(mock_server))
        },
        Err(e) => {
            panic!("Failed to start mock server: {}", e);
        }
    }
} 