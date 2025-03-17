use std::sync::Arc;
use std::time::Duration;
use axum::{
    routing::{get, post},
    Router,
    extract::{State, Path, WebSocketUpgrade},
    response::IntoResponse,
    Json,
};
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use super::{
    DashboardManager,
    DashboardLayout,
    DashboardComponent,
    DashboardUpdate,
    DashboardError,
};

/// Server state
#[derive(Clone)]
struct ServerState {
    /// Dashboard manager
    manager: Arc<DashboardManager>,
    /// Update channel
    update_tx: broadcast::Sender<DashboardUpdate>,
}

/// Start the dashboard HTTP server
pub async fn start_server(
    manager: Arc<DashboardManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create update channel
    let (update_tx, _) = broadcast::channel(100);

    // Create server state
    let state = ServerState {
        manager,
        update_tx: update_tx.clone(),
    };

    // Create router
    let app = Router::new()
        .route("/api/layouts", get(get_layouts))
        .route("/api/layouts/:id", get(get_layout))
        .route("/api/layouts", post(create_layout))
        .route("/api/layouts/:id", post(update_layout))
        .route("/api/layouts/:id/delete", post(delete_layout))
        .route("/api/components/:id/data", get(get_component_data))
        .route("/api/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    // Get server configuration
    let config = manager.get_config().await;
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], config.port));

    // Start server
    tracing::info!("Starting dashboard server on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Get all dashboard layouts
async fn get_layouts(
    State(state): State<ServerState>,
) -> Result<Json<Vec<DashboardLayout>>, DashboardError> {
    let layouts = state.manager.get_layouts().await?;
    Ok(Json(layouts))
}

/// Get a specific dashboard layout
async fn get_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<DashboardLayout>, DashboardError> {
    let layouts = state.manager.get_layouts().await?;
    let layout = layouts.iter()
        .find(|l| l.id == id)
        .ok_or_else(|| DashboardError::ComponentError("Layout not found".to_string()))?;
    Ok(Json(layout.clone()))
}

/// Create a new dashboard layout
async fn create_layout(
    State(state): State<ServerState>,
    Json(layout): Json<DashboardLayout>,
) -> Result<Json<DashboardLayout>, DashboardError> {
    state.manager.add_layout(layout.clone()).await?;
    Ok(Json(layout))
}

/// Update a dashboard layout
async fn update_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
    Json(layout): Json<DashboardLayout>,
) -> Result<Json<DashboardLayout>, DashboardError> {
    if layout.id != id {
        return Err(DashboardError::ConfigError("Layout ID mismatch".to_string()));
    }
    state.manager.remove_layout(&id).await?;
    state.manager.add_layout(layout.clone()).await?;
    Ok(Json(layout))
}

/// Delete a dashboard layout
async fn delete_layout(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<(), DashboardError> {
    state.manager.remove_layout(&id).await
}

/// Get component data
async fn get_component_data(
    State(state): State<ServerState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, DashboardError> {
    // Find component in layouts
    let layouts = state.manager.get_layouts().await?;
    let component = layouts.iter()
        .flat_map(|l| l.components.iter())
        .find(|c| match c {
            DashboardComponent::MetricChart { id: cid, .. } |
            DashboardComponent::HealthPanel { id: cid, .. } |
            DashboardComponent::AlertList { id: cid, .. } |
            DashboardComponent::ResourceGauge { id: cid, .. } |
            DashboardComponent::PerformanceGraph { id: cid, .. } => *cid == id,
        })
        .ok_or_else(|| DashboardError::ComponentError("Component not found".to_string()))?;

    // Get component data
    let data = state.manager.get_component_data(component).await?;
    Ok(Json(data))
}

/// WebSocket connection handler
async fn websocket_handler(
    State(state): State<ServerState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(
    socket: axum::extract::ws::WebSocket,
    state: ServerState,
) {
    let (mut sender, mut receiver) = socket.split();
    let mut update_rx = state.update_tx.subscribe();

    // Spawn task to send updates
    let send_task = tokio::spawn(async move {
        while let Ok(update) = update_rx.recv().await {
            if let Ok(msg) = serde_json::to_string(&update) {
                if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn task to receive messages
    let recv_task = tokio::spawn(async move {
        while let Some(Ok(_)) = receiver.next().await {
            // Handle client messages if needed
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = send_task => {}
        _ = recv_task => {}
    }
}

/// Start background update task
pub async fn start_update_task(
    manager: Arc<DashboardManager>,
    update_tx: broadcast::Sender<DashboardUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    let update_interval = {
        let config = manager.get_config().await;
        Duration::from_secs(config.refresh_interval)
    };

    tokio::spawn(async move {
        loop {
            // Get all layouts and components
            if let Ok(layouts) = manager.get_layouts().await {
                for layout in layouts {
                    for component in layout.components {
                        // Get component data
                        if let Ok(data) = manager.get_component_data(&component).await {
                            let component_id = match component {
                                DashboardComponent::MetricChart { id, .. } |
                                DashboardComponent::HealthPanel { id, .. } |
                                DashboardComponent::AlertList { id, .. } |
                                DashboardComponent::ResourceGauge { id, .. } |
                                DashboardComponent::PerformanceGraph { id, .. } => id,
                            };

                            // Send update
                            let update = DashboardUpdate {
                                component_id,
                                timestamp: time::OffsetDateTime::now_utc(),
                                data,
                            };
                            let _ = update_tx.send(update);
                        }
                    }
                }
            }

            tokio::time::sleep(update_interval).await;
        }
    });

    Ok(())
} 