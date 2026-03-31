// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Batch processor for bulk plugin operations.

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};
use tokio::time::MissedTickBehavior;
use tracing::{debug, info};

use crate::errors::{PluginError, Result};
use crate::zero_copy::{ZeroCopyPlugin, ZeroCopyPluginEntry};

use super::config::BatchProcessingConfig;
use super::types::{BatchOperation, BatchStatistics};

/// Estimated milliseconds saved per additional plugin when batching vs sequential loads.
const ESTIMATED_MS_SAVED_PER_EXTRA_PLUGIN: u64 = 2;

/// Batch processor for bulk operations
#[derive(Debug)]
pub struct BatchProcessor {
    /// Batch processing task handle (heartbeat; shared queue reserved for future deferred ops).
    task_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,

    /// Batch statistics
    stats: Arc<RwLock<BatchStatistics>>,

    /// Configuration for batch processing
    config: BatchProcessingConfig,

    /// Maximum plugin load operations allowed in one bounded queue (one `batch_load_plugins` call).
    max_queue_capacity: usize,
}

impl BatchProcessor {
    pub(super) fn new(config: BatchProcessingConfig) -> Self {
        let max_queue_capacity = config.max_batch_size.saturating_mul(128).max(1024);
        Self {
            task_handle: Arc::new(Mutex::new(None)),
            stats: Arc::new(RwLock::new(BatchStatistics::default())),
            config,
            max_queue_capacity,
        }
    }

    fn process_plugin_entries(
        entries: Vec<Arc<ZeroCopyPluginEntry>>,
    ) -> Vec<Result<Arc<dyn ZeroCopyPlugin>>> {
        entries
            .into_iter()
            .map(|entry| match entry.instance.as_ref() {
                Some(plugin) => Ok(Arc::clone(plugin)),
                None => Err(PluginError::LoadError(format!(
                    "plugin '{}' ({}) has no loaded instance; register a concrete implementation before batch load",
                    entry.name(),
                    entry.id()
                ))),
            })
            .collect()
    }

    #[expect(clippy::cast_precision_loss, reason = "Batch metrics calculation")]
    pub(super) async fn batch_load_plugins(
        &self,
        plugin_entries: Vec<Arc<ZeroCopyPluginEntry>>,
    ) -> Vec<Result<Arc<dyn ZeroCopyPlugin>>> {
        if plugin_entries.is_empty() {
            return Vec::new();
        }

        if plugin_entries.len() > self.max_queue_capacity {
            return plugin_entries
                .into_iter()
                .map(|_| {
                    Err(PluginError::StateError(format!(
                        "batch queue capacity exceeded (max {} operations per call)",
                        self.max_queue_capacity
                    )))
                })
                .collect();
        }

        let mut queue: VecDeque<BatchOperation> = plugin_entries
            .into_iter()
            .map(|entry| BatchOperation::PluginLoad {
                plugin_id: entry.id(),
                entry,
            })
            .collect();

        let mut results = Vec::with_capacity(queue.len());
        let mut batches_this_call = 0u64;
        let mut ops_this_call = 0u64;

        while !queue.is_empty() {
            let mut chunk: Vec<Arc<ZeroCopyPluginEntry>> = Vec::new();
            while chunk.len() < self.config.max_batch_size {
                match queue.front() {
                    Some(BatchOperation::PluginLoad { .. }) => {
                        if let Some(BatchOperation::PluginLoad { entry, .. }) = queue.pop_front() {
                            chunk.push(entry);
                        }
                    }
                    _ => break,
                }
            }

            if chunk.is_empty() {
                break;
            }

            let chunk_len = chunk.len();
            let batch_results = Self::process_plugin_entries(chunk);
            ops_this_call += batch_results.len() as u64;
            batches_this_call += 1;
            let time_saved_this_chunk =
                chunk_len.saturating_sub(1) as u64 * ESTIMATED_MS_SAVED_PER_EXTRA_PLUGIN;
            results.extend(batch_results);

            let mut stats = self.stats.write().await;
            stats.batches_processed += 1;
            stats.operations_batched += chunk_len as u64;
            stats.time_saved_ms = stats.time_saved_ms.saturating_add(time_saved_this_chunk);
            stats.average_batch_size =
                stats.operations_batched as f64 / stats.batches_processed as f64;
        }

        if batches_this_call > 0 {
            debug!(
                batches = batches_this_call,
                operations = ops_this_call,
                "batch_load_plugins completed"
            );
        }

        results
    }

    pub(super) async fn start_batch_processing(&self) {
        info!("Starting batch processor heartbeat");
        let config = self.config.clone();
        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.batch_timeout);
            interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
            loop {
                interval.tick().await;
                debug!("batch processor idle");
            }
        });

        let mut guard = self.task_handle.lock().await;
        *guard = Some(handle);
    }

    pub(super) async fn get_statistics(&self) -> BatchStatistics {
        self.stats.read().await.clone()
    }
}
