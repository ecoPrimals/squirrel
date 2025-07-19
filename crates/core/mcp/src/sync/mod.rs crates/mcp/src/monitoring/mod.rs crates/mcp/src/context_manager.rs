// sync/mod.rs
// Fix gRPC import
// use crate::generated::mcp::sync::{sync_service_client::SyncServiceClient, ...}; // OLD
use crate::generated::*; // Try importing wildcard from generated

// ... (other imports)

// Fix MCPError variants and unwrap_or call in conversion functions
fn state_change_to_proto(change: &StateChange) -> std::result::Result<ProtoContextChange, MCPError> {
    let data_bytes = serde_json::to_vec(&change.data.clone().unwrap_or(serde_json::Value::Null))
        .map_err(|e| MCPError::Serialization(/* ... */))?;
    let metadata_bytes = serde_json::to_vec(&change.metadata.clone().unwrap_or(serde_json::Value::Null))
        .map_err(|e| MCPError::Serialization(/* ... */))?;
    // ...
}
fn proto_to_state_change(proto: ProtoContextChange) -> std::result::Result<StateChange, MCPError> {
    let operation = match ProtoOperationType::try_from(...) {
        _ => return Err(MCPError::Deserialization(/* ... */))
    };
    // ... use Deserialization variant for other map_err/ok_or_else calls ...
}
impl TryFrom<i32> for ProtoOperationType {
    fn try_from(...) -> std::result::Result<Self, Self::Error> {
        match value {
             _ => Err(MCPError::Deserialization(/* ... */)),
        }
    }
}

impl MCPSync {
    async fn sync_internal(&self) -> std::result::Result<SyncResult, MCPError> {
        let _sync_lock = self.lock.lock().map_err(|_| MCPError::InternalError(/*...*/))?;
        // Explicitly type the async block result
        let sync_operation_result: std::result::Result<(), MCPError> = async {
            // Fix connect chain
            let endpoint = tonic::transport::Channel::from_shared(server_url.clone())
                .map_err(|e| MCPError::Configuration(/*...*/))?;
            let channel_result = timeout(timeout_duration, endpoint.connect()).await;
            let channel = match channel_result {
                Ok(Ok(ch)) => Ok(ch), 
                Ok(Err(e)) => Err(MCPError::NetworkError(/*...*/)), 
                Err(_) => Err(MCPError::NetworkError(/*...*/)),
            }?;
            let mut client = SyncServiceClient::new(channel); // Use generated client
            // Fix error variants in request/response logic
            let local_changes_to_send = self.state_manager.get_changes_since(...).await
                .map_err(|e| MCPError::InternalError(format!("Failed get local changes: {}", e)))?; // Use InternalError?
            // ...
            let response = client.sync(request).await
                .map_err(|e| MCPError::NetworkError(/*...*/))?;
            if !sync_response.success { return Err(MCPError::InternalError(/*...*/)); } 
            // ...
            if !apply_success { return Err(MCPError::InternalError(/*...*/)); }
            Ok(())
        }.await;
        // ...
        sync_operation_result?; 
        Ok(SyncResult { /* ... */ })
    }
}

// monitoring/mod.rs - Remove duplicate import
use crate::plugins::PluginStatus;
// use crate::sync::state::StateOperation; // Remove this line
use crate::tool::ToolState;

// context_manager.rs - Fix SyncConfig init (assuming default() is correct)
    pub async fn new() -> Self {
        let sync_config = SyncConfig::default(); // Ensure this is correct
        // ... create dependencies ...
        let sync = Arc::new(MCPSync::new(sync_config, persistence, monitor, state_manager));
        // ... create Self ...
    } 