// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;

pub struct ListHandler {
    tool_manager: Arc<dyn ToolManager>,
}

impl ListHandler {
    pub fn new(tool_manager: Arc<dyn ToolManager>) -> Self {
        Self { tool_manager }
    }
} 