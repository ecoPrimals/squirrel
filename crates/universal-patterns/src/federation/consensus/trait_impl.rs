// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! ConsensusManager Trait Implementation
//!
//! Implementation of the ConsensusManager trait for DefaultConsensusManager,
//! providing the public API for proposals, voting, and state queries.

use super::super::{
    ConsensusManager, ConsensusResult, ConsensusState, ConsensusStatus, FederationError,
    FederationResult, Vote,
};
use super::core::DefaultConsensusManager;
use super::types::{ConsensusMessage, ConsensusProposal};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

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
