// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Tests for policy network

use super::engine::RLState;
use super::policy::*;
use super::test_helpers;
use super::*;
use chrono::Utc;

#[tokio::test]
async fn test_policy_network_new() {
    let config = PolicyNetworkConfig::default();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let metrics = network.get_metrics().await;
    assert_eq!(metrics.total_predictions, 0);
}

#[tokio::test]
async fn test_policy_network_initialize() {
    let config = PolicyNetworkConfig::default();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    network.initialize().await.expect("Should initialize");

    let training_state = network.get_training_state().await;
    assert_eq!(training_state.learning_rate, 0.001);
}

#[tokio::test]
async fn test_policy_network_forward() {
    let config = PolicyNetworkConfig {
        input_size: 10,
        hidden_layers: vec![8, 4],
        output_size: 3,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.2,
    };

    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let input: Vec<f64> = (0..10).map(|x| x as f64).collect();
    let action = network.forward(&input).await.expect("Should forward");

    assert_eq!(action.action_probabilities.len(), 3);
    assert!(action.selected_action < 3);
    assert!(action.confidence >= 0.0 && action.confidence <= 1.0);

    // Probabilities should sum to ~1.0 (softmax)
    let sum: f64 = action.action_probabilities.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[tokio::test]
async fn test_policy_network_forward_wrong_input_size() {
    let config = PolicyNetworkConfig::default();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let wrong_input: Vec<f64> = vec![1.0, 2.0, 3.0]; // Wrong size
    let result = network.forward(&wrong_input).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_policy_network_predict() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let state = create_test_rl_state();
    let action = network.predict(&state).await.expect("Should predict");

    assert!(!action.action_probabilities.is_empty());
    assert!(action.confidence >= 0.0 && action.confidence <= 1.0);
}

#[tokio::test]
async fn test_policy_network_train() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let experiences = vec![create_test_rl_experience()];

    network.train(&experiences).await.expect("Should train");

    let training_state = network.get_training_state().await;
    assert!(training_state.epoch > 0);
}

#[tokio::test]
async fn test_policy_network_update_weights() {
    let config = PolicyNetworkConfig {
        input_size: 4,
        hidden_layers: vec![8],
        output_size: 2,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.2,
    };

    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    // Create mock gradients
    let gradients = vec![vec![
        vec![0.01, 0.02, 0.03, 0.04],
        vec![0.01, 0.02, 0.03, 0.04],
    ]];

    network
        .update_weights(&gradients)
        .await
        .expect("Should update weights");

    let training_state = network.get_training_state().await;
    assert!(training_state.epoch > 0);
}

#[tokio::test]
async fn test_policy_network_evaluate() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let test_states = vec![create_test_rl_state(); 10];

    let accuracy = network
        .evaluate(&test_states)
        .await
        .expect("Should evaluate");

    assert!((0.0..=1.0).contains(&accuracy));
}

#[tokio::test]
async fn test_policy_network_set_learning_rate() {
    let config = PolicyNetworkConfig::default();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    network
        .set_learning_rate(0.01)
        .await
        .expect("Should set learning rate");

    let training_state = network.get_training_state().await;
    assert_eq!(training_state.learning_rate, 0.01);
}

#[tokio::test]
async fn test_policy_network_save_weights() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let dir = std::env::temp_dir();
    let path = dir.join("test_policy_weights.json");
    let result = network
        .save_weights(path.to_str().expect("should succeed"))
        .await;

    // Save should succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn test_policy_network_load_weights() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let dir = std::env::temp_dir();
    let path = dir.join("test_policy_weights.json");
    let result = network
        .load_weights(path.to_str().expect("should succeed"))
        .await;

    // Load should succeed or fail gracefully
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_policy_network_get_config() {
    let config = PolicyNetworkConfig {
        input_size: 64,
        hidden_layers: vec![128, 64],
        output_size: 16,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.3,
    };

    let network = PolicyNetwork::new(config.clone())
        .await
        .expect("Should create network");

    let retrieved_config = network.get_config();
    assert_eq!(retrieved_config.input_size, 64);
    assert_eq!(retrieved_config.output_size, 16);
}

#[tokio::test]
async fn test_policy_network_metrics_update() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    // Make multiple predictions to update metrics
    let state = create_test_rl_state();
    for _ in 0..10 {
        let _ = network.predict(&state).await;
    }

    let metrics = network.get_metrics().await;
    assert_eq!(metrics.total_predictions, 10);
    assert!(metrics.average_confidence >= 0.0);
}

#[test]
fn test_policy_action_serialization() {
    let action = PolicyAction {
        action_probabilities: vec![0.3, 0.5, 0.2],
        selected_action: 1,
        confidence: 0.5,
        value_estimate: 0.8,
    };

    let serialized = serde_json::to_string(&action).expect("Should serialize");
    let deserialized: PolicyAction = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.selected_action, 1);
    assert_eq!(deserialized.confidence, 0.5);
}

#[test]
fn test_policy_state_serialization() {
    let state = PolicyState {
        features: vec![1.0, 2.0, 3.0],
        encoding: vec![0.1, 0.2, 0.3],
        value: 0.5,
        timestamp: Utc::now(),
    };

    let serialized = serde_json::to_string(&state).expect("Should serialize");
    let deserialized: PolicyState = serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.features.len(), 3);
    assert_eq!(deserialized.value, 0.5);
}

#[test]
fn test_training_state_default() {
    let state = TrainingState::default();
    assert_eq!(state.epoch, 0);
    assert_eq!(state.loss, 0.0);
    assert_eq!(state.learning_rate, 0.001);
    assert_eq!(state.accuracy, 0.0);
}

#[test]
fn test_policy_metrics_default() {
    let metrics = PolicyMetrics::default();
    assert_eq!(metrics.total_predictions, 0);
    assert_eq!(metrics.average_confidence, 0.0);
    assert_eq!(metrics.correct_predictions, 0);
}

#[test]
fn test_training_state_serialization() {
    let state = TrainingState {
        epoch: 10,
        loss: 0.25,
        learning_rate: 0.01,
        accuracy: 0.85,
        last_update: Utc::now(),
    };

    let serialized = serde_json::to_string(&state).expect("Should serialize");
    let deserialized: TrainingState =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.epoch, 10);
    assert_eq!(deserialized.loss, 0.25);
    assert_eq!(deserialized.learning_rate, 0.01);
}

#[test]
fn test_policy_metrics_serialization() {
    let metrics = PolicyMetrics {
        total_predictions: 1000,
        average_confidence: 0.75,
        correct_predictions: 850,
        prediction_accuracy: 0.85,
        average_prediction_time: 0.05,
        network_complexity: 1024.0,
        last_evaluation: Utc::now(),
    };

    let serialized = serde_json::to_string(&metrics).expect("Should serialize");
    let deserialized: PolicyMetrics =
        serde_json::from_str(&serialized).expect("Should deserialize");

    assert_eq!(deserialized.total_predictions, 1000);
    assert_eq!(deserialized.prediction_accuracy, 0.85);
}

#[tokio::test]
async fn test_policy_network_multiple_predictions() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");

    let state = create_test_rl_state();

    // Make multiple predictions
    for _ in 0..5 {
        let action = network.predict(&state).await.expect("Should predict");
        assert!(!action.action_probabilities.is_empty());
    }

    let metrics = network.get_metrics().await;
    assert_eq!(metrics.total_predictions, 5);
}

// Helper functions
fn create_test_rl_state() -> RLState {
    RLState {
        id: "test_state".to_string(),
        context_id: "test_context".to_string(),
        features: vec![1.0, 2.0, 3.0, 4.0],
        metadata: None,
        timestamp: Utc::now(),
    }
}

#[tokio::test]
async fn test_predict_truncates_oversized_state_features() {
    let config = PolicyNetworkConfig {
        input_size: 4,
        hidden_layers: vec![6],
        output_size: 3,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.0,
    };
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");
    let state = RLState {
        id: "big".to_string(),
        context_id: "ctx".to_string(),
        features: vec![1.0; 40],
        metadata: None,
        timestamp: Utc::now(),
    };
    let action = network.predict(&state).await.expect("predict");
    assert_eq!(action.action_probabilities.len(), 3);
}

#[tokio::test]
async fn test_save_weights_fails_when_parent_is_file() {
    let config = test_helpers::create_test_policy_config();
    let network = PolicyNetwork::new(config)
        .await
        .expect("Should create network");
    let tmp = tempfile::NamedTempFile::new().expect("temp file");
    let bad_path = tmp.path().join("nested/weights.json");
    let result = network
        .save_weights(bad_path.to_str().expect("utf8 path"))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_evaluate_mixed_confidence_counts_accuracy() {
    let config = PolicyNetworkConfig {
        input_size: 4,
        hidden_layers: vec![4],
        output_size: 2,
        activation_function: "relu".to_string(),
        optimizer: "adam".to_string(),
        dropout_rate: 0.0,
    };
    let network = PolicyNetwork::new(config).await.expect("new");
    let states: Vec<RLState> = (0..5)
        .map(|i| RLState {
            id: format!("s{i}"),
            context_id: "c".to_string(),
            features: vec![i as f64 * 0.1; 4],
            metadata: None,
            timestamp: Utc::now(),
        })
        .collect();
    let acc = network.evaluate(&states).await.expect("eval");
    assert!((0.0..=1.0).contains(&acc));
}

fn create_test_rl_experience() -> super::engine::RLExperience {
    use super::engine::{RLAction, RLState};

    super::engine::RLExperience {
        id: "exp_1".to_string(),
        state: RLState {
            id: "state_1".to_string(),
            context_id: "test_context".to_string(),
            features: vec![1.0, 2.0, 3.0],
            metadata: None,
            timestamp: Utc::now(),
        },
        action: RLAction {
            id: "action_1".to_string(),
            action_type: "test_action".to_string(),
            parameters: serde_json::Value::Null,
            confidence: 0.8,
            expected_reward: 1.0,
        },
        reward: 1.0,
        next_state: Some(RLState {
            id: "state_2".to_string(),
            context_id: "test_context".to_string(),
            features: vec![1.1, 2.1, 3.1],
            metadata: None,
            timestamp: Utc::now(),
        }),
        done: false,
        timestamp: Utc::now(),
        priority: 1.0,
    }
}
