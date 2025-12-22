//! Consensus Message Processing
//!
//! Handles all consensus-related messaging including proposals, votes,
//! heartbeats, leader election, and result notifications.

use super::super::{ConsensusResult, ConsensusStatus, ParticipationStats, Vote};
use super::types::{
    ConsensusManagerState, ConsensusMessage, ConsensusNodeState, ConsensusProposal,
};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
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
                handle_propose(state.clone(), proposal).await;
            }
            ConsensusMessage::Vote {
                proposal_id,
                vote,
                voter,
            } => {
                handle_vote(state.clone(), proposal_id, vote, voter).await;
            }
            ConsensusMessage::Heartbeat { leader, term } => {
                handle_heartbeat(state.clone(), leader, term).await;
            }
            ConsensusMessage::RequestVote { candidate, term } => {
                handle_request_vote(state.clone(), candidate, term).await;
            }
            ConsensusMessage::VoteResponse {
                voter,
                term,
                granted,
            } => {
                handle_vote_response(state.clone(), voter, term, granted).await;
            }
            ConsensusMessage::ResultNotification {
                proposal_id,
                result,
            } => {
                handle_result_notification(state.clone(), proposal_id, result).await;
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
