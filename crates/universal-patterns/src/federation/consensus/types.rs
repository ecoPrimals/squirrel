// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 DataScienceBioLab

//! Consensus Type Definitions
//!
//! Core types for the distributed consensus mechanism including configuration,
//! node states, proposals, messages, and internal state management.

use super::super::{ConsensusResult, ConsensusStatus, ParticipationStats, Vote};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Minimum number of nodes required for consensus
    pub min_nodes: u32,
    /// Timeout for proposal voting in seconds
    pub voting_timeout_seconds: u64,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_seconds: u64,
    /// Election timeout in seconds
    pub election_timeout_seconds: u64,
    /// Maximum number of proposals to keep in history
    pub max_proposal_history: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            min_nodes: 3,
            voting_timeout_seconds: 30,
            heartbeat_interval_seconds: 5,
            election_timeout_seconds: 10,
            max_proposal_history: 100,
        }
    }
}

/// Consensus node state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusNodeState {
    /// Node is a follower
    Follower,
    /// Node is a candidate seeking leadership
    Candidate,
    /// Node is the leader
    Leader,
}

/// Consensus proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    /// Proposal identifier
    pub id: Uuid,
    /// Proposal value/data
    pub value: Vec<u8>,
    /// Proposer node ID
    pub proposer: Uuid,
    /// Proposal timestamp
    pub timestamp: DateTime<Utc>,
    /// Voting deadline
    pub deadline: DateTime<Utc>,
    /// Current vote count
    pub votes: HashMap<Uuid, Vote>,
    /// Proposal status
    pub status: ConsensusStatus,
    /// Round number
    pub round: u64,
}

/// Consensus message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusMessage {
    /// Propose a new value
    Propose {
        /// The proposal being made
        proposal: ConsensusProposal,
    },
    /// Vote on a proposal
    Vote {
        /// ID of the proposal being voted on
        proposal_id: Uuid,
        /// The vote being cast
        vote: Vote,
        /// ID of the voter
        voter: Uuid,
    },
    /// Heartbeat message
    Heartbeat {
        /// ID of the current leader
        leader: Uuid,
        /// Current consensus term
        term: u64,
    },
    /// Request votes for leadership
    RequestVote {
        /// ID of the candidate requesting votes
        candidate: Uuid,
        /// The term for which votes are requested
        term: u64,
    },
    /// Vote response for leadership
    VoteResponse {
        /// ID of the voter responding
        voter: Uuid,
        /// The term being voted for
        term: u64,
        /// Whether the vote was granted
        granted: bool,
    },
    /// Consensus result notification
    ResultNotification {
        /// ID of the proposal that reached consensus
        proposal_id: Uuid,
        /// The consensus result
        result: ConsensusResult,
    },
}

/// Internal state for consensus manager
#[derive(Debug)]
pub(super) struct ConsensusManagerState {
    /// Current node state
    pub node_state: ConsensusNodeState,
    /// Current term/round
    pub current_term: u64,
    /// Current leader (if known)
    pub current_leader: Option<Uuid>,
    /// Active proposals
    pub active_proposals: HashMap<Uuid, ConsensusProposal>,
    /// Completed proposals
    pub completed_proposals: Vec<ConsensusResult>,
    /// Participation statistics
    pub participation_stats: HashMap<Uuid, ParticipationStats>,
    /// Last heartbeat received
    pub last_heartbeat: Option<DateTime<Utc>>,
    /// Votes for current term
    pub votes_received: HashMap<Uuid, bool>,
}

impl ConsensusManagerState {
    /// Create new default consensus manager state
    pub fn new() -> Self {
        Self {
            node_state: ConsensusNodeState::Follower,
            current_term: 0,
            current_leader: None,
            active_proposals: HashMap::new(),
            completed_proposals: Vec::new(),
            participation_stats: HashMap::new(),
            last_heartbeat: None,
            votes_received: HashMap::new(),
        }
    }
}
