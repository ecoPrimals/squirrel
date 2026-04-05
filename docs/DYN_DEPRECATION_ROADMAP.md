<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# `dyn` deprecation intent

**Goal:** Prefer static dispatch over heap indirection as the codebase and Rust ecosystem mature. Treat `#[async_trait]` (object-safe async traits) as a stepping stone: migrate call sites toward **generics**, **closed enum dispatch**, and **RPITIT** / native `async fn` in traits where MSRV and compiler support allow—reducing `dyn` usage, allocator traffic, and vtable churn on hot paths.

This document inventories remaining `#[async_trait]` **trait definitions** under `crates/` (from `rg '#\[async_trait\]|#\[async_trait::async_trait\]' --type rust -l crates/`) and records approximate `dyn <Trait>` hit counts for the heaviest symbols. Impl-only files (tests, adapters) are not listed as separate rows.

---

## `async_trait` trait inventory

| Trait | dyn count (approx.) | Location | Recommended strategy | Priority |
| --- | ---: | --- | --- | --- |
| `Plugin` | 65 | `crates/core/interfaces/src/plugins.rs`, `crates/core/plugins/src/plugin.rs`, `crates/tools/cli/src/plugins/plugin.rs` | Enum dispatch for built-ins + typed registry for extensions; converge on one `Plugin` surface | **High** |
| `Command` | 73 | `crates/adapter-pattern-examples/src/lib.rs` (pattern); `services/commands` uses `dyn Command` heavily | Closed command enum + `match` dispatch; keep thin trait only at boundaries if needed | **High** |
| `AIClient` | 53 | `crates/tools/ai-tools/src/common/client.rs` | Vendor/backend enum wrapping concrete clients | **High** |
| `AiProviderAdapter` | 23 | `crates/main/src/api/ai/adapters/mod.rs` | Vendor enum dispatch (OpenAI, Anthropic, universal, …) | **High** |
| `UniversalServiceRegistry` | 26 | `crates/main/src/universal_adapters/registry.rs` | Generic registry `Registry<D: Default>` or concrete default type parameter | **Medium** |
| `ServiceDiscovery` | 17 | `crates/core/core/src/service_discovery/trait_.rs` | Generics on client/registry; trait objects only at FFI | **Medium** |
| `ServiceDiscovery` (config) | 2 | `crates/universal-patterns/src/config/port_resolver.rs` | Merge or align with core trait; same generic strategy | **Medium** |
| `ContextPlugin` | 14 | `crates/core/interfaces/src/context.rs`, `crates/core/context/src/rules/mod.rs`, `crates/core/plugins/src/context/mod.rs` | Generics + fewer `Arc<dyn ContextTransformation>`; unify duplicate trait surfaces | **Medium** |
| `ContextTransformation` | 1 | `crates/core/interfaces/src/context.rs` | RPITIT / associated types or enum of transforms | **Medium** |
| `ContextAdapterPlugin` | 2 | `crates/core/interfaces/src/context.rs`, `crates/core/plugins/src/context_adapter/mod.rs` | Generics; remove duplicate local trait if possible | **Medium** |
| `ConditionEvaluator` | 4 | `crates/tools/rule-system/src/evaluator.rs` (also `crates/core/context/src/rules/plugin.rs` — same name) | Enum of evaluators or generic `E: ConditionEvaluator` | **Low** |
| `ActionPlugin` | 2 | `crates/tools/rule-system/src/actions.rs` | Enum dispatch for known actions | **Low** |
| `ActionExecutor` | 2 | `crates/core/context/src/rules/plugin.rs` | Enum dispatch or generic executor list | **Low** |
| `WebPlugin` | 10 | `crates/core/plugins/src/web/mod.rs` | Enum of web plugin kinds or generic `W: WebPlugin` | **Medium** |
| `PrimalProvider` | 7 | `crates/universal-patterns/src/traits/provider.rs` | Generic provider param or enum | **Low** |
| `UniversalPrimalProvider` | 1 | `crates/ecosystem-api/src/traits/primal.rs` | Align with `PrimalProvider`; enum or generics | **Low** |
| `MonitoringProvider` | 7 | `crates/core/core/src/monitoring/types.rs` | Narrow trait + concrete IPC provider; generics in tests | **Low** |
| `ZeroCopyPlugin` | 6 | `crates/core/plugins/src/zero_copy.rs` | Keep static where possible; enum for known plugins | **Medium** |
| `RepositoryProvider` | 2 | `crates/core/plugins/src/plugins/marketplace.rs` | Enum of repo backends | **Low** |
| `CommandAdapter` | 4+ | `crates/adapter-pattern-examples/src/lib.rs`, `crates/adapter-pattern-tests/src/commands.rs` | Enum of adapters or generic `A: CommandAdapter` | **Medium** |

**Note:** Several files under `crates/core/context/src/visualization/` use `#[async_trait]` on `impl Visualizable` only; if `Visualizable` is wired in, prefer native async in trait + RPITIT before adding new `dyn` bounds.

---

## High-impact traits — concrete migration paths

### `Plugin` (~65 `dyn Plugin`)

- Introduce a **typed plugin registry** (`PluginId` → enum of first-party plugins + extension slot).
- Keep a single canonical `Plugin` definition (`squirrel_interfaces`); deprecate parallel trait copies once call sites move to enums.
- Long-term: async methods via stable `async fn` in trait + avoid boxing where possible.

### `Command` (~73 `dyn Command`)

- Replace open registry with a **`Command` enum** (or `&'static str` → enum map) and a single `async fn run(&self, cmd: Command, args: Args)`.
- Reserve `dyn Command` only for dynamic/script plugins if still required.

### `AIClient` (~53 `dyn AIClient`)

- **`AiVendor` enum** (`Mock`, `IpcRouted`, `OpenAICompat`, …) with `match` on `chat` / `get_capabilities`.
- Collapse router `Arc<dyn AIClient>` fields to `enum AiClient { ... }`.

### `ContextPlugin` (~14 `dyn ContextPlugin`)

- Parameterize managers as `P: ContextPlugin` with a **default concrete** type in non-test builds.
- Reduce `Arc<dyn ContextTransformation>` by an enum of transform implementations.

### `AiProviderAdapter` (~23 `dyn AiProviderAdapter`)

- **Vendor enum** mirroring adapter modules (`OpenAi`, `Anthropic`, `Universal`, `Bridge`, …).
- Router holds `Vec<Adapter>` or fixed-size array of variants instead of `Vec<Arc<dyn _>>`.

### `UniversalServiceRegistry` (~26 `dyn UniversalServiceRegistry`)

- **`struct UniversalServiceRegistryImpl`** as default type; generic `T: UniversalServiceRegistry` only where tests need mocks.
- Avoid `Arc<dyn UniversalServiceRegistry>` in adapters; use `Arc<InMemoryServiceRegistry>` or an enum.

### `ServiceDiscovery` (~17 `dyn ServiceDiscovery`)

- **`S: ServiceDiscovery`** on `PortResolver` / clients; default to `MemoryServiceDiscovery` (or one concrete type).
- Deduplicate `universal-patterns` vs `core` trait definitions when feasible.

---

## Maintenance

- Regenerate trait rows when adding `#[async_trait]` definitions:  
  `rg '#\[async_trait\]|#\[async_trait::async_trait\]' --type rust -l crates/`
- Refresh approximate dyn counts:  
  `rg 'dyn <TraitName>\\b' --type rust crates/ | wc -l`
