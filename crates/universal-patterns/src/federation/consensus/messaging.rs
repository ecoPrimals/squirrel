// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Consensus Message Processing
//!
//! Handles all consensus-related messaging including proposals, votes,
//! heartbeats, leader election, and result notifications.

use super::super::{ConsensusResult, ConsensusStatus, ParticipationStats, Vote};
use super::types::{
    ConsensusManagerState, ConsensusMessage, ConsensusNodeState, ConsensusProposal,
};
use chrono::Utc;
use std::mem;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

use super::types::ConsensusConfig;

/// Process incoming consensus messages
pub(super) async fn process_messages(
    state: Arc<RwLock<ConsensusManagerState>>,
    _config: ConsensusConfig,
    _node_id: Uuid,
    mut message_rx: mpsc::UnboundedReceiver<ConsensusMessage>,
) {
    while let Some(message) = message_rx.recv().await {
        match message {
            ConsensusMessage::Propose { proposal } => {
                handle_propose(Arc::clone(&state), proposal).await;
            }
            ConsensusMessage::Vote {
                proposal_id,
                vote,
                voter,
            } => {
                handle_vote(Arc::clone(&state), proposal_id, vote, voter).await;
            }
            ConsensusMessage::Heartbeat { leader, term } => {
                handle_heartbeat(Arc::clone(&state), leader, term).await;
            }
            ConsensusMessage::RequestVote { candidate, term } => {
                handle_request_vote(Arc::clone(&state), candidate, term).await;
            }
            ConsensusMessage::VoteResponse {
                voter,
                term,
                granted,
            } => {
                handle_vote_response(Arc::clone(&state), voter, term, granted).await;
            }
            ConsensusMessage::ResultNotification {
                proposal_id,
                result,
            } => {
                handle_result_notification(Arc::clone(&state), proposal_id, result).await;
            }
        }
    }
}

/// Handle propose message
async fn handle_propose(state: Arc<RwLock<ConsensusManagerState>>, proposal: ConsensusProposal) {
    let mut state = state.write().await;

    // Only accept proposals if we have a leader
    if state.current_leader.is_none() {
        return;
    }

    // Add proposal to active proposals
    state.active_proposals.insert(proposal.id, proposal);
}

/// Handle vote message
pub(super) async fn handle_vote(
    state: Arc<RwLock<ConsensusManagerState>>,
    proposal_id: Uuid,
    vote: Vote,
    voter: Uuid,
) {
    let mut state = state.write().await;

    if let Some(proposal) = state.active_proposals.get_mut(&proposal_id) {
        // Record the vote
        proposal.votes.insert(voter, vote);
    }

    // Update participation stats
    let voter_stats = state
        .participation_stats
        .entry(voter)
        .or_insert(ParticipationStats {
            total_proposals: 0,
            votes_for: 0,
            votes_against: 0,
            abstentions: 0,
            participation_rate: 0.0,
        });

    voter_stats.total_proposals += 1;
    match vote {
        Vote::For => voter_stats.votes_for += 1,
        Vote::Against => voter_stats.votes_against += 1,
        Vote::Abstain => voter_stats.abstentions += 1,
    }

    // Update participation rate
    let total_votes = voter_stats.votes_for + voter_stats.votes_against + voter_stats.abstentions;
    voter_stats.participation_rate =
        f64::from(total_votes) / f64::from(voter_stats.total_proposals);

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
                status: proposal.status,
                value: Some(mem::take(&mut proposal.value)),
                votes_for: votes_for as u32,
                votes_against: votes_against as u32,
                participating_nodes: proposal.votes.keys().copied().collect(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::ConsensusStatus;

    fn make_state() -> Arc<RwLock<ConsensusManagerState>> {
        Arc::new(RwLock::new(ConsensusManagerState::new()))
    }

    fn make_proposal() -> ConsensusProposal {
        ConsensusProposal {
            id: Uuid::new_v4(),
            value: b"test value".to_vec(),
            proposer: Uuid::new_v4(),
            timestamp: Utc::now(),
            deadline: Utc::now() + chrono::Duration::seconds(30),
            votes: std::collections::HashMap::new(),
            status: ConsensusStatus::InProgress,
            round: 1,
        }
    }

    #[tokio::test]
    async fn test_handle_propose_with_leader() {
        let state = make_state();
        {
            let mut s = state.write().await;
            s.current_leader = Some(Uuid::new_v4());
        }
        let proposal = make_proposal();
        let pid = proposal.id;

        handle_propose(state.clone(), proposal).await;

        let s = state.read().await;
        assert!(s.active_proposals.contains_key(&pid));
    }

    #[tokio::test]
    async fn test_handle_propose_without_leader_rejected() {
        let state = make_state();
        let proposal = make_proposal();
        let pid = proposal.id;

        handle_propose(state.clone(), proposal).await;

        let s = state.read().await;
        assert!(!s.active_proposals.contains_key(&pid));
    }

    #[tokio::test]
    async fn test_handle_vote_updates_stats() {
        let state = make_state();
        let voter = Uuid::new_v4();
        let proposal_id = Uuid::new_v4();

        handle_vote(state.clone(), proposal_id, Vote::For, voter).await;

        let s = state.read().await;
        let participation = s.participation_stats.get(&voter).expect("should succeed");
        assert_eq!(participation.votes_for, 1);
        assert_eq!(participation.total_proposals, 1);
    }

    #[tokio::test]
    async fn test_handle_vote_against_updates_stats() {
        let state = make_state();
        let voter = Uuid::new_v4();
        let proposal_id = Uuid::new_v4();

        handle_vote(state.clone(), proposal_id, Vote::Against, voter).await;

        let s = state.read().await;
        let participation = s.participation_stats.get(&voter).expect("should succeed");
        assert_eq!(participation.votes_against, 1);
    }

    #[tokio::test]
    async fn test_handle_vote_abstain_updates_stats() {
        let state = make_state();
        let voter = Uuid::new_v4();
        let proposal_id = Uuid::new_v4();

        handle_vote(state.clone(), proposal_id, Vote::Abstain, voter).await;

        let s = state.read().await;
        let participation = s.participation_stats.get(&voter).expect("should succeed");
        assert_eq!(participation.abstentions, 1);
    }

    #[tokio::test]
    async fn test_handle_heartbeat_updates_state() {
        let state = make_state();
        let leader = Uuid::new_v4();

        handle_heartbeat(state.clone(), leader, 5).await;

        let s = state.read().await;
        assert_eq!(s.current_term, 5);
        assert_eq!(s.current_leader, Some(leader));
        assert!(s.last_heartbeat.is_some());
    }

    #[tokio::test]
    async fn test_handle_heartbeat_ignores_old_term() {
        let state = make_state();
        let leader1 = Uuid::new_v4();
        let leader2 = Uuid::new_v4();

        handle_heartbeat(state.clone(), leader1, 10).await;
        handle_heartbeat(state.clone(), leader2, 5).await; // Old term

        let s = state.read().await;
        assert_eq!(s.current_term, 10);
        assert_eq!(s.current_leader, Some(leader1));
    }

    #[tokio::test]
    async fn test_handle_request_vote_grants_for_higher_term() {
        let state = make_state();
        let candidate = Uuid::new_v4();

        handle_request_vote(state.clone(), candidate, 5).await;

        let s = state.read().await;
        assert_eq!(s.current_term, 5);
        assert!(s.votes_received.get(&candidate).copied().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_handle_request_vote_denies_same_term() {
        let state = make_state();
        let candidate = Uuid::new_v4();

        // Set term to 5
        {
            let mut s = state.write().await;
            s.current_term = 5;
        }

        handle_request_vote(state.clone(), candidate, 5).await;

        let s = state.read().await;
        // Same term should not grant
        assert!(!s.votes_received.get(&candidate).copied().unwrap_or(true));
    }

    #[tokio::test]
    async fn test_handle_vote_response_as_candidate() {
        let state = make_state();
        let voter = Uuid::new_v4();

        // Set up as candidate
        {
            let mut s = state.write().await;
            s.current_term = 3;
            s.node_state = ConsensusNodeState::Candidate;
        }

        handle_vote_response(state.clone(), voter, 3, true).await;

        let s = state.read().await;
        assert!(s.votes_received.get(&voter).copied().unwrap_or(false));
    }

    #[tokio::test]
    async fn test_handle_result_notification_moves_to_completed() {
        let state = make_state();
        let proposal_id = Uuid::new_v4();
        let result = ConsensusResult {
            proposal_id,
            status: ConsensusStatus::Agreed,
            value: Some(b"result".to_vec()),
            votes_for: 3,
            votes_against: 1,
            participating_nodes: vec![Uuid::new_v4()],
        };

        handle_result_notification(state.clone(), proposal_id, result).await;

        let s = state.read().await;
        assert_eq!(s.completed_proposals.len(), 1);
        assert_eq!(s.completed_proposals[0].status, ConsensusStatus::Agreed);
    }

    #[tokio::test]
    async fn test_handle_result_notification_limits_history() {
        let state = make_state();

        // Add 105 results
        for i in 0..105 {
            let proposal_id = Uuid::new_v4();
            let result = ConsensusResult {
                proposal_id,
                status: ConsensusStatus::Agreed,
                value: Some(vec![u8::try_from(i).expect("test index fits u8")]),
                votes_for: 1,
                votes_against: 0,
                participating_nodes: vec![],
            };
            handle_result_notification(state.clone(), proposal_id, result).await;
        }

        let s = state.read().await;
        // After exceeding 100, 10 should be drained
        assert!(s.completed_proposals.len() <= 100);
    }

    #[tokio::test]
    async fn test_process_messages_handles_all_types() {
        let state = make_state();
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let (tx, rx) = mpsc::unbounded_channel();

        // Set up leader
        {
            let mut s = state.write().await;
            s.current_leader = Some(Uuid::new_v4());
        }

        let proposal = make_proposal();

        // Send messages
        tx.send(ConsensusMessage::Propose {
            proposal: proposal.clone(),
        })
        .expect("should succeed");
        tx.send(ConsensusMessage::Heartbeat {
            leader: node_id,
            term: 1,
        })
        .expect("should succeed");

        // Drop sender so the receiver loop ends
        drop(tx);

        // Process messages
        process_messages(state.clone(), config, node_id, rx).await;

        let s = state.read().await;
        assert!(s.active_proposals.contains_key(&proposal.id));
        assert_eq!(s.current_term, 1);
    }
}
