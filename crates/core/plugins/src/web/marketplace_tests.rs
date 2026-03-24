// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 ecoPrimals Contributors

use super::*;
use crate::DefaultPluginManager;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::test]
async fn test_marketplace_client_creation() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);

    let endpoints = client.get_endpoints();
    assert!(!endpoints.is_empty());
    assert!(
        endpoints
            .iter()
            .any(|ep| ep.path == "/api/marketplace/search")
    );
}

#[tokio::test]
async fn test_repository_management() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);

    let repo = PluginRepository {
        id: Uuid::new_v4(),
        name: "Test Repository".to_string(),
        url: "https://example.com/plugins".to_string(),
        repo_type: "community".to_string(),
        enabled: true,
        auth: None,
        metadata: HashMap::new(),
    };

    let response = client.add_repository(repo).await.expect("should succeed");
    assert_eq!(response.status, HttpStatus::Created);
}

#[tokio::test]
async fn test_plugin_search() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);

    let criteria = MarketplaceSearchCriteria {
        query: Some("test".to_string()),
        category: None,
        author: None,
        capabilities: None,
        tags: None,
        min_rating: None,
        verified_only: None,
        sort_by: Some("rating".to_string()),
        sort_order: Some("desc".to_string()),
        page: Some(1),
        per_page: Some(10),
    };

    let response = client
        .search_plugins(criteria)
        .await
        .expect("should succeed");
    assert_eq!(response.status, HttpStatus::Ok);
}

#[tokio::test]
async fn handle_request_search_and_cache_hit() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);

    let criteria = MarketplaceSearchCriteria {
        query: Some("cache-key".to_string()),
        category: None,
        author: None,
        capabilities: None,
        tags: None,
        min_rating: None,
        verified_only: None,
        sort_by: None,
        sort_order: None,
        page: Some(1),
        per_page: Some(5),
    };
    let body = serde_json::to_value(&criteria).expect("should succeed");
    let req = WebRequest {
        method: HttpMethod::Post,
        path: "/api/marketplace/search".to_string(),
        query_params: HashMap::new(),
        headers: HashMap::new(),
        body: Some(body.clone()),
        user_id: None,
        permissions: vec![],
        route_params: HashMap::new(),
    };
    let r1 = client.handle_request(req).await.expect("should succeed");
    assert_eq!(r1.status, HttpStatus::Ok);
    let req2 = WebRequest {
        method: HttpMethod::Post,
        path: "/api/marketplace/search".to_string(),
        query_params: HashMap::new(),
        headers: HashMap::new(),
        body: Some(body),
        user_id: None,
        permissions: vec![],
        route_params: HashMap::new(),
    };
    let r2 = client.handle_request(req2).await.expect("should succeed");
    assert_eq!(r2.status, HttpStatus::Ok);
}

#[tokio::test]
async fn handle_request_not_found() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);
    let req = WebRequest {
        method: HttpMethod::Get,
        path: "/api/marketplace/unknown".to_string(),
        query_params: HashMap::new(),
        headers: HashMap::new(),
        body: None,
        user_id: None,
        permissions: vec![],
        route_params: HashMap::new(),
    };
    let res = client.handle_request(req).await.expect("should succeed");
    assert_eq!(res.status, HttpStatus::NotFound);
}

#[tokio::test]
async fn handle_request_featured_trending_repositories_install() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);

    let f = client
        .handle_request(WebRequest {
            method: HttpMethod::Get,
            path: "/api/marketplace/featured".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(f.status, HttpStatus::Ok);

    let t = client
        .handle_request(WebRequest {
            method: HttpMethod::Get,
            path: "/api/marketplace/trending".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(t.status, HttpStatus::Ok);

    let list = client
        .handle_request(WebRequest {
            method: HttpMethod::Get,
            path: "/api/marketplace/repositories".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(list.status, HttpStatus::Ok);

    let pid = Uuid::new_v4();
    let inst = client
        .handle_request(WebRequest {
            method: HttpMethod::Post,
            path: format!("/api/marketplace/install/{pid}"),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(inst.status, HttpStatus::Accepted);

    let installs = client
        .handle_request(WebRequest {
            method: HttpMethod::Get,
            path: "/api/marketplace/installations".to_string(),
            query_params: HashMap::new(),
            headers: HashMap::new(),
            body: None,
            user_id: None,
            permissions: vec![],
            route_params: HashMap::new(),
        })
        .await
        .expect("should succeed");
    assert_eq!(installs.status, HttpStatus::Ok);
}

#[tokio::test]
async fn extract_uuid_and_cancel_paths() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);
    let id = Uuid::new_v4();
    let p = format!("/api/marketplace/repositories/{id}");
    assert_eq!(
        client.extract_uuid_from_path(&p).expect("should succeed"),
        id
    );

    let cancel_path = format!("/api/marketplace/installations/{id}/cancel");
    assert_eq!(
        client
            .extract_uuid_from_path(&cancel_path)
            .expect("should succeed"),
        id
    );
    assert!(client.extract_uuid_from_path("/bad").is_err());
}

#[tokio::test]
async fn process_search_sorts_and_pagination() {
    let manager = Arc::new(DefaultPluginManager::new());
    let client = PluginMarketplaceClient::new(manager);
    let mut plugins = client.get_sample_plugins("x");
    plugins.push(MarketplacePlugin {
        id: Uuid::new_v4(),
        name: "alpha".to_string(),
        version: "1.0.0".to_string(),
        description: "d".to_string(),
        author: "a".to_string(),
        category: "cat".to_string(),
        tags: vec![],
        capabilities: vec![],
        dependencies: vec![],
        download_url: "u".to_string(),
        documentation_url: None,
        repository_url: None,
        rating: Some(1.0),
        downloads: 10,
        verified: false,
        size: 1,
        published_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        requirements: SystemRequirements {
            min_version: "1".to_string(),
            features: vec![],
            memory_mb: None,
            disk_space_mb: None,
        },
        screenshots: vec![],
        changelog: None,
    });

    let crit = MarketplaceSearchCriteria {
        query: None,
        category: None,
        author: None,
        capabilities: None,
        tags: None,
        min_rating: None,
        verified_only: None,
        sort_by: Some("name".to_string()),
        sort_order: Some("asc".to_string()),
        page: Some(1),
        per_page: Some(1),
    };
    let out = client
        .process_search_results(plugins, &crit)
        .expect("should succeed");
    assert_eq!(out.total, 2);
    assert_eq!(out.plugins.len(), 1);
    assert_eq!(out.total_pages, 2);
}

#[test]
fn marketplace_types_serde_roundtrip() {
    let repo = PluginRepository {
        id: Uuid::nil(),
        name: "n".to_string(),
        url: "https://x".to_string(),
        repo_type: "official".to_string(),
        enabled: true,
        auth: Some(RepositoryAuth {
            auth_type: "token".to_string(),
            credentials: std::iter::once(("t".to_string(), "v".to_string())).collect(),
        }),
        metadata: HashMap::new(),
    };
    let j = serde_json::to_string(&repo).expect("should succeed");
    let r2: PluginRepository = serde_json::from_str(&j).expect("should succeed");
    assert_eq!(r2.name, repo.name);

    let st = InstallationStatusType::Verifying;
    let s = serde_json::to_string(&st).expect("should succeed");
    let st2: InstallationStatusType = serde_json::from_str(&s).expect("should succeed");
    assert!(matches!(st2, InstallationStatusType::Verifying));
}
