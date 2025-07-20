//! Consensus Mechanism for Federation Coordination
//!
//! This module provides a distributed consensus mechanism for coordinating
//! decisions across the federation network. It implements a simplified
//! Raft-like consensus algorithm suitable for the universal patterns framework.

use super::{
    ConsensusManager, ConsensusResult, ConsensusState, ConsensusStatus, FederationError,
    FederationNode, FederationResult, NodeStatus, ParticipationStats, Vote,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
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
        proposal: ConsensusProposal 
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
        term: u64 
    },
    /// Request votes for leadership
    RequestVote { 
        /// ID of the candidate requesting votes
        candidate: Uuid, 
        /// The term for which votes are requested
        term: u64 
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

/// Default consensus manager implementation
pub struct DefaultConsensusManager {
    /// Node configuration
    config: ConsensusConfig,
    /// Current node ID
    node_id: Uuid,
    /// Current consensus state
    state: Arc<RwLock<ConsensusManagerState>>,
    /// Message channel for communication
    message_tx: mpsc::UnboundedSender<ConsensusMessage>,
    /// Registered nodes in the federation
    nodes: Arc<RwLock<HashMap<Uuid, FederationNode>>>,
}

/// Internal state for consensus manager
#[derive(Debug)]
struct ConsensusManagerState {
    /// Current node state
    node_state: ConsensusNodeState,
    /// Current term/round
    current_term: u64,
    /// Current leader (if known)
    current_leader: Option<Uuid>,
    /// Active proposals
    active_proposals: HashMap<Uuid, ConsensusProposal>,
    /// Completed proposals
    completed_proposals: Vec<ConsensusResult>,
    /// Participation statistics
    participation_stats: HashMap<Uuid, ParticipationStats>,
    /// Last heartbeat received
    last_heartbeat: Option<DateTime<Utc>>,
    /// Votes for current term
    votes_received: HashMap<Uuid, bool>,
}

impl DefaultConsensusManager {
    /// Create a new consensus manager
    pub fn new(config: ConsensusConfig, node_id: Uuid) -> Self {
        let (message_tx, message_rx) = mpsc::unbounded_channel();

        let state = Arc::new(RwLock::new(ConsensusManagerState {
            node_state: ConsensusNodeState::Follower,
            current_term: 0,
            current_leader: None,
            active_proposals: HashMap::new(),
            completed_proposals: Vec::new(),
            participation_stats: HashMap::new(),
            last_heartbeat: None,
            votes_received: HashMap::new(),
        }));

        let config_clone = config.clone();

        let manager = Self {
            config,
            node_id,
            state: state.clone(),
            message_tx,
            nodes: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start message processing task
        let state_clone = state.clone();
        let node_id_clone = node_id;
        tokio::spawn(async move {
            Self::process_messages(state_clone, config_clone, node_id_clone, message_rx).await;
        });

        manager
    }

    /// Register a node in the federation
    pub async fn register_node(&self, node: FederationNode) {
        let mut nodes = self.nodes.write().await;
        nodes.insert(node.id, node);
    }

    /// Remove a node from the federation
    pub async fn remove_node(&self, node_id: Uuid) {
        let mut nodes = self.nodes.write().await;
        nodes.remove(&node_id);
    }

    /// Get active nodes
    async fn get_active_nodes(&self) -> Vec<FederationNode> {
        let nodes = self.nodes.read().await;
        nodes
            .values()
            .filter(|node| node.status == NodeStatus::Active)
            .cloned()
            .collect()
    }

    /// Check if we have enough nodes for consensus
    async fn has_quorum(&self) -> bool {
        let active_nodes = self.get_active_nodes().await;
        active_nodes.len() >= self.config.min_nodes as usize
    }

    /// Process incoming messages
    async fn process_messages(
        state: Arc<RwLock<ConsensusManagerState>>,
        _config: ConsensusConfig,
        _node_id: Uuid,
        mut message_rx: mpsc::UnboundedReceiver<ConsensusMessage>,
    ) {
        while let Some(message) = message_rx.recv().await {
            match message {
                ConsensusMessage::Propose { proposal } => {
                    Self::handle_propose(state.clone(), proposal).await;
                }
                ConsensusMessage::Vote {
                    proposal_id,
                    vote,
                    voter,
                } => {
                    Self::handle_vote(state.clone(), proposal_id, vote, voter).await;
                }
                ConsensusMessage::Heartbeat { leader, term } => {
                    Self::handle_heartbeat(state.clone(), leader, term).await;
                }
                ConsensusMessage::RequestVote { candidate, term } => {
                    Self::handle_request_vote(state.clone(), candidate, term).await;
                }
                ConsensusMessage::VoteResponse {
                    voter,
                    term,
                    granted,
                } => {
                    Self::handle_vote_response(state.clone(), voter, term, granted).await;
                }
                ConsensusMessage::ResultNotification {
                    proposal_id,
                    result,
                } => {
                    Self::handle_result_notification(state.clone(), proposal_id, result).await;
                }
            }
        }
    }

    /// Handle propose message
    async fn handle_propose(
        state: Arc<RwLock<ConsensusManagerState>>,
        proposal: ConsensusProposal,
    ) {
        let mut state = state.write().await;

        // Only accept proposals if we have a leader
        if state.current_leader.is_none() {
            return;
        }

        // Add proposal to active proposals
        state.active_proposals.insert(proposal.id, proposal);
    }

    /// Handle vote message
    async fn handle_vote(
        state: Arc<RwLock<ConsensusManagerState>>,
        proposal_id: Uuid,
        vote: Vote,
        voter: Uuid,
    ) {
        let mut state = state.write().await;

        if let Some(proposal) = state.active_proposals.get_mut(&proposal_id) {
            // Record the vote
            proposal.votes.insert(voter, vote.clone());
        }

        // Update participation stats
        let stats = state
            .participation_stats
            .entry(voter)
            .or_insert(ParticipationStats {
                total_proposals: 0,
                votes_for: 0,
                votes_against: 0,
                abstentions: 0,
                participation_rate: 0.0,
            });

        stats.total_proposals += 1;
        match vote {
            Vote::For => stats.votes_for += 1,
            Vote::Against => stats.votes_against += 1,
            Vote::Abstain => stats.abstentions += 1,
        }

        // Update participation rate
        let total_votes = stats.votes_for + stats.votes_against + stats.abstentions;
        stats.participation_rate = total_votes as f64 / stats.total_proposals as f64;

        // Check if consensus is reached
        if let Some(proposal) = state.active_proposals.get_mut(&proposal_id) {
            let total_votes = proposal.votes.len();
            let votes_for = proposal
                .votes
                .values()
                .filter(|v| matches!(v, Vote::For))
                .count();
            let votes_against = proposal
                .votes
                .values()
                .filter(|v| matches!(v, Vote::Against))
                .count();

            // Simple majority consensus
            if votes_for > total_votes / 2 {
                proposal.status = ConsensusStatus::Agreed;
            } else if votes_against > total_votes / 2 {
                proposal.status = ConsensusStatus::Disagreed;
            }

            // Move to completed if consensus reached
            if proposal.status != ConsensusStatus::InProgress {
                let result = ConsensusResult {
                    proposal_id: proposal.id,
                    status: proposal.status.clone(),
                    value: Some(proposal.value.clone()),
                    votes_for: votes_for as u32,
                    votes_against: votes_against as u32,
                    participating_nodes: proposal.votes.keys().cloned().collect(),
                };

                state.completed_proposals.push(result);
                state.active_proposals.remove(&proposal_id);
            }
        }
    }

    /// Handle heartbeat message
    async fn handle_heartbeat(state: Arc<RwLock<ConsensusManagerState>>, leader: Uuid, term: u64) {
        let mut state = state.write().await;

        if term >= state.current_term {
            state.current_term = term;
            state.current_leader = Some(leader);
            state.node_state = ConsensusNodeState::Follower;
            state.last_heartbeat = Some(Utc::now());
        }
    }

    /// Handle request vote message
    async fn handle_request_vote(
        state: Arc<RwLock<ConsensusManagerState>>,
        candidate: Uuid,
        term: u64,
    ) {
        let mut state = state.write().await;

        let should_grant = term > state.current_term;

        if should_grant {
            state.current_term = term;
            state.current_leader = None;
            state.node_state = ConsensusNodeState::Follower;
        }

        // In a real implementation, we would send a vote response
        // For now, we just update our local state
        state.votes_received.insert(candidate, should_grant);
    }

    /// Handle vote response message
    async fn handle_vote_response(
        state: Arc<RwLock<ConsensusManagerState>>,
        voter: Uuid,
        term: u64,
        granted: bool,
    ) {
        let mut state = state.write().await;

        if term == state.current_term && matches!(state.node_state, ConsensusNodeState::Candidate) {
            state.votes_received.insert(voter, granted);

            // Check if we have majority votes
            let total_votes = state.votes_received.len();
            let granted_votes = state.votes_received.values().filter(|&&v| v).count();

            if granted_votes > total_votes / 2 {
                state.node_state = ConsensusNodeState::Leader;
                state.current_leader = Some(voter); // This would be our node ID in practice
            }
        }
    }

    /// Handle result notification
    async fn handle_result_notification(
        state: Arc<RwLock<ConsensusManagerState>>,
        proposal_id: Uuid,
        result: ConsensusResult,
    ) {
        let mut state = state.write().await;

        // Remove from active proposals and add to completed
        state.active_proposals.remove(&proposal_id);
        state.completed_proposals.push(result);

        // Limit history size
        if state.completed_proposals.len() > 100 {
            state.completed_proposals.drain(0..10);
        }
    }
}

#[async_trait]
impl ConsensusManager for DefaultConsensusManager {
    async fn propose(&self, value: Vec<u8>) -> FederationResult<ConsensusResult> {
        // Check if we have quorum
        if !self.has_quorum().await {
            return Err(FederationError::ConsensusFailure(
                "Insufficient nodes for consensus".to_string(),
            ));
        }

        let proposal_id = Uuid::new_v4();
        let now = Utc::now();
        let deadline = now + chrono::Duration::seconds(self.config.voting_timeout_seconds as i64);

        let proposal = ConsensusProposal {
            id: proposal_id,
            value: value.clone(),
            proposer: self.node_id,
            timestamp: now,
            deadline,
            votes: HashMap::new(),
            status: ConsensusStatus::InProgress,
            round: {
                let state = self.state.read().await;
                state.current_term
            },
        };

        // Add to active proposals
        {
            let mut state = self.state.write().await;
            state.active_proposals.insert(proposal_id, proposal.clone());
        }

        // Send proposal message
        self.message_tx
            .send(ConsensusMessage::Propose { proposal })
            .ok();

        // Wait for consensus or timeout
        let timeout = tokio::time::sleep(std::time::Duration::from_secs(
            self.config.voting_timeout_seconds,
        ));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    // Timeout - mark as failed
                    let mut state = self.state.write().await;
                    if let Some(mut proposal) = state.active_proposals.remove(&proposal_id) {
                        proposal.status = ConsensusStatus::TimedOut;

                        let result = ConsensusResult {
                            proposal_id,
                            status: ConsensusStatus::TimedOut,
                            value: Some(value),
                            votes_for: 0,
                            votes_against: 0,
                            participating_nodes: vec![],
                        };

                        state.completed_proposals.push(result.clone());
                        return Ok(result);
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
                    // Check if consensus reached
                    let state = self.state.read().await;
                    if let Some(result) = state.completed_proposals.iter()
                        .find(|r| r.proposal_id == proposal_id) {
                        return Ok(result.clone());
                    }
                }
            }
        }
    }

    async fn vote(&self, proposal_id: Uuid, vote: Vote) -> FederationResult<()> {
        // Send vote message
        self.message_tx
            .send(ConsensusMessage::Vote {
                proposal_id,
                vote,
                voter: self.node_id,
            })
            .map_err(|_| FederationError::InternalError("Failed to send vote".to_string()))?;

        Ok(())
    }

    async fn get_result(&self, proposal_id: Uuid) -> FederationResult<ConsensusResult> {
        let state = self.state.read().await;

        // Check completed proposals
        if let Some(result) = state
            .completed_proposals
            .iter()
            .find(|r| r.proposal_id == proposal_id)
        {
            return Ok(result.clone());
        }

        // Check active proposals
        if let Some(proposal) = state.active_proposals.get(&proposal_id) {
            let votes_for = proposal
                .votes
                .values()
                .filter(|v| matches!(v, Vote::For))
                .count();
            let votes_against = proposal
                .votes
                .values()
                .filter(|v| matches!(v, Vote::Against))
                .count();

            return Ok(ConsensusResult {
                proposal_id,
                status: proposal.status.clone(),
                value: Some(proposal.value.clone()),
                votes_for: votes_for as u32,
                votes_against: votes_against as u32,
                participating_nodes: proposal.votes.keys().cloned().collect(),
            });
        }

        Err(FederationError::ExecutionNotFound(proposal_id))
    }

    async fn get_state(&self) -> FederationResult<ConsensusState> {
        let state = self.state.read().await;

        Ok(ConsensusState {
            round: state.current_term,
            active_proposals: state.active_proposals.keys().cloned().collect(),
            recent_results: state.completed_proposals.clone(),
            participation_stats: state.participation_stats.clone(),
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_consensus_manager_creation() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let state = manager.get_state().await.unwrap();
        assert_eq!(state.round, 0);
        assert!(state.active_proposals.is_empty());
    }

    #[tokio::test]
    async fn test_node_registration() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let node = FederationNode {
            id: Uuid::new_v4(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            public_key: "test_key".to_string(),
            capabilities: vec!["consensus".to_string()],
            status: NodeStatus::Active,
            last_seen: Utc::now(),
            metadata: HashMap::new(),
        };

        manager.register_node(node.clone()).await;

        let active_nodes = manager.get_active_nodes().await;
        assert_eq!(active_nodes.len(), 1);
        assert_eq!(active_nodes[0].id, node.id);
    }

    #[tokio::test]
    async fn test_proposal_without_quorum() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let result = manager.propose(b"test value".to_vec()).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            FederationError::ConsensusFailure(msg) => {
                assert!(msg.contains("Insufficient nodes"));
            }
            _ => panic!("Expected ConsensusFailure error"),
        }
    }

    #[tokio::test]
    async fn test_vote_on_nonexistent_proposal() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let proposal_id = Uuid::new_v4();
        let result = manager.vote(proposal_id, Vote::For).await;
        assert!(result.is_ok()); // Vote sending should succeed, even if proposal doesn't exist

        let get_result = manager.get_result(proposal_id).await;
        assert!(get_result.is_err());

        match get_result.unwrap_err() {
            FederationError::ExecutionNotFound(id) => {
                assert_eq!(id, proposal_id);
            }
            _ => panic!("Expected ExecutionNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_consensus_state() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let state = manager.get_state().await.unwrap();
        assert_eq!(state.round, 0);
        assert!(state.active_proposals.is_empty());
        assert!(state.recent_results.is_empty());
        assert!(state.participation_stats.is_empty());
    }
}
