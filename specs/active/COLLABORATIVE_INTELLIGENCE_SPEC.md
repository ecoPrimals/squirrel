# Collaborative Intelligence Specification - Squirrel

**Version**: 1.0  
**Date**: January 11, 2026  
**Status**: ✅ Committed - 5 Week Implementation  
**Priority**: Critical Path  
**Initiative**: ecoPrimals Cross-Primal Collaboration

---

## 🎯 Vision

**Transform from**: "AI decides, user watches" (passive)  
**Transform to**: "Human and AI collaborate as equals" (active)

**Impact**: 10x faster system bootstrapping through human expertise + AI learning

---

## 📋 Overview

### **What**
Squirrel will learn from user graph modifications, suggest improvements with transparent reasoning, and enable users to bootstrap new systems 10x faster through collaborative intelligence.

### **Why**
- Current: Users wait for AI to learn patterns (slow, 2-4 weeks)
- Future: Users teach AI their expertise (fast, 2-4 days)
- Result: Bidirectional learning = exponential improvement

### **How**
- Learn from every user modification
- Build user preference models
- Suggest improvements with reasoning
- Enable template sharing
- Predict success probability

---

## 🏗️ Architecture

### **New Components**

```
Squirrel
├── Learning System (NEW)
│   ├── Pattern Extractor
│   │   └── Extract patterns from graph modifications
│   ├── User Model Builder
│   │   └── Build personalized preference models
│   ├── Community Aggregator
│   │   └── Aggregate and rank community patterns
│   └── Success Tracker
│       └── Track modification outcomes and metrics
│
├── Reasoning Engine (NEW)
│   ├── Suggestion Generator
│   │   └── Generate context-aware improvements
│   ├── Confidence Calculator
│   │   └── Calculate multi-factor confidence scores
│   ├── Alternative Analyzer
│   │   └── Consider and rank alternatives
│   └── Explanation Builder
│       └── Build transparent reasoning traces
│
├── JSON-RPC API (EXTENDED)
│   ├── Existing (4 methods)
│   │   ├── query_ai
│   │   ├── list_providers
│   │   ├── health_check
│   │   └── announce_capabilities
│   │
│   └── New (6 methods)
│       ├── learn.from_modification
│       ├── suggest.improvements
│       ├── recommend.templates
│       ├── explain.suggestion
│       ├── patterns.for_user
│       └── confidence.score
│
└── Storage Integration (NEW)
    ├── Pattern Cache (in-memory, Redis-backed)
    ├── User Model Cache (in-memory, TTL: 1 hour)
    └── NestGate Client (persistent storage)
```

---

## 📡 API Specification

### **Method 1: `learn.from_modification`**

**Purpose**: Learn from user's graph modification and outcome

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: < 100ms (async processing)  
**When**: After user modifies and deploys a graph

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "learn.from_modification",
  "params": {
    "original": {
      "nodes": [...],
      "edges": [...],
      "metadata": {...}
    },
    "modified": {
      "nodes": [...],
      "edges": [...],
      "metadata": {...}
    },
    "outcome": "success|failure|pending",
    "user_id": "string",
    "niche_type": "string",
    "execution_metrics": {
      "duration_ms": 5000,
      "resource_usage": {"cpu": 0.3, "memory_mb": 512},
      "success_rate": 0.95,
      "error_count": 0
    },
    "context": {
      "deployment_target": "production|staging|development",
      "modification_reason": "optional user-provided reason"
    }
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "learned": true,
    "patterns_updated": 3,
    "user_model_confidence": 0.78,
    "community_contribution": true,
    "insights": [
      "User prefers parallel execution over sequential",
      "Modified graph reduced latency by 40%",
      "Pattern added to successful templates"
    ],
    "affected_recommendations": 5
  },
  "id": 1
}
```

**Error Codes**:
- `-32602`: Invalid params (malformed graph structure)
- `1001`: Learning system unavailable
- `1002`: Storage backend error

---

### **Method 2: `suggest.improvements`**

**Purpose**: AI suggests improvements to a graph based on learned patterns

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: 200-500ms  
**When**: User opens graph editor or requests suggestions

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "suggest.improvements",
  "params": {
    "graph": {
      "nodes": [...],
      "edges": [...],
      "metadata": {...}
    },
    "user_id": "string (optional)",
    "niche_type": "string",
    "context": {
      "deployment_target": "production|development",
      "priority": "performance|reliability|cost",
      "max_suggestions": 5
    }
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "suggestions": [
      {
        "id": "suggestion-001",
        "type": "add_node|modify_node|remove_node|add_edge|modify_edge",
        "description": "Add caching layer before database queries",
        "reasoning": "92% of similar graphs benefit from caching. Expected 35% latency reduction based on 124 production deployments.",
        "confidence": 0.92,
        "impact": {
          "latency_improvement": "35%",
          "resource_savings": "20%",
          "complexity_increase": "low",
          "implementation_time": "15-30 minutes"
        },
        "alternatives": [
          {
            "description": "Use connection pooling instead",
            "confidence": 0.70,
            "pros": ["Simpler", "Lower memory"],
            "cons": ["Less effective", "Only 25% improvement"]
          }
        ],
        "data_sources": [
          {"type": "community_graphs", "count": 124, "relevance": 0.95},
          {"type": "user_history", "count": 15, "relevance": 0.88}
        ],
        "modification": {
          "action": "insert_node",
          "node": {
            "id": "cache-001",
            "type": "redis_cache",
            "config": {...}
          },
          "position": "before_node_id_5"
        }
      }
    ],
    "overall_confidence": 0.87,
    "applied_count": 0,
    "personalization_score": 0.82
  },
  "id": 2
}
```

---

### **Method 3: `recommend.templates`**

**Purpose**: Recommend graph templates based on niche type and user preferences

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: 100-300ms  
**When**: User starts new graph or browses templates

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "recommend.templates",
  "params": {
    "niche_type": "web_service|data_pipeline|ml_training|etc",
    "user_id": "string (optional)",
    "filters": {
      "max_complexity": 10,
      "min_success_rate": 0.8,
      "preferred_primals": ["songbird", "nestgate"],
      "exclude_patterns": []
    },
    "page": 1,
    "page_size": 10
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "recommendations": [
      {
        "template_id": "template-web-service-001",
        "name": "High-Availability Web Service",
        "description": "Load balanced web service with auto-scaling and caching",
        "success_rate": 0.94,
        "usage_count": 248,
        "complexity": 7,
        "confidence": 0.91,
        "reasoning": "Matches niche type (web_service). 94% success rate across 248 deployments. Similar to your graph from 3 weeks ago.",
        "match_score": 0.89,
        "personalization_factors": [
          "You prefer parallel execution (weight: 0.8)",
          "You typically use Songbird for coordination (weight: 0.6)",
          "Similar to your successful 'api-service-v2' graph"
        ],
        "preview": {
          "node_count": 12,
          "estimated_resources": {"cpu": 2.0, "memory_gb": 4},
          "estimated_cost": "medium",
          "key_features": ["load_balancing", "auto_scaling", "caching"],
          "primals_used": ["songbird", "nestgate", "beardog"]
        },
        "author": {
          "type": "community|user",
          "id": "user-456",
          "reputation": 0.95
        }
      }
    ],
    "total": 15,
    "page": 1,
    "page_size": 10
  },
  "id": 3
}
```

---

### **Method 4: `explain.suggestion`**

**Purpose**: Get detailed explanation for a specific suggestion

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: < 50ms (cached explanations)  
**When**: User clicks "why?" on a suggestion

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "explain.suggestion",
  "params": {
    "suggestion_id": "suggestion-001"
  },
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "suggestion_id": "suggestion-001",
    "explanation": {
      "summary": "Add caching layer to reduce database load and improve response times",
      "reasoning": {
        "pattern_analysis": "This pattern appears in 92% of successful production web services",
        "performance_data": "Average 35% latency reduction across 124 similar deployments",
        "user_history": "You've applied similar caching in 3 previous graphs with positive outcomes",
        "community_validation": "Top-rated pattern by 89 users in web_service niche"
      },
      "confidence_breakdown": {
        "pattern_match": 0.95,
        "historical_success": 0.92,
        "user_preference_alignment": 0.88,
        "community_validation": 0.89,
        "overall": 0.92
      },
      "alternatives_considered": [
        {
          "option": "Connection pooling only",
          "confidence": 0.70,
          "pros": ["Simpler implementation", "Lower memory usage"],
          "cons": ["Less effective than caching", "Only 25% improvement"],
          "why_not_chosen": "Caching provides better performance gain for your use case (2.8x vs 1.3x)"
        }
      ],
      "data_sources": [
        {
          "type": "community_graphs",
          "count": 124,
          "relevance": 0.95,
          "details": "Graphs with similar structure and niche type",
          "date_range": "2024-01-01 to 2026-01-11"
        },
        {
          "type": "user_history",
          "count": 15,
          "relevance": 0.88,
          "details": "Your previous graph modifications and outcomes"
        }
      ],
      "expected_impact": {
        "latency": {"current": "200ms", "predicted": "130ms", "improvement": "35%", "confidence": 0.88},
        "resource_usage": {"cpu_increase": "+10%", "memory_increase": "+200MB"},
        "cost": {"change": "+$15/month", "roi": "positive (saves $50/month in compute)"},
        "reliability": {"expected_change": "+2% uptime"}
      },
      "implementation": {
        "complexity": "low",
        "estimated_time": "15-30 minutes",
        "steps": ["Add Redis dependency", "Configure cache client", "Wrap DB queries"],
        "rollback_plan": "Remove cache node, revert to direct DB calls"
      }
    }
  },
  "id": 4
}
```

---

### **Method 5: `patterns.for_user`**

**Purpose**: Show what AI has learned about a specific user

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: < 100ms  
**When**: User views AI insights or profile

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "patterns.for_user",
  "params": {
    "user_id": "string",
    "include_community": true
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "user_id": "user-123",
    "model_confidence": 0.78,
    "graph_count": 23,
    "modification_count": 67,
    "learning_started": "2025-12-01T00:00:00Z",
    "preferences": {
      "execution_style": "parallel_preferred",
      "resource_priority": "performance_over_cost",
      "favorite_primals": ["songbird", "nestgate", "squirrel"],
      "typical_complexity": 6.5,
      "success_rate": 0.89
    },
    "preferred_patterns": [
      {
        "pattern_id": "pattern-caching-001",
        "name": "Redis caching layer",
        "usage_count": 12,
        "success_rate": 0.95,
        "confidence": 0.92,
        "last_used": "2026-01-05T00:00:00Z"
      }
    ],
    "avoided_patterns": [
      {
        "pattern_id": "pattern-sync-db-001",
        "name": "Synchronous database calls",
        "reason": "User consistently replaces with async patterns",
        "replacement_count": 8
      }
    ],
    "learning_insights": [
      "User prefers caching over repeated computation",
      "User typically parallelizes independent operations",
      "User avoids synchronous blocking operations",
      "User prefers explicit error handling over implicit"
    ],
    "community_comparison": {
      "user_success_rate": 0.89,
      "community_average": 0.82,
      "percentile": 78,
      "unique_patterns": 5,
      "contributions": 3
    }
  },
  "id": 5
}
```

---

### **Method 6: `confidence.score`**

**Purpose**: Predict success probability for a graph before deployment

**Protocol**: JSON-RPC 2.0 over Unix sockets  
**Latency Target**: 100-200ms  
**When**: Before deployment (pre-flight check)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "confidence.score",
  "params": {
    "graph": {
      "nodes": [...],
      "edges": [...],
      "metadata": {...}
    },
    "user_id": "string (optional)",
    "niche_type": "string",
    "deployment_context": {
      "environment": "production|staging|development",
      "scale": "small|medium|large",
      "criticality": "low|medium|high"
    }
  },
  "id": 6
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "overall_confidence": 0.87,
    "success_probability": 0.85,
    "risk_level": "low|medium|high",
    "risk_score": 0.15,
    "analysis": {
      "strengths": [
        "Well-tested pattern (95% success rate in community)",
        "Appropriate for niche type",
        "Matches user's successful patterns",
        "All required primals available and healthy"
      ],
      "concerns": [
        "Higher complexity than user's typical graphs (8 vs 6.5 average)",
        "Some nodes untested in production (2 of 15 nodes)"
      ],
      "warnings": [],
      "recommendations": [
        "Consider simplifying node dependencies (complexity reduction)",
        "Test in staging environment first",
        "Monitor node-7 and node-12 closely (new patterns)"
      ]
    },
    "confidence_breakdown": {
      "pattern_validation": {"score": 0.95, "weight": 0.4},
      "user_alignment": {"score": 0.82, "weight": 0.3},
      "community_validation": {"score": 0.88, "weight": 0.2},
      "resource_availability": {"score": 0.90, "weight": 0.1}
    },
    "predicted_metrics": {
      "success_rate": "85%",
      "avg_execution_time": "3.2 seconds",
      "resource_usage": {"cpu": 1.8, "memory_gb": 3.5},
      "failure_modes": [
        {"type": "timeout", "probability": 0.08},
        {"type": "resource_exhaustion", "probability": 0.05},
        {"type": "dependency_failure", "probability": 0.02}
      ]
    },
    "similar_graphs": {
      "count": 34,
      "avg_success_rate": 0.87,
      "total_deployments": 112,
      "most_similar_id": "graph-456"
    }
  },
  "id": 6
}
```

---

## 🔌 Integration Points

### **With NestGate** (Storage)

**Dependencies**:
- Template storage/retrieval
- Modification history storage
- Pattern metadata storage
- Community pattern sharing

**Timeline**: Week 2 (schema design), Week 4 (integration)

**API Needs**:
```rust
// NestGate must provide:
store_template(template: Template) -> Result<TemplateId>
retrieve_template(id: TemplateId) -> Result<Template>
list_templates(filters: TemplateFilters) -> Result<Vec<Template>>
store_modification_history(history: ModificationHistory) -> Result<()>
store_pattern(pattern: Pattern) -> Result<PatternId>
```

---

### **With petalTongue** (UI)

**Dependencies**:
- Graph editor WebSocket (optional, for real-time suggestions)
- Visualization of reasoning traces
- User modification tracking

**Timeline**: Week 4 (integration), Week 5 (polish)

**Provided APIs**: All 6 JSON-RPC methods

**Optional**: WebSocket for streaming suggestions during editing

---

### **With Songbird** (Coordination/Validation)

**Dependencies**:
- Graph validation results (for learning)
- Primal availability info (for suggestions)

**Timeline**: Week 3 (basic), Week 5 (advanced)

**API Needs**:
```rust
// Songbird should provide:
validate_graph(graph: Graph) -> Result<ValidationResult>
check_primal_availability(primal: String) -> Result<bool>
```

---

### **With biomeOS** (Orchestration)

**Dependencies**:
- Graph execution outcomes (for learning)
- Real-time execution metrics
- User modification events

**Timeline**: Week 4 (end-to-end), Week 6 (live integration)

**Provided APIs**: All 6 JSON-RPC methods

---

## 📅 Implementation Timeline

### **Week 1: Foundation**
- Set up learning infrastructure
- Design data models for patterns
- Create storage schema (with NestGate)
- Implement basic pattern matching
- **Deliverables**: Data structures, storage schema, basic algorithm, unit tests

---

### **Week 2: Core Learning**
- Implement user preference modeling
- Build pattern extraction algorithms
- Create success/failure tracking
- Add community pattern aggregation
- **Deliverables**: User model, pattern extraction, success tracker, integration tests

---

### **Week 3: JSON-RPC Part 1**
- Implement `learn.from_modification`
- Implement `patterns.for_user`
- Implement `confidence.score`
- **Deliverables**: 3 methods, API docs, integration tests, benchmarks

---

### **Week 4: JSON-RPC Part 2**
- Implement `suggest.improvements`
- Implement `recommend.templates`
- Implement `explain.suggestion`
- **Deliverables**: 3 methods, complete docs, reasoning system, integration tests

---

### **Week 5: Reasoning & Polish**
- Enhance reasoning traces
- Add confidence explanations
- Optimize performance
- Complete documentation
- End-to-end testing
- **Deliverables**: Complete system, optimizations, full docs, E2E tests

---

## 📊 Success Metrics

### **Technical Metrics**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Suggestion latency | < 500ms | p95 response time |
| Learning latency | < 100ms | Async processing |
| Confidence accuracy | > 80% | Predicted vs actual |
| Pattern extraction | > 90% | Relevant patterns found |
| Memory usage | < 1GB | At 10K patterns |
| Cache hit rate | > 85% | Pattern cache hits |

---

### **User Metrics**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Suggestion acceptance | > 70% | % applied by users |
| User satisfaction | > 8/10 | Survey rating |
| Bootstrap time | 10x faster | Before vs after |
| Reasoning clarity | > 8/10 | Survey rating |
| Learning speed | Visible in 5 graphs | User perception |
| Template adoption | > 50% | % using templates |

---

### **Community Metrics**

| Metric | Target | Measurement |
|--------|--------|-------------|
| Template contributions | > 100 users | Contributing templates |
| Success rate improvement | +10% | Before vs after |
| Pattern diversity | > 500 unique | Distinct patterns |

---

## 💡 Key Principles

### **1. Human and AI as Equals**
- AI suggests, human decides
- Human teaches, AI learns
- Neither is subservient
- **Implementation**: Every suggestion is just that—a suggestion. User has final say.

### **2. Transparent Reasoning**
- Every suggestion explains "why"
- Show confidence calculation
- List alternatives considered
- Acknowledge uncertainty
- **Implementation**: Comprehensive reasoning traces for all suggestions.

### **3. User Always in Control**
- User can override any suggestion
- User can teach AI their preferences
- User can see what AI learned
- **Implementation**: `patterns.for_user` shows learning, user can modify.

### **4. Learn Together**
- AI learns from user modifications
- User learns from AI suggestions
- Both improve over time
- **Implementation**: Bidirectional learning loop with clear feedback.

### **5. Bootstrap Fast**
- User expertise + AI learning = 10x
- Leverage community intelligence
- Personalize to user preferences
- **Implementation**: Community patterns + user preferences = targeted suggestions.

---

## 🧪 Testing Strategy

### **Unit Tests** (Ongoing)
- Pattern extraction accuracy (> 90%)
- User model building (confidence > 0.7)
- Confidence calculation (accuracy > 80%)
- Reasoning trace generation (completeness)
- **Target**: 90% code coverage

### **Integration Tests** (Week 4, Week 6)
- JSON-RPC method testing (all 6 methods)
- NestGate storage integration
- Multi-primal coordination
- End-to-end learning flows
- **Target**: All critical paths covered

### **Performance Tests** (Week 5)
- < 100ms for learning (async)
- < 500ms for suggestions (p95)
- < 200ms for confidence scoring (p95)
- < 1GB memory for 10,000 patterns
- **Target**: Production-ready performance

### **User Acceptance** (Week 6, Week 8)
- Real users test suggestions (> 70% acceptance)
- Measure suggestion usefulness (> 8/10 rating)
- Validate reasoning clarity (> 8/10 rating)
- Collect qualitative feedback
- **Target**: Positive user reception

---

## 🔒 Security Considerations

### **Privacy**
- User modifications are private by default
- Opt-in for community pattern sharing
- Anonymize data in community aggregation
- User can delete learning data

### **Authorization**
- Only authorized users can learn from modifications
- BearDog validates user permissions
- Templates have author attribution
- Prevent malicious pattern injection

### **Data Integrity**
- Validate all input graphs
- Sanitize user-provided reasons
- Prevent SQL injection in queries
- Rate limit learning requests

---

## 📚 Documentation

### **For Users**
- What is Collaborative Intelligence?
- How does AI learn from me?
- How to interpret suggestions
- How to teach AI preferences

### **For Developers**
- API reference (6 methods)
- Integration examples
- Testing guidelines
- Architecture diagrams

### **For DevOps**
- Deployment considerations
- Performance tuning
- Monitoring and alerts
- Backup and recovery

---

## 🚀 Status

**Current**: ✅ Committed (5 weeks, critical path)  
**Next**: Kick-off meeting (Week 1)  
**Blockers**: None (dependencies identified and tracked)

**Contact**: Squirrel Team (@squirrel-team)  
**Coordination**: #collaborative-intelligence  
**Updates**: Weekly on Wednesdays, 2pm UTC

---

**Version**: 1.0  
**Last Updated**: January 11, 2026  
**Status**: Active Specification

