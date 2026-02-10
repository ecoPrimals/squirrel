// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 DataScienceBioLab

//! Context Analysis and Classification

use serde_json::json;

use super::core::SquirrelPrimalProvider;
use crate::error::PrimalError;

/// Context Analysis functionality
pub struct ContextAnalysis;

impl ContextAnalysis {
    /// Perform sentiment analysis on text
    #[must_use]
    pub fn analyze_sentiment(text: &str) -> SentimentResult {
        // Simple heuristic sentiment analysis (would use ML models in production)
        let positive_words = [
            "good",
            "great",
            "excellent",
            "amazing",
            "wonderful",
            "fantastic",
        ];
        let negative_words = [
            "bad",
            "terrible",
            "awful",
            "horrible",
            "disappointing",
            "poor",
        ];

        let text_lower = text.to_lowercase();
        let positive_count = positive_words
            .iter()
            .filter(|&&word| text_lower.contains(word))
            .count();
        let negative_count = negative_words
            .iter()
            .filter(|&&word| text_lower.contains(word))
            .count();

        let sentiment = match (positive_count, negative_count) {
            (p, n) if p > n => "positive",
            (p, n) if n > p => "negative",
            _ => "neutral",
        };

        let confidence = ((positive_count + negative_count) as f64
            / text.split_whitespace().count() as f64)
            .min(1.0);

        SentimentResult {
            sentiment: sentiment.to_string(),
            confidence,
            positive_score: positive_count as f64,
            negative_score: negative_count as f64,
        }
    }

    /// Classify intent from user input using keyword matching
    ///
    /// Confidence is computed based on keyword density in the input text.
    /// Higher keyword density → higher confidence in the classification.
    #[must_use]
    pub fn classify_intent(text: &str) -> IntentResult {
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        let word_count = words.len().max(1);

        // Count matching keywords per intent category and pick the best
        let categories: &[(&str, &[&str])] = &[
            ("help_request", &["help", "assist", "support", "guide"]),
            ("creation", &["create", "make", "build", "new", "generate"]),
            ("search", &["search", "find", "look", "query", "locate"]),
            ("deletion", &["delete", "remove", "drop", "clear", "purge"]),
            (
                "modification",
                &["update", "modify", "change", "edit", "alter"],
            ),
            (
                "question",
                &["what", "how", "why", "when", "where", "which"],
            ),
        ];

        let mut best_intent = "general";
        let mut best_match_count = 0usize;

        for &(intent_name, keywords) in categories {
            let match_count = words.iter().filter(|w| keywords.contains(w)).count();
            if match_count > best_match_count {
                best_match_count = match_count;
                best_intent = intent_name;
            }
        }

        // Confidence: base 0.5 for keyword match, scaled by keyword density
        let confidence = if best_match_count > 0 {
            (0.5 + 0.5 * (best_match_count as f64 / word_count as f64)).min(0.95)
        } else {
            0.3 // Low confidence for "general" fallback
        };

        IntentResult {
            intent: best_intent.to_string(),
            confidence,
            entities: extract_entities(&text_lower),
        }
    }

    /// Extract entities from text
    #[must_use]
    pub fn extract_entities(text: &str) -> Vec<Entity> {
        extract_entities(text)
    }

    /// Detect topic from text
    #[must_use]
    pub fn detect_topic(text: &str) -> TopicResult {
        let text_lower = text.to_lowercase();

        let topic = if text_lower.contains("code") || text_lower.contains("programming") {
            "programming"
        } else if text_lower.contains("data") || text_lower.contains("analysis") {
            "data_analysis"
        } else if text_lower.contains("business") || text_lower.contains("management") {
            "business"
        } else if text_lower.contains("science") || text_lower.contains("research") {
            "science"
        } else if text_lower.contains("health") || text_lower.contains("medical") {
            "healthcare"
        } else {
            "general"
        };

        TopicResult {
            topic: topic.to_string(),
            confidence: 0.7,
            keywords: extract_keywords(&text_lower),
        }
    }
}

/// Extract entities from text (simple implementation)
fn extract_entities(text: &str) -> Vec<Entity> {
    let mut entities = Vec::new();

    // Simple pattern matching for common entity types
    let words: Vec<&str> = text.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        // Email detection
        if word.contains('@') && word.contains('.') {
            entities.push(Entity {
                entity_type: "email".to_string(),
                value: (*word).to_string(),
                start: i,
                end: i + 1,
                confidence: 0.9,
            });
        }

        // Number detection
        if word.parse::<f64>().is_ok() {
            entities.push(Entity {
                entity_type: "number".to_string(),
                value: (*word).to_string(),
                start: i,
                end: i + 1,
                confidence: 0.95,
            });
        }
    }

    entities
}

/// Extract keywords from text
fn extract_keywords(text: &str) -> Vec<String> {
    let stop_words = [
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
    ];

    text.split_whitespace()
        .filter(|word| word.len() > 3 && !stop_words.contains(&word.to_lowercase().as_str()))
        .take(5) // Top 5 keywords
        .map(str::to_lowercase)
        .collect()
}

// Data structures for analysis results
#[derive(Debug, Clone)]
pub struct SentimentResult {
    pub sentiment: String,
    pub confidence: f64,
    pub positive_score: f64,
    pub negative_score: f64,
}

#[derive(Debug, Clone)]
pub struct IntentResult {
    pub intent: String,
    pub confidence: f64,
    pub entities: Vec<Entity>,
}

#[derive(Debug, Clone)]
pub struct Entity {
    pub entity_type: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct TopicResult {
    pub topic: String,
    pub confidence: f64,
    pub keywords: Vec<String>,
}

impl SquirrelPrimalProvider {
    /// Handle context analysis request
    pub async fn handle_context_analysis_request(
        &self,
        request: serde_json::Value,
    ) -> Result<serde_json::Value, PrimalError> {
        let text = request
            .get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PrimalError::ValidationError("Missing text field".to_string()))?;

        let analysis_type = request
            .get("analysis_type")
            .and_then(|v| v.as_str())
            .unwrap_or("full");

        match analysis_type {
            "sentiment" => {
                let result = ContextAnalysis::analyze_sentiment(text);
                Ok(json!({
                    "analysis_type": "sentiment",
                    "sentiment": result.sentiment,
                    "confidence": result.confidence,
                    "positive_score": result.positive_score,
                    "negative_score": result.negative_score
                }))
            }
            "intent" => {
                let result = ContextAnalysis::classify_intent(text);
                Ok(json!({
                    "analysis_type": "intent",
                    "intent": result.intent,
                    "confidence": result.confidence,
                    "entities": result.entities.iter().map(|e| json!({
                        "type": e.entity_type,
                        "value": e.value,
                        "start": e.start,
                        "end": e.end,
                        "confidence": e.confidence
                    })).collect::<Vec<_>>()
                }))
            }
            "topic" => {
                let result = ContextAnalysis::detect_topic(text);
                Ok(json!({
                    "analysis_type": "topic",
                    "topic": result.topic,
                    "confidence": result.confidence,
                    "keywords": result.keywords
                }))
            }
            "full" => {
                let sentiment = ContextAnalysis::analyze_sentiment(text);
                let intent = ContextAnalysis::classify_intent(text);
                let topic = ContextAnalysis::detect_topic(text);

                Ok(json!({
                    "analysis_type": "full",
                    "sentiment": {
                        "sentiment": sentiment.sentiment,
                        "confidence": sentiment.confidence,
                        "positive_score": sentiment.positive_score,
                        "negative_score": sentiment.negative_score
                    },
                    "intent": {
                        "intent": intent.intent,
                        "confidence": intent.confidence,
                        "entities": intent.entities.iter().map(|e| json!({
                            "type": e.entity_type,
                            "value": e.value,
                            "start": e.start,
                            "end": e.end,
                            "confidence": e.confidence
                        })).collect::<Vec<_>>()
                    },
                    "topic": {
                        "topic": topic.topic,
                        "confidence": topic.confidence,
                        "keywords": topic.keywords
                    }
                }))
            }
            _ => Err(PrimalError::ValidationError(format!(
                "Unknown analysis type: {analysis_type}"
            ))),
        }
    }
}
