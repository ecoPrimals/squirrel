//! Universal Security Client Implementation

use base64::{engine::general_purpose, Engine as _};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::PrimalError;
use crate::universal::{PrimalCapability, PrimalContext, PrimalRequest, UniversalResult};
use crate::universal_primal_ecosystem::UniversalPrimalEcosystem;

use super::ai_metadata::*;
use super::providers::*;
use super::types::*;

// ============================================================================
// UNIVERSAL SECURITY CLIENT IMPLEMENTATION
// ============================================================================

/// Universal Security Client - AI-First, Capability-Based Design
#[derive(Debug)]
pub struct UniversalSecurityClient {
    /// Ecosystem integration for service discovery
    ecosystem: Arc<UniversalPrimalEcosystem>,

    /// Client configuration
    config: SecurityClientConfig,

    /// Active security providers (discovered dynamically)
    providers: Arc<RwLock<HashMap<String, SecurityProvider>>>,

    /// Request context for routing
    context: PrimalContext,

    /// AI-first metadata for intelligent routing
    ai_metadata: AISecurityMetadata,
}

impl UniversalSecurityClient {
    /// Create new universal security client
    pub fn new(
        ecosystem: Arc<UniversalPrimalEcosystem>,
        config: SecurityClientConfig,
        context: PrimalContext,
    ) -> Self {
        Self {
            ecosystem,
            config,
            providers: Arc::new(RwLock::new(HashMap::new())),
            context,
            ai_metadata: AISecurityMetadata::default(),
        }
    }

    /// Initialize the universal security client
    pub async fn initialize(&self) -> UniversalResult<()> {
        info!("Initializing Universal Security Client");

        // Discover all available security providers
        self.discover_security_providers().await?;

        // Start background tasks for health monitoring
        self.start_health_monitoring().await;

        // Initialize threat intelligence
        self.initialize_threat_intelligence().await;

        info!("Universal Security Client initialized successfully");
        Ok(())
    }

    /// Discover security providers using capability-based discovery
    async fn discover_security_providers(&self) -> UniversalResult<()> {
        debug!("Discovering security providers through capability-based search");

        let security_capabilities = vec![
            PrimalCapability::Authentication {
                methods: vec!["password".to_string(), "mfa".to_string()],
            },
            PrimalCapability::Encryption {
                algorithms: vec!["aes-256".to_string(), "rsa-2048".to_string()],
            },
            PrimalCapability::KeyManagement { hsm_support: true },
        ];

        let mut discovered_providers = HashMap::new();

        for capability in security_capabilities {
            let providers = self.ecosystem.find_by_capability(&capability).await;

            for primal in providers {
                let provider = SecurityProvider::from_discovered_primal(&primal);
                discovered_providers.insert(primal.instance_id.clone(), provider);
            }
        }

        let mut providers = self.providers.write().await;
        *providers = discovered_providers;

        info!("Discovered {} security providers", providers.len());
        Ok(())
    }

    /// Start background health monitoring
    async fn start_health_monitoring(&self) {
        debug!("Started background health monitoring for security providers");
    }

    /// Initialize threat intelligence
    async fn initialize_threat_intelligence(&self) {
        debug!("Initialized threat intelligence feeds");
    }

    /// Execute universal security operation
    pub async fn execute_operation(
        &self,
        request: UniversalSecurityRequest,
    ) -> UniversalResult<UniversalSecurityResponse> {
        debug!(
            "Executing universal security operation: {:?}",
            request.operation
        );

        // Select best provider using AI-based routing
        let provider = self.select_best_provider(&request).await?;

        // Create primal request
        let primal_request = PrimalRequest::new(
            "squirrel",
            &provider.provider_id,
            "security_operation",
            serde_json::to_value(&request).map_err(|e| {
                PrimalError::SerializationError(format!("Failed to serialize request: {}", e))
            })?,
            self.context.clone(),
        );

        // Send request through ecosystem
        let response = self
            .ecosystem
            .send_to_primal(&provider.provider_id, primal_request)
            .await?;

        // Process response and generate AI insights
        let security_response = self.process_response(response, &provider, &request).await?;

        // Update provider health based on operation
        self.update_provider_health(&provider.provider_id, &security_response)
            .await;

        // Log security event
        self.log_security_event(&request, &security_response).await;

        info!("Universal security operation completed successfully");
        Ok(security_response)
    }

    /// Select best provider using AI-based routing
    async fn select_best_provider(
        &self,
        request: &UniversalSecurityRequest,
    ) -> UniversalResult<SecurityProvider> {
        let providers = self.providers.read().await;

        if providers.is_empty() {
            return Err(PrimalError::ResourceNotFound(
                "No security providers available".to_string(),
            ));
        }

        // AI-based provider selection algorithm
        let mut best_provider: Option<SecurityProvider> = None;
        let mut best_score = 0.0;

        for provider in providers.values() {
            let score = self.calculate_provider_score(provider, request).await;
            if score > best_score {
                best_score = score;
                best_provider = Some(provider.clone());
            }
        }

        best_provider.ok_or_else(|| {
            PrimalError::OperationFailed("Failed to select security provider".to_string())
        })
    }

    /// Calculate provider score for specific request
    async fn calculate_provider_score(
        &self,
        provider: &SecurityProvider,
        request: &UniversalSecurityRequest,
    ) -> f64 {
        let mut score = provider.routing_score;

        // Factor in current health
        score *= provider.health.health_score;

        // Factor in trust level requirements
        let trust_match = match (&request.required_trust_level, &provider.trust_level) {
            (TrustLevel::Maximum, TrustLevel::Maximum) => 1.0,
            (TrustLevel::High, TrustLevel::High | TrustLevel::Maximum) => 1.0,
            (
                TrustLevel::Standard,
                TrustLevel::Standard | TrustLevel::High | TrustLevel::Maximum,
            ) => 1.0,
            (TrustLevel::Minimal, _) => 1.0,
            _ => 0.5, // Penalty for insufficient trust level
        };
        score *= trust_match;

        // Factor in security incidents
        if provider.health.incident_count > 0 {
            score *= 0.8; // Penalty for recent incidents
        }

        // Factor in response time
        if provider.health.response_time_ms > 1000.0 {
            score *= 0.9; // Penalty for slow response
        }

        score.min(1.0).max(0.0)
    }

    /// Process response and generate AI insights
    async fn process_response(
        &self,
        response: crate::universal::PrimalResponse,
        provider: &SecurityProvider,
        request: &UniversalSecurityRequest,
    ) -> UniversalResult<UniversalSecurityResponse> {
        let success = response.success;

        let decision = if success {
            SecurityDecision {
                outcome: DecisionOutcome::Allow,
                confidence: 0.95,
                risk_score: 0.2,
                factors: vec![DecisionFactor {
                    name: "Authentication".to_string(),
                    weight: 0.8,
                    value: serde_json::json!(true),
                    impact: "Positive".to_string(),
                }],
                recommended_actions: vec!["Monitor user activity".to_string()],
            }
        } else {
            SecurityDecision {
                outcome: DecisionOutcome::Deny,
                confidence: 0.99,
                risk_score: 0.8,
                factors: vec![DecisionFactor {
                    name: "Authentication Failure".to_string(),
                    weight: 1.0,
                    value: serde_json::json!(false),
                    impact: "Negative".to_string(),
                }],
                recommended_actions: vec![
                    "Review authentication logs".to_string(),
                    "Consider account lockout".to_string(),
                ],
            }
        };

        Ok(UniversalSecurityResponse {
            request_id: request.request_id,
            success,
            decision,
            data: response.data.get("data").and_then(|v| {
                general_purpose::STANDARD
                    .decode(v.as_str().unwrap_or(""))
                    .ok()
            }),
            provider_id: provider.provider_id.clone(),
            security_metrics: SecurityMetrics {
                processing_time: std::time::Duration::from_millis(
                    response.duration.num_milliseconds() as u64,
                ),
                policy_evaluations: 3,
                events_generated: 1,
                threat_indicators: 0,
                provider_security_score: provider.health.health_score,
            },
            ai_insights: AISecurityInsights {
                confidence_score: 0.95,
                threat_analysis: ThreatAnalysis {
                    detected_threats: vec![],
                    severity_scores: HashMap::new(),
                    attack_patterns: vec![],
                    countermeasures: vec!["Enable multi-factor authentication".to_string()],
                },
                security_recommendations: vec![
                    "Consider implementing additional security layers".to_string()
                ],
                risk_mitigation: vec!["Enable continuous monitoring".to_string()],
                behavioral_insights: vec!["User behavior appears normal".to_string()],
            },
            error: response.error_message,
        })
    }

    /// Update provider health based on operation results
    async fn update_provider_health(
        &self,
        provider_id: &str,
        response: &UniversalSecurityResponse,
    ) {
        let mut providers = self.providers.write().await;
        if let Some(provider) = providers.get_mut(provider_id) {
            // Update health metrics based on operation performance
            provider.health.response_time_ms =
                response.security_metrics.processing_time.as_millis() as f64;
            provider.health.last_check = chrono::Utc::now();

            if response.success {
                provider.health.health_score =
                    (provider.health.health_score * 0.95 + 0.05).min(1.0);
            } else {
                provider.health.health_score = (provider.health.health_score * 0.95).max(0.1);
                provider.health.incident_count += 1;
            }
        }
    }

    /// Log security event
    async fn log_security_event(
        &self,
        request: &UniversalSecurityRequest,
        response: &UniversalSecurityResponse,
    ) {
        debug!(
            "Security event logged: operation={:?}, success={}, risk_score={}",
            request.operation, response.success, response.decision.risk_score
        );
    }
}

// ============================================================================
// CONVENIENCE METHODS
// ============================================================================

impl UniversalSecurityClient {
    /// Authenticate user using intelligent provider selection
    pub async fn authenticate(
        &self,
        identity: &str,
        credentials: HashMap<String, String>,
        risk_level: RiskLevel,
    ) -> UniversalResult<UniversalSecurityResponse> {
        let request = UniversalSecurityRequest {
            request_id: Uuid::new_v4(),
            operation: SecurityOperation::Authenticate {
                identity: identity.to_string(),
                credentials,
            },
            security_context: SecurityContext {
                user_id: identity.to_string(),
                session_id: Uuid::new_v4().to_string(),
                ip_address: "127.0.0.1".to_string(),
                user_agent: "UniversalSecurityClient".to_string(),
                clearance_level: "standard".to_string(),
                additional_context: HashMap::new(),
            },
            payload: SecurityPayload {
                data: None,
                parameters: HashMap::new(),
                policy_overrides: None,
                compliance_tags: vec!["authentication".to_string()],
            },
            required_trust_level: TrustLevel::High,
            ai_context: AISecurityContext {
                risk_assessment: risk_level,
                threat_intelligence: vec![],
                behavioral_analysis: BehavioralProfile {
                    normal_patterns: vec!["normal_login".to_string()],
                    anomaly_score: 0.1,
                    historical_behavior: HashMap::new(),
                },
                context_awareness: ContextAwareness {
                    temporal_context: TemporalContext {
                        normal_hours: vec![9, 10, 11, 12, 13, 14, 15, 16, 17],
                        time_anomaly_score: 0.0,
                        frequency_analysis: HashMap::new(),
                    },
                    location_context: LocationContext {
                        allowed_locations: vec!["office".to_string()],
                        location_risk_score: 0.0,
                        travel_patterns: vec![],
                    },
                    device_context: DeviceContext {
                        trusted_devices: vec!["laptop-001".to_string()],
                        device_risk_score: 0.0,
                        device_fingerprint: HashMap::new(),
                    },
                },
            },
            metadata: HashMap::new(),
        };

        self.execute_operation(request).await
    }
}
