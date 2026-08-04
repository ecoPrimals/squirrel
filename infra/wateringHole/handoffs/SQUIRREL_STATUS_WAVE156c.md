# Squirrel Status — Wave 156c (Aug 4, 2026)

## Summary

**Deprecated surface cleanup**: Reduced deprecated item count from **40 → 13** across the workspace. Focused on removing dead-code deprecated aliases, constants, and builder methods that had zero consumers.

## Changes

### Beardog → SecurityProvider Migration (COMPLETE)

Removed all deprecated items from the Beardog → SecurityProvider rename:

| Removed Item | File |
|-------------|------|
| `BeardogSecurityCoordinator` type alias | `security_coordinator.rs` |
| `BeardogSecurityProvider` type alias | `providers/security_provider.rs` |
| `BeardogIntegration` type alias | `providers/security_provider.rs` |
| `BEARDOG_SECURITY_SERVICE_ID` constant | `providers/security_provider.rs` (inlined as private `LEGACY_SECURITY_SERVICE_ID`) |
| `BearDogClient`, `BearDogClientConfig` aliases | `security_provider_client.rs` |
| `BearDogJwtConfig`, `BearDogJwtService` aliases | `ecosystem_jwt.rs` |
| `beardog_client` module alias | `auth/lib.rs` |
| `beardog_jwt` module alias | `auth/lib.rs` |
| `deprecated_primal_named_auth` module + re-exports | `auth/lib.rs` |
| `beardog_endpoint()` builder method | `config/builder.rs` |
| `beardog_auth()` builder method | `config/builder.rs` |
| `beardog_endpoint_optional()` builder method | `config/builder.rs` |
| `create_beardog_client()` function | `security/mod.rs` |
| `AUTH_METHOD_SERDE_BEARDOG`, `AUTH_METHOD_SERDE_BEARDOG_PASCAL` | `config/types/security.rs` |

Tests migrated to use canonical names. Serde aliases (`"Beardog"`, `"beardog"`) preserved for config file backward compatibility.

### Port Constants Removed (7 dead constants)

| Removed | Replacement |
|---------|-------------|
| `DEFAULT_BIND_ADDRESS` | `get_bind_address()` |
| `DEFAULT_WEBSOCKET_PORT` | `get_service_port("websocket")` |
| `DEFAULT_HTTP_PORT` | `get_service_port("http")` |
| `DEFAULT_ADMIN_PORT` | `get_service_port("admin")` |
| `DEFAULT_METRICS_PORT` | `DEFAULT_METRICS_LISTEN_PORT` (non-deprecated) |
| `DEFAULT_SONGBIRD_PORT` | `DEFAULT_DISCOVERY_PORT` |
| `BIOMEOS_SOCKET_FALLBACK_DIR` | `get_socket_dir()` |

`get_port_from_env()` evolved to `port_from_env()` — the deprecation was incorrect (it's a self-knowledge utility, not a discovery function).

### Other Removals

- `DefaultConfigManager`, `Config` type aliases in `squirrel-mcp-config`
- `HttpMethod::parse_method()` in SDK (test migrated to `FromStr`)
- `discover_services_by_primal_types()` (zero callers, dead code)

## Remaining Deprecated Items (13)

| Item | Scope | Blocker |
|------|-------|---------|
| `PrimalType` enum | 42 files, ~500 refs | Multi-wave migration (entire ecosystem uses it) |
| `EcosystemPrimalType` enum | Same surface | Same |
| `PluginError` enum (SDK) | 600+ refs in SDK crate | SDK-wide error type migration |
| `AIError` enum (ai-tools) | 3 consumer files | Touches `crate::Result` alias |
| `PluginMetadata` struct | 10+ consumer files | Plugin subsystem migration |

## For Upstream Teams

- **bearDog**: All hardcoded `beardog` references in squirrel's TYPE SYSTEM are removed. Environment variable fallbacks (`BEARDOG_ENDPOINT`, `BEARDOG_SOCKET`, `BEARDOG_FAMILY_SEED`) remain as runtime backward compat — those are in fallback chains, not type aliases.
- **Serde**: Config files using `"beardog"` or `"Beardog"` for auth method still deserialize correctly via serde aliases. No config migration needed.

## Known Pre-existing Issue

`squirrel-core::monitoring_service_provider_trait_methods` intermittently fails in workspace-wide runs but passes in isolation. Pre-existing test isolation bug (shared state leaking from parallel crate tests). Not introduced by this wave.

## Metrics

- **Tests**: 4,613 passing / 5 ignored
- **Deprecated items**: 40 → 13
- **Files changed**: 24
- **Zero warnings**, zero test regressions from this wave
