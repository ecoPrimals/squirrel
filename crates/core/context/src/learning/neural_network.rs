// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

//! Small feed-forward policy head used for Deep Q-style action selection.

use rand::Rng;
use tracing::debug;

/// Small feed-forward policy head used for Deep Q-style action selection in-process.
///
/// This is a **real** (but lightweight) MLP: ReLU hidden layer and linear readout to one logit per
/// discrete action. Training does not run full backprop through an external ML framework; the
/// engine still records replay batches and metrics, while tabular Q-learning carries the primary
/// value updates. A future Phase-2 integration can swap this struct for a framework-backed
/// network without changing the surrounding RL loop structure.
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    /// Layer widths: `[input_dim, hidden_dim, output_dim]` (output = discrete actions).
    pub layers: Vec<usize>,

    /// Hidden layer weights: `hidden × input` row-major (`weights[i][j]`).
    pub weights: Vec<Vec<f64>>,

    /// Readout layer: `output × hidden` (`readout_weights[k][i]` × hidden unit `i`).
    pub readout_weights: Vec<Vec<f64>>,

    /// `biases[0]`: hidden biases; `biases[1]`: output biases.
    pub biases: Vec<Vec<f64>>,

    /// Label for logging / future activation variants (`relu` used in [`Self::forward_scores`]).
    pub activation: String,
}

impl NeuralNetwork {
    pub(super) fn new_mlp(input_dim: usize, hidden: usize, output: usize) -> Self {
        let mut rng = rand::rng();
        let scale_in = 1.0 / (input_dim as f64).sqrt();
        let w1: Vec<Vec<f64>> = (0..hidden)
            .map(|_| {
                (0..input_dim)
                    .map(|_| rng.random_range(-scale_in..scale_in))
                    .collect()
            })
            .collect();
        let b0: Vec<f64> = (0..hidden).map(|_| rng.random_range(-0.01..0.01)).collect();
        let scale_h = 1.0 / (hidden as f64).sqrt();
        let w2: Vec<Vec<f64>> = (0..output)
            .map(|_| {
                (0..hidden)
                    .map(|_| rng.random_range(-scale_h..scale_h))
                    .collect()
            })
            .collect();
        let b1: Vec<f64> = (0..output).map(|_| rng.random_range(-0.01..0.01)).collect();
        Self {
            layers: vec![input_dim, hidden, output],
            weights: w1,
            readout_weights: w2,
            biases: vec![b0, b1],
            activation: "relu".to_string(),
        }
    }

    fn pad_or_trunc(features: &[f64], len: usize) -> Vec<f64> {
        let mut v = features.to_vec();
        if v.len() < len {
            v.resize(len, 0.0);
        } else if v.len() > len {
            v.truncate(len);
        }
        v
    }

    fn relu(x: f64) -> f64 {
        x.max(0.0)
    }

    /// Forward pass: returns one score per discrete action (readout size from `layers[2]`).
    pub(crate) fn forward_scores(&self, features: &[f64]) -> Vec<f64> {
        if self.activation != "relu" {
            debug!(
                activation = %self.activation,
                "Policy head uses ReLU hidden units; non-ReLU label is informational only"
            );
        }
        let input_dim = self.layers.first().copied().unwrap_or(1).max(1);
        let hidden = self.layers.get(1).copied().unwrap_or(32);
        let output = self.layers.get(2).copied().unwrap_or(6);
        let x = Self::pad_or_trunc(features, input_dim);
        let b0 = self.biases.first().map_or(&[] as &[f64], Vec::as_slice);
        let mut h = Vec::with_capacity(hidden);
        for i in 0..hidden {
            let row = self.weights.get(i);
            let sum: f64 = row.map_or(0.0, |rw| {
                rw.iter().zip(x.iter()).map(|(a, b)| a * b).sum::<f64>()
                    + b0.get(i).copied().unwrap_or(0.0)
            });
            h.push(Self::relu(sum));
        }
        let b1 = self.biases.get(1).map_or(&[] as &[f64], Vec::as_slice);
        let mut logits = Vec::with_capacity(output);
        for k in 0..output {
            let mut sum = 0.0f64;
            let row = self.readout_weights.get(k);
            for (i, h_unit) in h.iter().enumerate().take(hidden) {
                let w = row.and_then(|r| r.get(i)).copied().unwrap_or(0.0);
                sum += w * h_unit;
            }
            sum += b1.get(k).copied().unwrap_or(0.0);
            logits.push(sum);
        }
        logits
    }
}
