// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

use crate::plugins::interfaces::{Plugin, PluginMetadata};
use crate::error::{Result, PluginError};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use libloading::{Library, Symbol};
use std::collections::HashMap;
use crate::plugins::cache::PluginCache;
use tracing::debug;

// ... existing code ... 