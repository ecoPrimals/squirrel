// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

//! Authentication and authorization types.

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Credentials for authentication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Credentials {
    /// Username and password
    Password {
        /// Username for authentication
        username: String,
        /// Password for authentication
        password: String,
    },
    /// API key
    ApiKey {
        /// The API key
        key: String,
        /// Service ID for the API key
        service_id: String,
    },
    /// Bearer token
    Bearer {
        /// Bearer token string
        token: String,
    },
    /// JWT token
    Token {
        /// JWT token string
        token: String,
    },
    /// Certificate
    Certificate {
        /// Certificate data
        cert: Vec<u8>,
    },
    /// Service account credentials
    ServiceAccount {
        /// Service ID for the service account
        service_id: String,
        /// API key for the service account
        api_key: String,
    },
    /// Bootstrap credentials
    Bootstrap {
        /// Service ID for bootstrap
        service_id: String,
    },
    /// Test credentials
    Test {
        /// Service ID for testing
        service_id: String,
    },
    /// Custom credentials
    Custom(HashMap<String, String>),
}

/// Authentication result
#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct AuthResult {
    /// Authenticated principal
    #[zeroize(skip)]
    pub principal: Principal,

    /// Authentication token
    pub token: String,

    /// Token expiration time
    #[zeroize(skip)]
    pub expires_at: DateTime<chrono::Utc>,

    /// Granted permissions
    pub permissions: Vec<String>,

    /// Additional metadata
    #[zeroize(skip)]
    pub metadata: HashMap<String, String>,
}

/// Principal (authenticated user/service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principal {
    /// Principal ID
    pub id: String,

    /// Principal name
    pub name: String,

    /// Principal type
    pub principal_type: PrincipalType,

    /// Roles
    pub roles: Vec<String>,

    /// Permissions
    pub permissions: Vec<String>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Type of principal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrincipalType {
    /// Human user
    User,
    /// Service account
    Service,
    /// API client
    Client,
    /// System account
    System,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_password_serde() {
        let creds = Credentials::Password {
            username: "admin".to_string(),
            password: "secret".to_string(),
        };
        let json = serde_json::to_string(&creds).expect("should succeed");
        let deserialized: Credentials = serde_json::from_str(&json).expect("should succeed");
        if let Credentials::Password { username, password } = deserialized {
            assert_eq!(username, "admin");
            assert_eq!(password, "secret");
        } else {
            unreachable!("Expected Password variant");
        }
    }

    #[test]
    fn test_credentials_api_key_serde() {
        let creds = Credentials::ApiKey {
            key: "key123".to_string(),
            service_id: "svc-1".to_string(),
        };
        let json = serde_json::to_string(&creds).expect("should succeed");
        let deserialized: Credentials = serde_json::from_str(&json).expect("should succeed");
        if let Credentials::ApiKey { key, service_id } = deserialized {
            assert_eq!(key, "key123");
            assert_eq!(service_id, "svc-1");
        } else {
            unreachable!("Expected ApiKey variant");
        }
    }

    #[test]
    fn test_credentials_bearer_serde() {
        let creds = Credentials::Bearer {
            token: "tok".to_string(),
        };
        let json = serde_json::to_string(&creds).expect("should succeed");
        let deserialized: Credentials = serde_json::from_str(&json).expect("should succeed");
        if let Credentials::Bearer { token } = deserialized {
            assert_eq!(token, "tok");
        } else {
            unreachable!("Expected Bearer variant");
        }
    }

    #[test]
    fn test_credentials_all_variants_serde() {
        let variants: Vec<Credentials> = vec![
            Credentials::Token {
                token: "jwt".to_string(),
            },
            Credentials::Certificate {
                cert: vec![1, 2, 3],
            },
            Credentials::ServiceAccount {
                service_id: "svc".to_string(),
                api_key: "key".to_string(),
            },
            Credentials::Bootstrap {
                service_id: "boot".to_string(),
            },
            Credentials::Test {
                service_id: "test".to_string(),
            },
            Credentials::Custom({
                let mut m = HashMap::new();
                m.insert("k".to_string(), "v".to_string());
                m
            }),
        ];
        for creds in variants {
            let json = serde_json::to_string(&creds).expect("should succeed");
            let _deserialized: Credentials = serde_json::from_str(&json).expect("should succeed");
        }
    }

    #[test]
    fn test_principal_serde() {
        let principal = Principal {
            id: "p1".to_string(),
            name: "Admin".to_string(),
            principal_type: PrincipalType::User,
            roles: vec!["admin".to_string()],
            permissions: vec!["read".to_string(), "write".to_string()],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&principal).expect("should succeed");
        let deserialized: Principal = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.id, "p1");
        assert_eq!(deserialized.name, "Admin");
        assert_eq!(deserialized.roles.len(), 1);
        assert_eq!(deserialized.permissions.len(), 2);
    }

    #[test]
    fn test_principal_type_serde() {
        for pt in [
            PrincipalType::User,
            PrincipalType::Service,
            PrincipalType::Client,
            PrincipalType::System,
        ] {
            let json = serde_json::to_string(&pt).expect("should succeed");
            let _deserialized: PrincipalType = serde_json::from_str(&json).expect("should succeed");
        }
    }

    #[test]
    fn test_auth_result_serde() {
        let result = AuthResult {
            principal: Principal {
                id: "p1".to_string(),
                name: "User".to_string(),
                principal_type: PrincipalType::User,
                roles: vec![],
                permissions: vec![],
                metadata: HashMap::new(),
            },
            token: "token123".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            permissions: vec!["read".to_string()],
            metadata: HashMap::new(),
        };
        let json = serde_json::to_string(&result).expect("should succeed");
        let deserialized: AuthResult = serde_json::from_str(&json).expect("should succeed");
        assert_eq!(deserialized.token, "token123");
        assert_eq!(deserialized.permissions.len(), 1);
    }
}

#[cfg(test)]
mod proptest_tests {
    use super::*;
    use proptest::prelude::*;

    fn credentials_strategy() -> impl Strategy<Value = Credentials> {
        prop_oneof![
            (any::<String>(), any::<String>()).prop_map(|(u, p)| Credentials::Password {
                username: u,
                password: p,
            }),
            (any::<String>(), any::<String>()).prop_map(|(k, s)| Credentials::ApiKey {
                key: k,
                service_id: s,
            }),
            any::<String>().prop_map(|t| Credentials::Bearer { token: t }),
            any::<String>().prop_map(|t| Credentials::Token { token: t }),
            proptest::collection::vec(any::<u8>(), 0..64)
                .prop_map(|c| Credentials::Certificate { cert: c }),
            (any::<String>(), any::<String>()).prop_map(|(s, k)| Credentials::ServiceAccount {
                service_id: s,
                api_key: k,
            }),
            any::<String>().prop_map(|s| Credentials::Bootstrap { service_id: s }),
            any::<String>().prop_map(|s| Credentials::Test { service_id: s }),
            proptest::collection::hash_map(any::<String>(), any::<String>(), 0..8)
                .prop_map(Credentials::Custom),
        ]
    }

    proptest! {
        #[test]
        fn credentials_round_trip_serde(creds in credentials_strategy()) {
            let json = serde_json::to_string(&creds).expect("should succeed");
            let deserialized: Credentials = serde_json::from_str(&json).expect("should succeed");
            let json2 = serde_json::to_string(&deserialized).expect("should succeed");
            let round_tripped: Credentials = serde_json::from_str(&json2).expect("should succeed");
            prop_assert_eq!(deserialized, round_tripped);
        }
    }
}
