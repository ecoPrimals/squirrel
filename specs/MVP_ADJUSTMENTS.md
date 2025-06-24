---
description: MVP Adjustment Tracking - Work items for transitioning towards production MVP
version: 1.0.0
date: 2025-01-15
owner: All Teams
status: In Progress
related_docs:
  - SPECS.md
  - SPRINT_PLAN_Q1_2025.md
---

# MVP Adjustments Tracking

## Overview

This document tracks the critical work items needed to transition from our current development state towards the production-ready MVP defined in `SPECS.md`. These items represent key adjustments, additions, and refinements required for a complete, deployable system.

## Status Legend

- 🔴 **Not Started** - Work not yet begun
- 🟡 **In Progress** - Currently being worked on
- 🟢 **Complete** - Work completed and tested
- 🔵 **Blocked** - Waiting on dependencies

---

## Core Protocol & Cost Management

### 1. Protocol Cost Header Enhancement
**Status:** 🔴 Not Started  
**Path:** `protocol/cost_header.rs` + spec documentation  
**Priority:** High  

#### Scope & Acceptance Criteria
- Add `rate_per_1k_tokens` field to message envelope
- Add `estimated_tokens` field to message envelope  
- Unit tests for serialization/deserialization
- Default value `0` maintains current behavior compatibility
- Documentation update for protocol specification

#### Dependencies
- Protocol message structure finalization
- Backward compatibility verification

---

### 2. Usage Ledger System
**Status:** 🔴 Not Started  
**Path:** `crates/ledger/`  
**Priority:** High  

#### Scope & Acceptance Criteria
- **Core Structs:** `UsageRecord`, `Invoice`, `LedgerWriter`
- **Recording Fields:** node ID, timestamp, capability tag, token count, cost, latency
- **Output Format:** JSON lines with daily rotation
- **Storage:** Efficient append-only logging system
- **Querying:** Basic usage analytics and reporting

#### Dependencies
- Cost header implementation (#1)
- Node identification system

---

### 3. Cost-Meter Middleware
**Status:** 🔴 Not Started  
**Path:** Nestgate orchestrator middleware  
**Priority:** High  

#### Scope & Acceptance Criteria
- Calculate cost on response completion
- Write to usage ledger (#2)
- Enforce `daily_spend_limit` from TOML configuration
- Fail-closed behavior with `429 PAYMENT_REQUIRED` status
- Real-time spend tracking and alerts

#### Dependencies
- Usage ledger system (#2)
- Configuration schema updates (#10)

---

## External Integration & Plugins

### 4. External Model Proxy Plugin
**Status:** 🔴 Not Started  
**Path:** `plugins/ext_model_proxy/`  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- Accept parameters: model ID, provider, prompt, max_tokens
- Stream completion responses
- Hash prompt and result for auditing
- Respect `max_cost` guard limits
- Log policy-hash drift detection
- Support major providers (OpenAI, Anthropic, etc.)

#### Dependencies
- Cost-meter middleware (#3)
- Plugin system architecture
- Provider API integrations

---

## Node Operations & Federation

### 5. Node Starter Kit Template
**Status:** 🔴 Not Started  
**Path:** `squirrel-node-template` repository  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **Docker Setup:** Complete `docker-compose.yml`
- **Configuration:** Sample `.env` with all required variables
- **Corpus:** Minimal corpus YAML for testing
- **Documentation:** Comprehensive "Quick-start" README
- **CLI Integration:** `squirrel handshake --init` generates keypair + metadata

#### Dependencies
- CLI tooling completion
- Docker containerization
- Configuration schema finalization

---

### 6. Federation Join Documentation
**Status:** 🔴 Not Started  
**Path:** `docs/federation/join.md`  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **Step-by-step Process:** keygen → metadata → cost table → policy tests → invite token
- **Reference Links:** Starter kit and CLI flag documentation
- **Troubleshooting:** Common issues and solutions
- **Requirements:** Hardware, network, and security prerequisites

#### Dependencies
- Node starter kit (#5)
- Policy test suite (#7)
- CLI documentation

---

### 7. Policy Test Suite
**Status:** 🔴 Not Started  
**Path:** `crates/policy_verifier`  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **CLI Tool:** `squirrel-policy verify <path/to/policy.toml>`
- **Verification Checks:**
  - Lineage logging enabled
  - RBAC roles properly configured
  - Forbidden data-classes absent
  - Cost controls in place
- **Reporting:** Clear pass/fail with detailed explanations

#### Dependencies
- Policy schema standardization
- RBAC system implementation

---

## Future Architecture & Standards

### 8. QUIC Envelope Placeholder
**Status:** 🔴 Not Started  
**Path:** `protocol/quic_envelope/README.md`  
**Priority:** Low  

#### Scope & Acceptance Criteria
- **Documentation:** One-page overview with TODO items
- **Field Mapping:** Planned migration from gRPC to QUIC
- **Migration Plan:** Timeline and compatibility strategy
- **User Communication:** Clear messaging for gRPC users

#### Dependencies
- gRPC deprecation timeline
- QUIC protocol research

---

## Security & Cryptography

### 9. Crypto Helpers Library
**Status:** 🔴 Not Started  
**Path:** `crates/crypto_helpers`  
**Priority:** High  

#### Scope & Acceptance Criteria
- **Dependencies:** Integration with `ed25519-dalek`
- **Core Functions:** `sign(msg)`, `verify(sig, msg)`
- **Use Cases:** Handshake authentication, invoice signing
- **Security:** Constant-time operations, secure key handling
- **Testing:** Comprehensive cryptographic test suite

#### Dependencies
- Security requirements finalization
- Key management strategy

---

## Configuration & Deprecation

### 10. Config & Spec Cleanup
**Status:** 🔴 Not Started  
**Path:** Configuration files and specifications  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **Default Values:** Add `rate=0` default and `daily_spend_limit` to config schema
- **Deprecation Marking:** Mark old whitelist/quota sections as "deprecated"
- **Transport Notice:** Add gRPC formal deprecation note with QUIC work-in-progress status
- **Schema Validation:** Ensure backward compatibility during transition

#### Dependencies
- Cost management implementation (#1-3)
- QUIC roadmap (#8)

---

## Monitoring & User Experience

### 11. Dashboard Spend Widget
**Status:** 🔴 Not Started  
**Path:** Web dashboard components  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **API Endpoint:** `/metrics/spend_today` with current usage data
- **UI Components:** Spend tracking card showing dollars and tokens
- **Alerts:** Warning at 80% of daily spend cap
- **Real-time Updates:** Live spending updates
- **Historical View:** Basic spend history and trends

#### Dependencies
- Usage ledger system (#2)
- Dashboard infrastructure
- Authentication integration

---

### 12. Documentation Updates
**Status:** 🔴 Not Started  
**Path:** Various documentation files  
**Priority:** Medium  

#### Scope & Acceptance Criteria
- **External Models Policy:** `external_models_policy.md` with T0-T3 trust tiers
- **Cost Meter Design:** `specs/core/cost_meter.md` with design rationale
- **Corpus Charter:** Amendment to reference cost ledger integration
- **API Documentation:** Updated with cost and usage endpoints

#### Dependencies
- Cost management system completion (#1-3)
- External model proxy (#4)
- Policy framework (#7)

---

## Implementation Timeline

### Phase 1: Foundation (Weeks 1-4)
1. Protocol Cost Header Enhancement (#1)
2. Usage Ledger System (#2)
3. Crypto Helpers Library (#9)

### Phase 2: Core Features (Weeks 5-8)
1. Cost-Meter Middleware (#3)
2. Config & Spec Cleanup (#10)
3. Policy Test Suite (#7)

### Phase 3: Integration (Weeks 9-12)
1. External Model Proxy Plugin (#4)
2. Dashboard Spend Widget (#11)
3. Documentation Updates (#12)

### Phase 4: Deployment (Weeks 13-16)
1. Node Starter Kit Template (#5)
2. Federation Join Documentation (#6)
3. QUIC Envelope Placeholder (#8)

---

## Success Metrics

- [ ] All cost management features functional and tested
- [ ] Complete federation onboarding process
- [ ] Production-ready node deployment kit
- [ ] Comprehensive documentation and policies
- [ ] Security audit completion for crypto components
- [ ] Performance benchmarks meet production requirements

---

## Notes

- This document should be updated weekly with progress status
- Each work item requires dedicated testing and documentation
- Security review required for items #9 (crypto) and #4 (external integrations)
- User acceptance testing needed for dashboard and federation components

---

**Last Updated:** 2025-01-15  
**Next Review:** 2025-01-22 