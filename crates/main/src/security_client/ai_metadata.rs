//! AI Metadata and Intelligence Types for Security Client

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// AI METADATA TYPES
// ============================================================================

/// AI-first metadata for intelligent security decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISecurityMetadata {
    /// Confidence in provider selection
    pub provider_confidence: f64,
    
    /// Threat level assessments
    pub threat_assessments: Vec<ThreatAssessment>,
    
    /// Security recommendations
    pub security_recommendations: Vec<String>,
    
    /// Risk analysis
    pub risk_analysis: RiskAnalysis,
}

/// Threat assessment for security operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessment {
    /// Threat type
    pub threat_type: String,
    
    /// Threat level (0.0 - 1.0)
    pub threat_level: f64,
    
    /// Confidence in assessment
    pub confidence: f64,
    
    /// Mitigation recommendations
    pub mitigations: Vec<String>,
}

/// Risk analysis for security decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysis {
    /// Overall risk score (0.0 - 1.0)
    pub overall_risk: f64,
    
    /// Risk categories
    pub risk_categories: HashMap<String, f64>,
    
    /// Risk mitigation strategies
    pub mitigation_strategies: Vec<String>,
    
    /// Confidence in analysis
    pub confidence: f64,
}

impl Default for AISecurityMetadata {
    fn default() -> Self {
        Self {
            provider_confidence: 0.9,
            threat_assessments: Vec::new(),
            security_recommendations: Vec::new(),
            risk_analysis: RiskAnalysis {
                overall_risk: 0.3,
                risk_categories: HashMap::new(),
                mitigation_strategies: Vec::new(),
                confidence: 0.8,
            },
        }
    }
} 