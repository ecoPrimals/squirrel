// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use crate::error::{Result, PluginError};
use crate::plugins::interfaces::PluginState;
use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error};

// ... existing code ... 