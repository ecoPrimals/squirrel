// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Command journal for tracking command execution.

use std::collections::VecDeque;
use std::path::Path;
use std::sync::{Arc, RwLock};

use tracing::{debug, error, info, warn};

use crate::{Command, CommandResult};

use super::entry::{JournalEntry, JournalEntryState, RecoveryReport};
use super::error::{JournalError, JournalResult};
use super::persistence::{FileJournalPersistence, JournalBackend, JournalPersistence};

/// Maximum number of journal entries to keep in memory by default
const DEFAULT_MAX_ENTRIES: usize = 1000;

/// Command journal for tracking command execution
pub struct CommandJournal {
    /// Journal entries
    entries: Arc<RwLock<VecDeque<JournalEntry>>>,

    /// Journal persistence
    persistence: Arc<JournalBackend>,

    /// Maximum number of entries to keep in memory
    max_entries: usize,
}

impl CommandJournal {
    /// Creates a new command journal with persistence
    pub fn new(persistence: Arc<JournalBackend>, max_entries: usize) -> Self {
        let journal = Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            persistence,
            max_entries,
        };

        // Load entries from persistence
        if let Err(e) = journal.load_entries() {
            error!("Failed to load journal entries: {}", e);
        }

        journal
    }

    /// Creates a new command journal with file persistence
    pub fn with_file<P: AsRef<Path>>(file_path: P, max_entries: usize) -> Self {
        let persistence = Arc::new(JournalBackend::File(FileJournalPersistence::new(file_path)));
        Self::new(persistence, max_entries)
    }

    /// Handle poisoned lock by recovering safely (write)
    fn handle_poisoned_entries_write(
        &self,
    ) -> JournalResult<std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>> {
        self.entries
            .write()
            .or_else(
                |poison: std::sync::PoisonError<
                    std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                >| {
                    warn!("Journal entries lock was poisoned, recovering");
                    Ok::<
                        std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                        std::sync::PoisonError<
                            std::sync::RwLockWriteGuard<'_, VecDeque<JournalEntry>>,
                        >,
                    >(poison.into_inner())
                },
            )
            .map_err(|e: std::sync::PoisonError<_>| {
                JournalError::CorruptedData(format!("Lock error: {}", e))
            })
    }

    /// Handle poisoned lock by recovering safely (read)
    fn handle_poisoned_entries_read(
        &self,
    ) -> JournalResult<std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>> {
        self.entries
            .read()
            .or_else(
                |poison: std::sync::PoisonError<
                    std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                >| {
                    warn!("Journal entries lock was poisoned, recovering");
                    Ok::<
                        std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                        std::sync::PoisonError<
                            std::sync::RwLockReadGuard<'_, VecDeque<JournalEntry>>,
                        >,
                    >(poison.into_inner())
                },
            )
            .map_err(|e: std::sync::PoisonError<_>| {
                JournalError::CorruptedData(format!("Lock error: {}", e))
            })
    }

    /// Creates a new journal entry for a command execution
    pub fn record_start(&self, command: &dyn Command, args: &[String]) -> JournalResult<String> {
        let entry = JournalEntry::new(command.name(), args.to_vec());
        let id = entry.id.clone();

        debug!("Recording command start: {} (ID: {})", command.name(), id);

        // Add the entry to the journal
        {
            let mut entries = self.handle_poisoned_entries_write()?;

            // If the journal is full, remove the oldest entry
            if entries.len() >= self.max_entries {
                entries.pop_front();
            }

            entries.push_back(entry.clone());
        }

        // Save entry to persistence
        self.persistence.save_entry(&entry)?;

        Ok(id)
    }

    /// Records the completion of a command execution
    pub fn record_completion(
        &self,
        id: String,
        result: CommandResult<String>,
    ) -> JournalResult<()> {
        let mut updated = false;

        // Update the entry in the journal
        {
            let mut entries = self.handle_poisoned_entries_write()?;

            for entry in entries.iter_mut() {
                if entry.id == id {
                    entry.complete(result);
                    updated = true;

                    // Save the updated entry to persistence
                    self.persistence.save_entry(entry)?;

                    break;
                }
            }
        }

        if !updated {
            return Err(JournalError::EntryNotFound(id));
        }

        Ok(())
    }

    /// Finds incomplete journal entries
    pub fn find_incomplete(&self) -> JournalResult<Vec<JournalEntry>> {
        let entries = self.handle_poisoned_entries_read()?;
        let incomplete = entries
            .iter()
            .filter(|e| e.state == JournalEntryState::Started)
            .cloned()
            .collect();

        Ok(incomplete)
    }

    /// Recovers incomplete commands
    pub fn recover_incomplete<F>(&self, recover_fn: F) -> JournalResult<RecoveryReport>
    where
        F: Fn(&JournalEntry) -> CommandResult<String>,
    {
        let incomplete = self.find_incomplete()?;
        let mut report = RecoveryReport {
            processed: incomplete.len(),
            recovered: 0,
            failed: 0,
            recovered_entries: Vec::new(),
            failed_entries: Vec::new(),
        };

        if incomplete.is_empty() {
            return Ok(report);
        }

        info!("Recovering {} incomplete commands", incomplete.len());

        for incomplete_entry in incomplete {
            let id = incomplete_entry.id.clone();
            let command_name = incomplete_entry.command_name.clone();

            debug!("Recovering command: {} (ID: {})", command_name, id);

            // Try to recover the command
            let result = recover_fn(&incomplete_entry);

            // Update the entry with the recovery result
            {
                let mut entries = self.handle_poisoned_entries_write()?;

                for entry in entries.iter_mut() {
                    if entry.id == id {
                        entry.complete(result);
                        entry.recover();

                        // Save the updated entry to persistence
                        self.persistence.save_entry(entry)?;

                        if entry.state == JournalEntryState::Recovered {
                            report.recovered += 1;
                            report.recovered_entries.push(entry.clone());
                            info!(
                                "Successfully recovered command: {} (ID: {})",
                                command_name, id
                            );
                        } else {
                            report.failed += 1;
                            report.failed_entries.push(entry.clone());
                            warn!("Failed to recover command: {} (ID: {})", command_name, id);
                        }

                        break;
                    }
                }
            }
        }

        info!(
            "Recovery completed: {} processed, {} recovered, {} failed",
            report.processed, report.recovered, report.failed
        );

        Ok(report)
    }

    /// Gets all journal entries
    pub fn get_entries(&self) -> JournalResult<Vec<JournalEntry>> {
        let entries = self.handle_poisoned_entries_read()?;
        Ok(entries.iter().cloned().collect())
    }

    /// Gets a journal entry by ID
    pub fn get_entry(&self, id: String) -> JournalResult<JournalEntry> {
        let entries = self.handle_poisoned_entries_read()?;

        for entry in entries.iter() {
            if entry.id == id {
                return Ok(entry.clone());
            }
        }

        Err(JournalError::EntryNotFound(id))
    }

    /// Searches for journal entries by criteria
    pub fn search_entries<F>(&self, predicate: F) -> JournalResult<Vec<JournalEntry>>
    where
        F: Fn(&JournalEntry) -> bool,
    {
        let entries = self.handle_poisoned_entries_read()?;
        let matches = entries.iter().filter(|e| predicate(e)).cloned().collect();
        Ok(matches)
    }

    /// Loads entries from persistence
    fn load_entries(&self) -> JournalResult<()> {
        let loaded_entries = self.persistence.load_entries()?;

        if !loaded_entries.is_empty() {
            let mut entries = self.handle_poisoned_entries_write()?;
            entries.clear();

            // Only keep the most recent entries up to max_entries
            let start_idx = loaded_entries.len().saturating_sub(self.max_entries);
            for entry in loaded_entries.into_iter().skip(start_idx) {
                entries.push_back(entry);
            }

            info!("Loaded {} journal entries", entries.len());
        }

        Ok(())
    }
}

impl Default for CommandJournal {
    fn default() -> Self {
        let persistence = Arc::new(JournalBackend::File(FileJournalPersistence::default()));
        Self::new(persistence, DEFAULT_MAX_ENTRIES)
    }
}
