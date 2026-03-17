// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 ecoPrimals Contributors

#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Property-based round-trip tests for serialization invariants.
//!
//! Follows the wetSpring / groundSpring proptest pattern:
//! for every `Serialize + Deserialize` type, verify that
//! `decode(encode(x)) == x` holds for all generated inputs.

use proptest::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

fn roundtrip_json<T>(value: &T)
where
    T: Serialize + for<'de> Deserialize<'de> + std::fmt::Debug + PartialEq,
{
    let json = serde_json::to_string(value).expect("test: serialization must not fail");
    let decoded: T = serde_json::from_str(&json).expect("test: deserialization must not fail");
    assert_eq!(*value, decoded, "round-trip mismatch");
}

// ── JSON-RPC types ──────────────────────────────────────────────────────

use squirrel::rpc::types::{
    AnnounceCapabilitiesRequest, AnnounceCapabilitiesResponse, HealthCheckRequest,
    HealthCheckResponse, ListProvidersRequest, ListProvidersResponse, ProviderInfo, QueryAiRequest,
    QueryAiResponse, ToolListEntry, ToolListResponse, ToolSource,
};

prop_compose! {
    fn arb_query_ai_request()(
        prompt in "[a-zA-Z0-9 ]{1,100}",
        provider in proptest::option::of("[a-z]{3,10}"),
        model in proptest::option::of("[a-z0-9-]{3,20}"),
        priority in proptest::option::of(0u8..=100),
        max_tokens in proptest::option::of(1usize..10000),
        // Use tenths to avoid JSON float precision loss (0.0, 0.1, ..., 2.0)
        temp_tenths in proptest::option::of(0u8..=20),
        stream in proptest::option::of(proptest::bool::ANY),
    ) -> QueryAiRequest {
        QueryAiRequest {
            prompt,
            provider,
            model,
            priority,
            max_tokens,
            temperature: temp_tenths.map(|t| f32::from(t) / 10.0),
            stream,
        }
    }
}

prop_compose! {
    fn arb_query_ai_response()(
        response in "[a-zA-Z0-9 ]{0,200}",
        provider in "[a-z]{3,10}",
        model in "[a-z0-9-]{3,20}",
        tokens_used in proptest::option::of(0usize..100_000),
        latency_ms in 0u64..60_000,
        success in proptest::bool::ANY,
    ) -> QueryAiResponse {
        QueryAiResponse {
            response,
            provider,
            model,
            tokens_used,
            latency_ms,
            success,
        }
    }
}

prop_compose! {
    fn arb_list_providers_request()(
        capability in proptest::option::of("[a-z.]{3,30}"),
        include_offline in proptest::option::of(proptest::bool::ANY),
    ) -> ListProvidersRequest {
        ListProvidersRequest {
            capability,
            include_offline,
        }
    }
}

prop_compose! {
    fn arb_provider_info()(
        id in "[a-z0-9-]{3,20}",
        name in "[A-Za-z ]{3,30}",
        models in proptest::collection::vec("[a-z0-9-]{3,15}", 0..5),
        capabilities in proptest::collection::vec("[a-z.]{3,20}", 0..5),
        online in proptest::bool::ANY,
        avg_latency_ms in proptest::option::of(0u64..10_000),
        cost_tier in prop_oneof!["free", "basic", "premium", "enterprise"],
    ) -> ProviderInfo {
        ProviderInfo {
            id,
            name,
            models,
            capabilities,
            online,
            avg_latency_ms,
            cost_tier,
        }
    }
}

prop_compose! {
    fn arb_list_providers_response()(
        providers in proptest::collection::vec(arb_provider_info(), 0..5),
    ) -> ListProvidersResponse {
        let total = providers.len();
        ListProvidersResponse { providers, total }
    }
}

prop_compose! {
    fn arb_announce_capabilities_request()(
        capabilities in proptest::collection::vec("[a-z.]{3,30}", 1..10),
        primal in proptest::option::of("[a-zA-Z]{3,20}"),
        socket_path in proptest::option::of("/tmp/[a-z]{3,20}.sock"),
        tools in proptest::option::of(proptest::collection::vec("[a-z.]{3,30}", 0..5)),
        sub_federations in proptest::option::of(proptest::collection::vec("[a-z-]{3,15}", 0..3)),
        genetic_families in proptest::option::of(proptest::collection::vec("[a-z]{3,15}", 0..3)),
    ) -> AnnounceCapabilitiesRequest {
        AnnounceCapabilitiesRequest {
            capabilities,
            primal,
            socket_path,
            tools,
            sub_federations,
            genetic_families,
        }
    }
}

prop_compose! {
    fn arb_announce_capabilities_response()(
        success in proptest::bool::ANY,
        message in "[a-zA-Z0-9 ]{5,50}",
        announced_at in "[0-9T:Z-]{20,30}",
        tools_registered in 0usize..100,
    ) -> AnnounceCapabilitiesResponse {
        AnnounceCapabilitiesResponse {
            success,
            message,
            announced_at,
            tools_registered,
        }
    }
}

prop_compose! {
    fn arb_health_check_response()(
        status in prop_oneof!["healthy", "degraded", "unhealthy"],
        version in "[0-9]+\\.[0-9]+\\.[0-9]+",
        uptime_seconds in 0u64..1_000_000,
        active_providers in 0usize..20,
        requests_processed in 0u64..1_000_000,
        // Use integer-backed f64 to avoid JSON float precision loss
        avg_ms_int in proptest::option::of(0u32..10_000),
    ) -> HealthCheckResponse {
        HealthCheckResponse {
            status,
            version,
            uptime_seconds,
            active_providers,
            requests_processed,
            avg_response_time_ms: avg_ms_int.map(f64::from),
        }
    }
}

fn arb_tool_source() -> impl Strategy<Value = ToolSource> {
    prop_oneof![
        Just(ToolSource::Builtin),
        "[a-zA-Z]{3,20}".prop_map(|primal| ToolSource::Remote { primal }),
    ]
}

prop_compose! {
    fn arb_tool_list_entry()(
        name in "[a-z.]{3,30}",
        description in "[a-zA-Z0-9 ]{5,80}",
        domain in "[a-z]{3,15}",
        source in arb_tool_source(),
    ) -> ToolListEntry {
        ToolListEntry {
            name,
            description,
            domain,
            source,
            input_schema: None,
        }
    }
}

prop_compose! {
    fn arb_tool_list_response()(
        tools in proptest::collection::vec(arb_tool_list_entry(), 0..10),
    ) -> ToolListResponse {
        let total = tools.len();
        ToolListResponse { tools, total }
    }
}

// ── Proptest declarations ───────────────────────────────────────────────

proptest! {
    #[test]
    fn query_ai_request_roundtrip(req in arb_query_ai_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn query_ai_response_roundtrip(resp in arb_query_ai_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn list_providers_request_roundtrip(req in arb_list_providers_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn list_providers_response_roundtrip(resp in arb_list_providers_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn announce_capabilities_request_roundtrip(req in arb_announce_capabilities_request()) {
        roundtrip_json(&req);
    }

    #[test]
    fn announce_capabilities_response_roundtrip(resp in arb_announce_capabilities_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn health_check_response_roundtrip(resp in arb_health_check_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn tool_list_response_roundtrip(resp in arb_tool_list_response()) {
        roundtrip_json(&resp);
    }

    #[test]
    fn health_check_request_roundtrip(_req in Just(HealthCheckRequest {})) {
        roundtrip_json(&HealthCheckRequest {});
    }
}

// ── Niche invariants ────────────────────────────────────────────────────

#[test]
fn niche_json_roundtrip() {
    let cost_json = squirrel::niche::cost_estimates_json();
    let serialized =
        serde_json::to_string(&cost_json).expect("test: niche cost JSON must serialize");
    let decoded: serde_json::Value =
        serde_json::from_str(&serialized).expect("test: niche cost JSON must deserialize");
    assert_eq!(cost_json, decoded);

    let deps_json = squirrel::niche::operation_dependencies();
    let serialized =
        serde_json::to_string(&deps_json).expect("test: niche deps JSON must serialize");
    let decoded: serde_json::Value =
        serde_json::from_str(&serialized).expect("test: niche deps JSON must deserialize");
    assert_eq!(deps_json, decoded);
}
