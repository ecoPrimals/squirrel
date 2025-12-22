//! Tests for ecosystem registry discovery operations

#[cfg(test)]
mod tests {
    use crate::ecosystem::registry::discovery::DiscoveryOps;
    use crate::ecosystem::registry::types::*;
    use crate::ecosystem::EcosystemPrimalType;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    #[test]
    fn test_build_service_endpoint_squirrel() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::Squirrel);
        assert!(endpoint.contains("http"));
        assert!(!endpoint.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_build_service_endpoint_songbird() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::Songbird);
        assert!(endpoint.contains("http"));
    }

    #[test]
    fn test_build_service_endpoint_biomeos() {
        let endpoint = DiscoveryOps::build_service_endpoint(&EcosystemPrimalType::BiomeOS);
        assert!(endpoint.contains("http"));
    }

    #[test]
    fn test_build_service_endpoint_custom() {
        let custom_type = EcosystemPrimalType::Custom("test_primal".to_string());
        let endpoint = DiscoveryOps::build_service_endpoint(&custom_type);
        assert!(!endpoint.is_empty());
    }

    #[test]
    fn test_get_capabilities_squirrel() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::Squirrel);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "ai_coordination"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_songbird() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::Songbird);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "service_mesh"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_toadstool() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::ToadStool);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "compute" || &**c == "storage"));
    }

    #[test]
    #[allow(deprecated)]
    fn test_get_capabilities_beardog() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::BearDog);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "security"));
    }

    #[test]
    fn test_get_capabilities_biomeos() {
        let caps = DiscoveryOps::get_capabilities_for_primal(&EcosystemPrimalType::BiomeOS);
        assert!(caps.len() > 0);
        assert!(caps.iter().any(|c| &**c == "operating_system"));
    }

    #[test]
    fn test_get_capabilities_custom() {
        let custom_type = EcosystemPrimalType::Custom("test".to_string());
        let caps = DiscoveryOps::get_capabilities_for_primal(&custom_type);
        assert_eq!(caps.len(), 1);
        assert_eq!(&*caps[0], "discovery");
    }

    #[tokio::test]
    async fn test_discover_services_empty() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_discover_services_single() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[allow(deprecated)]
    async fn test_discover_services_multiple() {
        let registry = Arc::new(RwLock::new(HashMap::new()));
        let primal_types = vec![EcosystemPrimalType::Squirrel, EcosystemPrimalType::Songbird];

        let result = DiscoveryOps::discover_services(&registry, primal_types).await;
        assert!(result.is_ok());
    }

    #[test]
    #[allow(deprecated)]
    fn test_all_primal_types_have_endpoints() {
        let types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        for primal_type in types {
            let endpoint = DiscoveryOps::build_service_endpoint(&primal_type);
            assert!(!endpoint.is_empty());
            assert!(endpoint.starts_with("http"));
        }
    }

    #[test]
    #[allow(deprecated)]
    fn test_all_primal_types_have_capabilities() {
        let types = vec![
            EcosystemPrimalType::Squirrel,
            EcosystemPrimalType::Songbird,
            EcosystemPrimalType::ToadStool,
            EcosystemPrimalType::BearDog,
            EcosystemPrimalType::NestGate,
            EcosystemPrimalType::BiomeOS,
        ];

        for primal_type in types {
            let caps = DiscoveryOps::get_capabilities_for_primal(&primal_type);
            assert!(!caps.is_empty());
        }
    }
}
