//! Tests for security client types

#[cfg(test)]
mod tests {
    use crate::security_client::types::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[test]
    fn test_security_client_config_default() {
        let config = SecurityClientConfig::default();

        assert_eq!(config.max_retries, 3);
        assert_eq!(config.operation_timeout.as_secs(), 30);
        assert!(!config.preferred_capabilities.is_empty());
        assert!(config.policy_requirements.encryption_policy.data_at_rest);
    }

    #[test]
    fn test_trust_level_variants() {
        let minimal = TrustLevel::Minimal;
        let standard = TrustLevel::Standard;
        let high = TrustLevel::High;
        let maximum = TrustLevel::Maximum;

        assert!(matches!(minimal, TrustLevel::Minimal));
        assert!(matches!(standard, TrustLevel::Standard));
        assert!(matches!(high, TrustLevel::High));
        assert!(matches!(maximum, TrustLevel::Maximum));
    }

    #[test]
    fn test_access_level_variants() {
        let none = AccessLevel::None;
        let readonly = AccessLevel::ReadOnly;
        let readwrite = AccessLevel::ReadWrite;
        let admin = AccessLevel::Admin;
        let full = AccessLevel::Full;

        assert!(matches!(none, AccessLevel::None));
        assert!(matches!(readonly, AccessLevel::ReadOnly));
        assert!(matches!(readwrite, AccessLevel::ReadWrite));
        assert!(matches!(admin, AccessLevel::Admin));
        assert!(matches!(full, AccessLevel::Full));
    }

    #[test]
    fn test_risk_level_variants() {
        let very_low = RiskLevel::VeryLow;
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;
        let critical = RiskLevel::Critical;

        assert!(matches!(very_low, RiskLevel::VeryLow));
        assert!(matches!(low, RiskLevel::Low));
        assert!(matches!(medium, RiskLevel::Medium));
        assert!(matches!(high, RiskLevel::High));
        assert!(matches!(critical, RiskLevel::Critical));
    }

    #[test]
    fn test_compliance_framework_variants() {
        let soc2 = ComplianceFramework::Soc2;
        let iso = ComplianceFramework::Iso27001;
        let gdpr = ComplianceFramework::Gdpr;
        let hipaa = ComplianceFramework::Hipaa;
        let pci = ComplianceFramework::PciDss;
        let nist = ComplianceFramework::NistCsf;
        let custom = ComplianceFramework::Custom("MyFramework".to_string());

        assert!(matches!(soc2, ComplianceFramework::Soc2));
        assert!(matches!(iso, ComplianceFramework::Iso27001));
        assert!(matches!(gdpr, ComplianceFramework::Gdpr));
        assert!(matches!(hipaa, ComplianceFramework::Hipaa));
        assert!(matches!(pci, ComplianceFramework::PciDss));
        assert!(matches!(nist, ComplianceFramework::NistCsf));
        assert!(matches!(custom, ComplianceFramework::Custom(_)));
    }

    #[test]
    fn test_security_capability_type_authentication() {
        let auth = SecurityCapabilityType::Authentication {
            methods: vec!["password".to_string(), "mfa".to_string()],
            multi_factor: true,
            biometric_support: false,
        };

        match auth {
            SecurityCapabilityType::Authentication {
                methods,
                multi_factor,
                biometric_support,
            } => {
                assert_eq!(methods.len(), 2);
                assert!(multi_factor);
                assert!(!biometric_support);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_security_capability_type_authorization() {
        let authz = SecurityCapabilityType::Authorization {
            rbac_support: true,
            abac_support: false,
            policy_engines: vec!["opa".to_string()],
        };

        match authz {
            SecurityCapabilityType::Authorization {
                rbac_support,
                abac_support,
                policy_engines,
            } => {
                assert!(rbac_support);
                assert!(!abac_support);
                assert_eq!(policy_engines.len(), 1);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_security_capability_type_encryption() {
        let enc = SecurityCapabilityType::Encryption {
            algorithms: vec!["AES-256".to_string()],
            key_management: true,
            hardware_security_modules: false,
        };

        match enc {
            SecurityCapabilityType::Encryption {
                algorithms,
                key_management,
                hardware_security_modules,
            } => {
                assert_eq!(algorithms.len(), 1);
                assert!(key_management);
                assert!(!hardware_security_modules);
            }
            _ => panic!("Wrong capability type"),
        }
    }

    #[test]
    fn test_security_operation_authenticate() {
        let mut creds = HashMap::new();
        creds.insert("username".to_string(), "test".to_string());

        let op = SecurityOperation::Authenticate {
            identity: "user@example.com".to_string(),
            credentials: creds,
        };

        match op {
            SecurityOperation::Authenticate {
                identity,
                credentials,
            } => {
                assert_eq!(identity, "user@example.com");
                assert!(credentials.contains_key("username"));
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_security_operation_authorize() {
        let op = SecurityOperation::Authorize {
            subject: "user123".to_string(),
            resource: "/api/data".to_string(),
            action: "read".to_string(),
        };

        match op {
            SecurityOperation::Authorize {
                subject,
                resource,
                action,
            } => {
                assert_eq!(subject, "user123");
                assert_eq!(resource, "/api/data");
                assert_eq!(action, "read");
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_security_operation_encrypt() {
        let op = SecurityOperation::Encrypt {
            algorithm: "AES-256-GCM".to_string(),
            key_id: Some("key123".to_string()),
        };

        match op {
            SecurityOperation::Encrypt { algorithm, key_id } => {
                assert_eq!(algorithm, "AES-256-GCM");
                assert_eq!(key_id, Some("key123".to_string()));
            }
            _ => panic!("Wrong operation type"),
        }
    }

    #[test]
    fn test_security_context_creation() {
        let mut additional = HashMap::new();
        additional.insert("organization".to_string(), "acme".to_string());

        let context = SecurityContext {
            user_id: "user123".to_string(),
            session_id: "sess456".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            clearance_level: "standard".to_string(),
            additional_context: additional,
        };

        assert_eq!(context.user_id, "user123");
        assert_eq!(context.ip_address, "192.168.1.1");
        assert!(context.additional_context.contains_key("organization"));
    }

    #[test]
    fn test_security_payload_creation() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), serde_json::json!("value"));

        let payload = SecurityPayload {
            data: Some(vec![1, 2, 3]),
            parameters: params,
            policy_overrides: None,
            compliance_tags: vec!["gdpr".to_string()],
        };

        assert_eq!(payload.data, Some(vec![1, 2, 3]));
        assert!(payload.parameters.contains_key("key"));
        assert_eq!(payload.compliance_tags.len(), 1);
    }

    #[test]
    fn test_threat_indicator_creation() {
        let indicator = ThreatIndicator {
            indicator_type: "ip".to_string(),
            value: "192.168.1.100".to_string(),
            confidence: 0.85,
            source: "threat_feed".to_string(),
        };

        assert_eq!(indicator.indicator_type, "ip");
        assert_eq!(indicator.confidence, 0.85);
    }

    #[test]
    fn test_behavioral_profile_creation() {
        let mut historical = HashMap::new();
        historical.insert("login_frequency".to_string(), 0.9);

        let profile = BehavioralProfile {
            normal_patterns: vec!["weekday_login".to_string()],
            anomaly_score: 0.15,
            historical_behavior: historical,
        };

        assert_eq!(profile.anomaly_score, 0.15);
        assert_eq!(profile.normal_patterns.len(), 1);
    }

    #[test]
    fn test_temporal_context_creation() {
        let mut freq = HashMap::new();
        freq.insert("hourly".to_string(), 0.8);

        let temporal = TemporalContext {
            normal_hours: vec![9, 10, 11, 12, 13, 14, 15, 16, 17],
            time_anomaly_score: 0.1,
            frequency_analysis: freq,
        };

        assert_eq!(temporal.normal_hours.len(), 9);
        assert_eq!(temporal.time_anomaly_score, 0.1);
    }

    #[test]
    fn test_location_context_creation() {
        let location = LocationContext {
            allowed_locations: vec!["US".to_string(), "CA".to_string()],
            location_risk_score: 0.2,
            travel_patterns: vec!["business".to_string()],
        };

        assert_eq!(location.allowed_locations.len(), 2);
        assert_eq!(location.location_risk_score, 0.2);
    }

    #[test]
    fn test_device_context_creation() {
        let mut fingerprint = HashMap::new();
        fingerprint.insert("browser".to_string(), "Chrome".to_string());

        let device = DeviceContext {
            trusted_devices: vec!["device123".to_string()],
            device_risk_score: 0.05,
            device_fingerprint: fingerprint,
        };

        assert_eq!(device.trusted_devices.len(), 1);
        assert_eq!(device.device_risk_score, 0.05);
    }

    #[test]
    fn test_decision_outcome_allow() {
        let outcome = DecisionOutcome::Allow;
        assert!(matches!(outcome, DecisionOutcome::Allow));
    }

    #[test]
    fn test_decision_outcome_deny() {
        let outcome = DecisionOutcome::Deny;
        assert!(matches!(outcome, DecisionOutcome::Deny));
    }

    #[test]
    fn test_decision_outcome_allow_with_conditions() {
        let outcome = DecisionOutcome::AllowWithConditions {
            conditions: vec!["mfa_required".to_string()],
        };

        match outcome {
            DecisionOutcome::AllowWithConditions { conditions } => {
                assert_eq!(conditions.len(), 1);
            }
            _ => panic!("Wrong outcome type"),
        }
    }

    #[test]
    fn test_decision_factor_creation() {
        let factor = DecisionFactor {
            name: "time_of_day".to_string(),
            weight: 0.3,
            value: serde_json::json!("business_hours"),
            impact: "positive".to_string(),
        };

        assert_eq!(factor.name, "time_of_day");
        assert_eq!(factor.weight, 0.3);
    }

    #[test]
    fn test_security_metrics_creation() {
        let metrics = SecurityMetrics {
            processing_time: std::time::Duration::from_millis(150),
            policy_evaluations: 5,
            events_generated: 2,
            threat_indicators: 3,
            provider_security_score: 0.95,
        };

        assert_eq!(metrics.policy_evaluations, 5);
        assert_eq!(metrics.provider_security_score, 0.95);
    }

    #[test]
    fn test_threat_analysis_creation() {
        let mut severity = HashMap::new();
        severity.insert("sql_injection".to_string(), 0.8);

        let analysis = ThreatAnalysis {
            detected_threats: vec!["sql_injection".to_string()],
            severity_scores: severity,
            attack_patterns: vec!["pattern1".to_string()],
            countermeasures: vec!["input_validation".to_string()],
        };

        assert_eq!(analysis.detected_threats.len(), 1);
        assert!(analysis.severity_scores.contains_key("sql_injection"));
    }

    #[test]
    fn test_universal_security_request_serialization() {
        let mut params = HashMap::new();
        params.insert("test".to_string(), serde_json::json!("value"));

        let request = UniversalSecurityRequest {
            request_id: Uuid::new_v4(),
            operation: SecurityOperation::Authorize {
                subject: "user".to_string(),
                resource: "/api".to_string(),
                action: "read".to_string(),
            },
            security_context: SecurityContext {
                user_id: "user123".to_string(),
                session_id: "sess123".to_string(),
                ip_address: "127.0.0.1".to_string(),
                user_agent: "test".to_string(),
                clearance_level: "standard".to_string(),
                additional_context: HashMap::new(),
            },
            payload: SecurityPayload {
                data: None,
                parameters: params,
                policy_overrides: None,
                compliance_tags: vec![],
            },
            required_trust_level: TrustLevel::Standard,
            ai_context: AISecurityContext {
                risk_assessment: RiskLevel::Low,
                threat_intelligence: vec![],
                behavioral_analysis: BehavioralProfile {
                    normal_patterns: vec![],
                    anomaly_score: 0.0,
                    historical_behavior: HashMap::new(),
                },
                context_awareness: ContextAwareness {
                    temporal_context: TemporalContext {
                        normal_hours: vec![],
                        time_anomaly_score: 0.0,
                        frequency_analysis: HashMap::new(),
                    },
                    location_context: LocationContext {
                        allowed_locations: vec![],
                        location_risk_score: 0.0,
                        travel_patterns: vec![],
                    },
                    device_context: DeviceContext {
                        trusted_devices: vec![],
                        device_risk_score: 0.0,
                        device_fingerprint: HashMap::new(),
                    },
                },
            },
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("user123"));

        let deserialized: UniversalSecurityRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.security_context.user_id, "user123");
    }

    #[test]
    fn test_encryption_policy_creation() {
        let policy = EncryptionPolicy {
            data_at_rest: true,
            data_in_transit: true,
            data_in_use: false,
            key_rotation_days: 90,
            min_key_strength: 256,
        };

        assert!(policy.data_at_rest);
        assert!(policy.data_in_transit);
        assert_eq!(policy.key_rotation_days, 90);
        assert_eq!(policy.min_key_strength, 256);
    }

    #[test]
    fn test_audit_policy_creation() {
        let policy = AuditPolicy {
            log_all_events: true,
            retention_days: 365,
            real_time_alerts: true,
            compliance_reporting: true,
        };

        assert!(policy.log_all_events);
        assert_eq!(policy.retention_days, 365);
        assert!(policy.real_time_alerts);
    }

    #[test]
    fn test_access_control_policy_creation() {
        let policy = AccessControlPolicy {
            default_access: AccessLevel::None,
            rbac_enabled: true,
            abac_enabled: false,
            zero_trust: true,
        };

        assert!(matches!(policy.default_access, AccessLevel::None));
        assert!(policy.rbac_enabled);
        assert!(policy.zero_trust);
    }
}
