// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use crate::error::ClientError;

fn main() {
    // ... existing code ...
    .map_err(|e| MCPError::Client(ClientError::ApiRequestError(format!("Failed to add task: {}", e))).into())
    // ... existing code ...
    .map_err(|e| MCPError::Client(ClientError::ApiRequestError(format!("Failed to serialize task: {}", e))).into())?;
    // ... existing code ...
    .map_err(|e| MCPError::Client(ClientError::ApiRequestError(format!("Failed to deserialize task: {}", e))).into())?;
    // ... existing code ...
} 