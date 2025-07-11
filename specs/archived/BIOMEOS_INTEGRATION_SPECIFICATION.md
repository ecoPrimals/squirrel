# biomeOS Integration Specification

**Status:** Master Integration Document | **Date:** January 2025 | **Version:** 1.0.0

---

## 🎯 **Executive Summary**

This document provides the master specification for integrating the five Primals into biomeOS. Each team should use this as their integration roadmap. The ecosystem is **91.6% ready** - this document defines the remaining work to achieve full biomeOS integration.

**Timeline:** 10-11 weeks to full biomeOS  
**Risk Level:** Low (proven patterns, working integrations exist)  
**Team Coordination:** Required across all Primals

---

## 📋 **Integration Architecture Overview**

### **biomeOS Flow**
```
biome.yaml (Genome) → Toadstool (Orchestrator) → Service Mesh (Songbird) → Domain Services
                                    ↓
                         Security Layer (BearDog) + Storage (NestGate) + AI (Squirrel)
```

### **Communication Pattern: "Songbird Pattern"**
- **Single HTTPS endpoint** for entire biomeOS
- **Internal service mesh** for Primal-to-Primal communication  
- **Unified API gateway** for external access
- **Service discovery** through Songbird ecosystem

---

## 🍄 **Toadstool Team - Universal Runtime Integration**

### **Priority 1: Manifest Schema Alignment** 🔴 **CRITICAL**
**Timeline:** Week 1-2 | **Effort:** High | **Risk:** Low

#### **Current State**
- ✅ Sophisticated `BiomeManifest` structure exists (90% compatible)
- ✅ Multi-runtime support (Container, WASM, Native, GPU) complete
- ✅ BearDog-first startup sequence implemented

#### **Required Work**
1. **Align BiomeManifest with biomeOS API inoculum**
   ```rust
   // Extend existing structure
   pub struct BiomeManifest {
       pub metadata: BiomeMetadata,
       pub primals: HashMap<String, PrimalConfig>,
       pub services: HashMap<String, ServiceConfig>,
       pub resources: BiomeResources,
       pub security: BiomeSecurity,
       pub networking: BiomeNetworking,
       pub storage: BiomeStorage,
       // ADD: biomeOS-specific sections
       pub specialization: BiomeSpecialization,
       pub templates: BiomeTemplates,
   }
   ```

2. **Implement volume provisioning integration with NestGate**
   ```yaml
   # Support parsing this syntax in biome.yaml
   storage:
     volumes:
       - name: "data-volume"
         size: "100Gi"
         tier: "hot"
         provisioner: "nestgate"
   ```

3. **Add Squirrel MCP agent runtime support**
   ```yaml
   # Support parsing this syntax in biome.yaml
   agents:
     - name: "data-analyst"
       runtime: "wasm"
       capabilities: ["data_analysis", "visualization"]
       executor: "squirrel"
   ```

#### **Integration Points**
- **With NestGate:** Volume mounting API implementation
- **With Squirrel:** Agent execution API implementation  
- **With Songbird:** Service registration for orchestrator
- **With BearDog:** Security context validation

#### **Success Criteria**
- [ ] Single `biome.yaml` can orchestrate all 5 Primals
- [ ] Volume provisioning from manifest works end-to-end
- [ ] Agent deployment from manifest functional
- [ ] BearDog-first startup sequence working

---

## 🎼 **Songbird Team - Service Mesh Integration**

### **Priority 1: biomeOS Service Registration Standards** 🟡 **HIGH**
**Timeline:** Week 1-2 | **Effort:** Medium | **Risk:** Low

#### **Current State**  
- ✅ Complete orchestrator with 82/82 tests passing
- ✅ Multi-protocol communication operational
- ✅ Federation support for multi-biome networks
- ✅ Already integrates with NestGate and Squirrel

#### **Required Work**
1. **Define biomeOS-specific service registration patterns**
   ```json
   // Standard service registration format for all Primals
   {
     "service_id": "primal-{type}-{instance}",
     "primal_type": "toadstool|songbird|nestgate|beardog|squirrel",
     "biome_id": "biome-uuid",
     "capabilities": {
       "core": ["capability1", "capability2"],
       "extended": ["feature1", "feature2"],
       "integrations": ["primal1_integration", "primal2_integration"]
     },
     "api_endpoints": { /* standardized endpoints */ },
     "health_checks": { /* health monitoring config */ }
   }
   ```

2. **Create biomeOS dashboard endpoints**
   ```
   GET  /biome/status           - Overall biome health
   GET  /biome/primals          - All Primal statuses  
   GET  /biome/services         - Service mesh overview
   GET  /biome/metrics          - Biome-wide metrics
   POST /biome/commands         - Biome-level operations
   ```

3. **Implement biome.yaml service discovery integration**
   - Parse service definitions from biome.yaml
   - Register services according to biomeOS patterns
   - Enable cross-Primal service discovery

#### **Integration Points**
- **With All Primals:** Standardized service registration
- **With Toadstool:** Orchestrator coordination
- **With BearDog:** Authentication provider integration

#### **Success Criteria**
- [ ] All Primals register using standard format
- [ ] biomeOS dashboard shows unified view
- [ ] Service discovery works across all Primals
- [ ] Federation ready for multi-biome networks

---

## 🏰 **NestGate Team - Storage Integration**

### **Priority 1: Automated Provisioning from Manifest** 🟡 **HIGH**
**Timeline:** Week 2-3 | **Effort:** Medium | **Risk:** Medium

#### **Current State**
- ✅ Complete ZFS management with 89% test success
- ✅ Volume provisioning APIs with MCP integration  
- ✅ Songbird integration operational
- ✅ Multi-protocol access (NFS, SMB, iSCSI, S3)

#### **Required Work**
1. **Parse biome.yaml volume definitions**
   ```rust
   // Add to existing volume provisioning
   pub async fn provision_from_manifest(
       &self,
       volume_spec: &VolumeSpec,
       biome_context: &BiomeContext,
   ) -> Result<VolumeInfo> {
       // Parse manifest → create ZFS dataset → mount → register
   }
   ```

2. **Implement automated provisioning workflow**
   ```
   biome.yaml → Volume Request → ZFS Creation → Mount Point → Service Registration
   ```

3. **Add Primal-specific storage templates**
   ```yaml
   # Pre-defined storage patterns for each Primal
   templates:
     toadstool_runtime: 
       - scratch_space: "10Gi"
       - results_storage: "100Gi"
     squirrel_agents:
       - model_cache: "50Gi" 
       - training_data: "500Gi"
   ```

4. **Integrate BearDog security policies**
   - Encryption-at-rest for sensitive datasets
   - Access control integration
   - Audit trail for storage operations

#### **Integration Points**
- **With Toadstool:** Volume mounting for workloads
- **With Squirrel:** Agent data storage
- **With BearDog:** Storage encryption and access control
- **With Songbird:** Storage service registration

#### **Success Criteria**
- [ ] Volumes provision automatically from biome.yaml
- [ ] All Primals can request and use storage
- [ ] Encryption works with BearDog policies
- [ ] Performance monitoring operational

---

## 🐕 **BearDog Team - Security Integration**

### **Priority 1: Cross-Primal Authentication System** 🟡 **HIGH**
**Timeline:** Week 3-4 | **Effort:** High | **Risk:** Medium

#### **Current State**
- ✅ Enterprise security framework operational
- ✅ Songbird SecurityProvider implemented
- ✅ Service-to-service authentication working
- ✅ Comprehensive audit framework

#### **Required Work**
1. **Implement biomeOS security context definitions**
   ```rust
   pub struct BiomeSecurityContext {
       pub biome_id: String,
       pub security_level: SecurityLevel,
       pub encryption_policies: EncryptionPolicies,
       pub access_controls: AccessControls,
       pub audit_requirements: AuditRequirements,
   }
   ```

2. **Create cross-Primal token propagation system**
   ```
   BearDog → Token Generation → All Primals → Token Validation → Secure Operations
   ```

3. **Define Primal-specific key scoping**
   - Toadstool: Execution environment encryption
   - Songbird: Service mesh TLS certificates  
   - NestGate: Storage encryption keys
   - Squirrel: AI model and data encryption

4. **Implement biome-wide threat correlation**
   - Cross-Primal security event aggregation
   - Threat pattern recognition across services
   - Coordinated incident response

#### **Integration Points**
- **With All Primals:** Authentication token validation
- **With Songbird:** Security provider integration
- **With NestGate:** Storage encryption policies
- **With Squirrel:** AI model security

#### **Success Criteria**
- [ ] Single sign-on across all Primals
- [ ] Unified security policy enforcement
- [ ] Cross-Primal audit trail working
- [ ] Threat detection operational

---

## 🐿️ **Squirrel Team - MCP Platform Integration**

### **Priority 1: Agent Deployment from Manifest** 🟢 **MEDIUM**
**Timeline:** Week 2-3 | **Effort:** Medium | **Risk:** Low

#### **Current State**
- ✅ Complete MCP protocol with multi-transport
- ✅ Plugin platform with cross-platform sandboxing
- ✅ Ecosystem integration (Songbird + Toadstool delegation)
- ✅ AI coordination with multi-provider routing

#### **Required Work**
1. **Define biome.yaml agent deployment patterns**
   ```yaml
   # Standard agent deployment syntax
   agents:
     - name: "data-analyst"
       capabilities: ["data_analysis", "visualization", "reporting"]
       ai_provider: "openai"
       model: "gpt-4"
       execution_environment: "wasm"
       resource_limits:
         memory_mb: 256
         cpu_percent: 10
         timeout_seconds: 300
   ```

2. **Implement biomeOS service discovery integration**
   - Register MCP services with Songbird
   - Enable agent discovery across biome
   - Support cross-Primal agent communication

3. **Add BearDog security provider integration**
   - Agent authentication and authorization
   - Secure AI model access
   - Audit trail for agent operations

4. **Create agent lifecycle management for biomes**
   - Deploy agents from manifest
   - Monitor agent health and performance
   - Scale agents based on demand

#### **Integration Points**
- **With Toadstool:** Plugin execution delegation (already working)
- **With Songbird:** Service registration and discovery
- **With BearDog:** Agent security and authentication
- **With NestGate:** Agent data storage access

#### **Success Criteria**
- [ ] Agents deploy automatically from biome.yaml
- [ ] MCP integration across all Primals
- [ ] Agent security through BearDog
- [ ] Plugin execution via Toadstool working

---

## 🔗 **Cross-Team Integration Requirements**

### **Phase 1: Foundation (Weeks 1-4)**
1. **Schema Alignment** (All teams)
   - Agree on biome.yaml specification
   - Implement parsing in respective systems
   - Test end-to-end manifest processing

2. **Service Registration** (All teams)
   - Implement Songbird pattern consistently
   - Register services with standard metadata
   - Enable cross-Primal discovery

3. **Basic Authentication** (BearDog + All teams)
   - Implement token-based authentication
   - Validate tokens in all Primals
   - Test secure cross-Primal communication

### **Phase 2: Advanced Integration (Weeks 5-8)**
1. **Automated Provisioning** (Toadstool + NestGate)
   - Volume provisioning from manifest
   - Mount points in execution environments
   - Storage lifecycle management

2. **Agent Deployment** (Toadstool + Squirrel)
   - Agent runtime in execution environments
   - MCP protocol integration
   - Plugin execution coordination

3. **Security Integration** (BearDog + All teams)
   - Unified security policies
   - Cross-Primal audit trails
   - Threat correlation across services

### **Phase 3: Optimization (Weeks 9-11)**
1. **Performance Tuning**
   - Cross-Primal communication optimization
   - Resource usage optimization
   - Scaling and load balancing

2. **End-to-End Testing**
   - Complete biome deployment tests
   - Multi-Primal workflow validation
   - Failure recovery testing

3. **Documentation and Training**
   - Integration documentation
   - Team coordination procedures
   - Deployment guides

---

## 📊 **Success Metrics**

### **Technical Milestones**
- [ ] Single `biome.yaml` orchestrates all 5 Primals
- [ ] Sub-60-second biomeOS bootstrap time
- [ ] Cross-Primal authentication working
- [ ] Automated storage provisioning from manifest
- [ ] End-to-end service discovery through Songbird
- [ ] AI agents deployable from manifest
- [ ] Unified security policy enforcement

### **User Experience Goals**
- [ ] "Grandma-safe" installation from single ISO
- [ ] Zero-configuration Primal discovery
- [ ] Self-healing and automatic recovery  
- [ ] Real-time monitoring dashboard
- [ ] Simple biome.yaml creates complex systems

---

## 🚦 **Team Coordination Protocol**

### **Weekly Sync Schedule**
- **Monday:** Technical sync across all teams
- **Wednesday:** Integration testing and issue resolution
- **Friday:** Progress review and next week planning

### **Communication Channels**
- **Integration Issues:** Central issue tracker
- **Technical Discussions:** Team coordination channel
- **Documentation:** Shared documentation repository

### **Integration Dependencies**
```
Week 1: Toadstool schema → All teams can align
Week 2: Songbird patterns → All teams can register  
Week 3: BearDog auth → All teams can authenticate
Week 4: End-to-end testing → Full integration validation
```

---

## 🎯 **Final Notes for Teams**

### **What You DON'T Need to Change**
- **Core functionality** - your systems work great
- **Internal architecture** - keep your proven patterns
- **Performance optimizations** - maintain your efficiency

### **What You DO Need to Add**
- **biomeOS integration points** (specified above)
- **Standard service registration** (Songbird pattern)
- **Cross-Primal communication** (authentication, discovery)

### **Integration Philosophy**
This is **enhancement, not replacement**. Each Primal remains autonomous and fully functional while gaining the ability to work together as biomeOS.

**Remember:** We're not building from scratch - we're connecting proven systems. The hard work is done; now we're making them dance together.

---

**Start immediately. The foundation is exceptional, the gaps are well-defined, and success is inevitable.**

**Next Steps:** Each team lead should review this spec and confirm timeline and resource allocation by end of week. 