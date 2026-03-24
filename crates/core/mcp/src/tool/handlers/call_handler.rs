// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use std::sync::Arc;

pub struct CallHandler {
    tool_manager: Arc<dyn ToolManager>,
}

impl CallHandler {
    pub fn new(tool_manager: Arc<dyn ToolManager>) -> Self {
        Self { tool_manager }
    }
} 