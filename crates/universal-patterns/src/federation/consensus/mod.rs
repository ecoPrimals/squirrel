// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Consensus Mechanism for Federation Coordination
//!
//! This module provides a distributed consensus mechanism for coordinating
//! decisions across the federation network. It implements a simplified
//! Raft-like consensus algorithm suitable for the universal patterns framework.
//!
//! ## Architecture
//!
//! The consensus module is organized into logical, semantic sub-modules:
//!
//! - **`types`**: Core type definitions (Config, Proposal, Messages, State)
//! - **`core`**: Core manager implementation and node management
//! - **`messaging`**: Message processing and handlers
//! - **`trait_impl`**: ConsensusManager trait implementation
//!
//! This structure provides clear separation of concerns while keeping
//! each module focused and maintainable (<250 lines each).

mod core;
mod messaging;
mod trait_impl;
mod types;

// Re-export public API
pub use core::DefaultConsensusManager;
pub use types::{ConsensusConfig, ConsensusMessage, ConsensusNodeState, ConsensusProposal};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::{ConsensusManager, FederationError, FederationNode, NodeStatus, Vote};
    use chrono::Utc;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_consensus_manager_creation() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let state = manager.get_state().await.expect("should succeed");
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
            _ => unreachable!("Expected ConsensusFailure error"),
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
            _ => unreachable!("Expected ExecutionNotFound error"),
        }
    }

    #[tokio::test]
    async fn test_consensus_state() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let state = manager.get_state().await.expect("should succeed");
        assert_eq!(state.round, 0);
        assert!(state.active_proposals.is_empty());
        assert!(state.recent_results.is_empty());
        assert!(state.participation_stats.is_empty());
    }

    #[tokio::test]
    async fn test_multiple_node_registration() {
        let config = ConsensusConfig::default();
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        // Register multiple nodes
        for i in 0..5 {
            let node = FederationNode {
                id: Uuid::new_v4(),
                address: format!("127.0.0.{}", i),
                port: 8080 + u16::try_from(i).expect("test loop index fits u16"),
                public_key: format!("key_{}", i),
                capabilities: vec!["consensus".to_string()],
                status: NodeStatus::Active,
                last_seen: Utc::now(),
                metadata: HashMap::new(),
            };
            manager.register_node(node).await;
        }

        let active_nodes = manager.get_active_nodes().await;
        assert_eq!(active_nodes.len(), 5);
    }

    #[tokio::test]
    async fn test_node_removal() {
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
        assert_eq!(manager.get_active_nodes().await.len(), 1);

        manager.remove_node(node.id).await;
        assert_eq!(manager.get_active_nodes().await.len(), 0);
    }

    #[tokio::test]
    async fn test_quorum_check() {
        let config = ConsensusConfig {
            min_nodes: 3,
            ..Default::default()
        };
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        // No quorum initially
        assert!(!manager.has_quorum().await);

        // Add nodes until quorum
        for i in 0..3 {
            let node = FederationNode {
                id: Uuid::new_v4(),
                address: format!("127.0.0.{}", i),
                port: 8080 + u16::try_from(i).expect("test loop index fits u16"),
                public_key: format!("key_{}", i),
                capabilities: vec!["consensus".to_string()],
                status: NodeStatus::Active,
                last_seen: Utc::now(),
                metadata: HashMap::new(),
            };
            manager.register_node(node).await;
        }

        // Now we have quorum
        assert!(manager.has_quorum().await);
    }

    #[tokio::test]
    async fn test_inactive_nodes_excluded_from_quorum() {
        let config = ConsensusConfig {
            min_nodes: 2,
            ..Default::default()
        };
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        // Add active node
        let active_node = FederationNode {
            id: Uuid::new_v4(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            public_key: "key_1".to_string(),
            capabilities: vec!["consensus".to_string()],
            status: NodeStatus::Active,
            last_seen: Utc::now(),
            metadata: HashMap::new(),
        };
        manager.register_node(active_node).await;

        // Add inactive node
        let inactive_node = FederationNode {
            id: Uuid::new_v4(),
            address: "127.0.0.2".to_string(),
            port: 8081,
            public_key: "key_2".to_string(),
            capabilities: vec!["consensus".to_string()],
            status: NodeStatus::Inactive,
            last_seen: Utc::now(),
            metadata: HashMap::new(),
        };
        manager.register_node(inactive_node).await;

        // Only active nodes count for quorum
        assert!(!manager.has_quorum().await);
        assert_eq!(manager.get_active_nodes().await.len(), 1);
    }

    #[tokio::test]
    async fn test_consensus_config_default() {
        let config = ConsensusConfig::default();

        // Verify default values are reasonable
        assert!(config.min_nodes > 0);
        assert!(config.voting_timeout_seconds > 0);
        assert!(config.heartbeat_interval_seconds > 0);
        assert!(config.election_timeout_seconds > 0);
        assert!(config.max_proposal_history > 0);
    }

    #[tokio::test]
    async fn test_consensus_proposal_creation() {
        use crate::federation::ConsensusStatus;
        use types::ConsensusProposal;

        let proposal = ConsensusProposal {
            id: Uuid::new_v4(),
            value: b"test data".to_vec(),
            proposer: Uuid::new_v4(),
            timestamp: Utc::now(),
            deadline: Utc::now() + chrono::Duration::seconds(30),
            votes: HashMap::new(),
            status: ConsensusStatus::InProgress,
            round: 1,
        };

        assert_eq!(proposal.status, ConsensusStatus::InProgress);
        assert!(proposal.votes.is_empty());
        assert_eq!(proposal.value, b"test data");
    }

    #[tokio::test]
    async fn test_vote_types() {
        // Test vote enum variants
        let vote_for = Vote::For;
        let vote_against = Vote::Against;
        let vote_abstain = Vote::Abstain;

        // Ensure they're different
        match vote_for {
            Vote::For => {}
            _ => unreachable!("Expected Vote::For"),
        }

        match vote_against {
            Vote::Against => {}
            _ => unreachable!("Expected Vote::Against"),
        }

        match vote_abstain {
            Vote::Abstain => {}
            _ => unreachable!("Expected Vote::Abstain"),
        }
    }

    #[tokio::test]
    async fn test_consensus_status_types() {
        use crate::federation::ConsensusStatus;

        // Test all status variants
        assert!(ConsensusStatus::Agreed == ConsensusStatus::Agreed);
        assert!(ConsensusStatus::Disagreed == ConsensusStatus::Disagreed);
        assert!(ConsensusStatus::InProgress == ConsensusStatus::InProgress);
        assert!(ConsensusStatus::TimedOut == ConsensusStatus::TimedOut);
        assert!(ConsensusStatus::Failed == ConsensusStatus::Failed);

        // Ensure different statuses are not equal
        assert!(ConsensusStatus::Agreed != ConsensusStatus::Disagreed);
    }

    #[tokio::test]
    async fn test_consensus_node_states() {
        use types::ConsensusNodeState;

        // Test node state variants
        let follower = ConsensusNodeState::Follower;
        let candidate = ConsensusNodeState::Candidate;
        let leader = ConsensusNodeState::Leader;

        // Ensure they're distinct
        match follower {
            ConsensusNodeState::Follower => {}
            _ => unreachable!("Expected Follower"),
        }

        match candidate {
            ConsensusNodeState::Candidate => {}
            _ => unreachable!("Expected Candidate"),
        }

        match leader {
            ConsensusNodeState::Leader => {}
            _ => unreachable!("Expected Leader"),
        }
    }

    #[tokio::test]
    async fn test_propose_times_out_when_no_votes() {
        use std::sync::Arc;
        use tokio::time::Duration;

        let config = ConsensusConfig {
            min_nodes: 1,
            voting_timeout_seconds: 1,
            ..Default::default()
        };
        let node_id = Uuid::new_v4();
        let manager = Arc::new(DefaultConsensusManager::new(config, node_id));

        let node = FederationNode {
            id: Uuid::new_v4(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            public_key: "k".to_string(),
            capabilities: vec![],
            status: NodeStatus::Active,
            last_seen: Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        manager.register_node(node).await;

        let result =
            tokio::time::timeout(Duration::from_secs(5), manager.propose(b"payload".to_vec()))
                .await
                .expect("propose should finish")
                .expect("propose result");

        assert_eq!(result.status, crate::federation::ConsensusStatus::TimedOut);
    }

    #[tokio::test]
    async fn test_propose_completes_after_vote_for_majority() {
        use std::sync::Arc;
        use tokio::time::{Duration, sleep};

        let config = ConsensusConfig {
            min_nodes: 1,
            voting_timeout_seconds: 30,
            ..Default::default()
        };
        let node_id = Uuid::new_v4();
        let manager = Arc::new(DefaultConsensusManager::new(config, node_id));

        let node = FederationNode {
            id: Uuid::new_v4(),
            address: "127.0.0.1".to_string(),
            port: 8080,
            public_key: "k".to_string(),
            capabilities: vec![],
            status: NodeStatus::Active,
            last_seen: Utc::now(),
            metadata: std::collections::HashMap::new(),
        };
        manager.register_node(node).await;

        let mgr = manager.clone();
        let propose_task = tokio::spawn(async move { mgr.propose(b"value".to_vec()).await });

        sleep(Duration::from_millis(50)).await;
        let active = manager.get_state().await.expect("state").active_proposals;
        assert_eq!(active.len(), 1);
        let proposal_id = active[0];

        manager.vote(proposal_id, Vote::For).await.expect("vote");

        let result = tokio::time::timeout(Duration::from_secs(5), propose_task)
            .await
            .expect("join timeout")
            .expect("task panic")
            .expect("propose");

        assert_eq!(result.status, crate::federation::ConsensusStatus::Agreed);
        assert_eq!(result.value, Some(b"value".to_vec()));

        let by_get = manager.get_result(proposal_id).await.expect("get_result");
        assert_eq!(by_get.proposal_id, proposal_id);
        assert_eq!(by_get.status, crate::federation::ConsensusStatus::Agreed);
    }

    #[tokio::test]
    async fn test_get_result_active_proposal_vote_counts() {
        let config = ConsensusConfig {
            min_nodes: 1,
            voting_timeout_seconds: 30,
            ..Default::default()
        };
        let node_id = Uuid::new_v4();
        let manager = DefaultConsensusManager::new(config, node_id);

        let proposal_id = Uuid::new_v4();
        let now = Utc::now();
        let proposal = ConsensusProposal {
            id: proposal_id,
            value: b"x".to_vec(),
            proposer: node_id,
            timestamp: now,
            deadline: now + chrono::Duration::seconds(60),
            votes: {
                let mut m = std::collections::HashMap::new();
                m.insert(Uuid::new_v4(), Vote::For);
                m.insert(Uuid::new_v4(), Vote::Against);
                m
            },
            status: crate::federation::ConsensusStatus::InProgress,
            round: 0,
        };

        {
            let mut state = manager.state.write().await;
            state.active_proposals.insert(proposal_id, proposal);
        }

        let r = manager.get_result(proposal_id).await.expect("get_result");
        assert_eq!(r.votes_for, 1);
        assert_eq!(r.votes_against, 1);
        assert_eq!(r.participating_nodes.len(), 2);
    }
}
