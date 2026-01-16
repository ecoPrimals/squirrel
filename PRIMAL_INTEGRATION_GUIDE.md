# 🔗 Primal Integration Guide

**Version**: 3.0+  
**Last Updated**: January 15, 2026  
**Audience**: ecoPrimals Developers, System Architects

---

## 🎯 Purpose

This guide explains how to integrate your primal with Squirrel to leverage:
- **Intelligent AI routing** for your primal's AI needs
- **Dynamic tool registration** to expose your capabilities to the ecosystem
- **Capability-based discovery** to find and use other primals' tools
- **MCP protocol** to interact with external agents

---

## 🚀 Quick Start

### 1. Connect to Squirrel for AI

**Simple text generation**:
```rust
use reqwest::Client;

async fn generate_text(prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .post("http://localhost:9010/ai/generate-text")
        .json(&serde_json::json!({
            "prompt": prompt,
            "max_tokens": 100,
            "constraints": ["prefer_local"]  // Privacy-first
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    Ok(response["text"].as_str().unwrap().to_string())
}
```

**That's it!** Squirrel handles provider selection, routing, and fallback.

---

### 2. Register Your Tools

**Tell Squirrel what your primal can do**:

```bash
curl -X POST http://localhost:9010/api/v1/providers/register \
  -H "Content-Type: application/json" \
  -d '{
    "provider_id": "nestgate",
    "provider_name": "Nestgate Data Primal",
    "advertised_capabilities": [
      {
        "action": "data.query",
        "input_schema": {
          "type": "object",
          "properties": {
            "query": {"type": "string"},
            "source": {"type": "string"}
          },
          "required": ["query"]
        },
        "output_schema": {
          "type": "object",
          "properties": {
            "results": {"type": "array"},
            "count": {"type": "integer"}
          }
        },
        "cost_per_unit": 0.001,
        "avg_latency_ms": 200,
        "quality": "high",
        "metadata": {
          "category": "data_operations",
          "supports_streaming": true
        }
      }
    ]
  }'
```

**Done!** Your tools are now discoverable ecosystem-wide.

---

## 📊 Integration Levels

### Level 1: AI Consumer (Easiest)

**What**: Use Squirrel's AI routing for your primal's needs  
**Benefit**: Intelligent routing, cost optimization, privacy control  
**Time**: 30 minutes  
**Endpoints**: `/ai/generate-text`, `/ai/generate-image`

---

### Level 2: Tool Provider (Intermediate)

**What**: Register your primal's capabilities with Squirrel  
**Benefit**: Ecosystem-wide discovery, intelligent routing to you  
**Time**: 2-4 hours  
**Endpoints**: `/api/v1/providers/register`, implement tool handlers

---

### Level 3: Full Integration (Advanced)

**What**: Both consume AI and provide tools  
**Benefit**: Full ecosystem participation, maximum flexibility  
**Time**: 1 day  
**Includes**: Capability discovery, cross-primal communication, MCP support

---

## 🔧 Level 1: AI Consumer

### Use Case: Nestgate Needs AI for Data Analysis

```rust
// Nestgate wants to use AI to analyze query patterns
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct AiRequest {
    prompt: String,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    constraints: Vec<String>,
}

#[derive(Deserialize)]
struct AiResponse {
    text: String,
    provider_id: String,
    cost_usd: f64,
}

async fn analyze_query_pattern(queries: Vec<String>) -> Result<String, Error> {
    let prompt = format!(
        "Analyze these database queries and identify patterns: {:?}",
        queries
    );
    
    let request = AiRequest {
        prompt,
        max_tokens: 500,
        constraints: vec!["optimize_cost".to_string()],  // Use free local AI
    };
    
    let response: AiResponse = reqwest::Client::new()
        .post("http://localhost:9010/ai/generate-text")
        .json(&request)
        .send()
        .await?
        .json()
        .await?;
    
    info!(
        "AI analysis completed using {} (cost: ${:.4})",
        response.provider_id, response.cost_usd
    );
    
    Ok(response.text)
}
```

### Benefits

- ✅ **No AI provider management** - Squirrel handles it
- ✅ **Cost optimization** - Routes to cheapest provider
- ✅ **Privacy control** - Can force local execution
- ✅ **Automatic fallback** - If one provider fails, tries others
- ✅ **Cost tracking** - Know exactly what you're spending

---

## 🛠️ Level 2: Tool Provider

### Use Case: Nestgate Provides Data Query Capability

**Step 1: Define Your Tools**

```json
{
  "provider_id": "nestgate",
  "provider_name": "Nestgate Data Primal",
  "advertised_capabilities": [
    {
      "action": "data.query",
      "input_schema": {
        "type": "object",
        "properties": {
          "query": {
            "type": "string",
            "description": "SQL or natural language query"
          },
          "source": {
            "type": "string",
            "description": "Data source identifier"
          },
          "format": {
            "type": "string",
            "enum": ["json", "csv", "parquet"],
            "default": "json"
          }
        },
        "required": ["query"]
      },
      "output_schema": {
        "type": "object",
        "properties": {
          "results": {
            "type": "array",
            "description": "Query results"
          },
          "count": {
            "type": "integer"
          },
          "execution_time_ms": {
            "type": "integer"
          }
        }
      },
      "cost_per_unit": 0.001,
      "avg_latency_ms": 200,
      "quality": "high",
      "metadata": {
        "category": "data_operations",
        "supports_streaming": true,
        "max_result_size": 1000000
      }
    },
    {
      "action": "data.transform",
      "input_schema": {...},
      "output_schema": {...},
      "cost_per_unit": 0.0005,
      "avg_latency_ms": 150,
      "quality": "high"
    }
  ]
}
```

**Step 2: Register on Startup**

```rust
use reqwest::Client;
use serde_json::json;

async fn register_with_squirrel() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    let registration = json!({
        "provider_id": "nestgate",
        "provider_name": "Nestgate Data Primal",
        "advertised_capabilities": [
            {
                "action": "data.query",
                "input_schema": {...},
                "output_schema": {...},
                "cost_per_unit": 0.001,
                "avg_latency_ms": 200,
                "quality": "high"
            }
        ]
    });
    
    let response = client
        .post("http://localhost:9010/api/v1/providers/register")
        .json(&registration)
        .send()
        .await?;
    
    if response.status().is_success() {
        info!("✅ Successfully registered with Squirrel");
    } else {
        warn!("⚠️ Registration failed: {}", response.status());
    }
    
    Ok(())
}

// Call during primal initialization
#[tokio::main]
async fn main() {
    // ... other initialization ...
    
    register_with_squirrel().await.expect("Failed to register tools");
    
    // ... start server ...
}
```

**Step 3: Implement Tool Endpoints**

```rust
use warp::{Filter, Reply};

async fn handle_data_query(request: DataQueryRequest) -> Result<impl Reply, warp::Rejection> {
    // Your implementation
    let results = execute_query(&request.query, &request.source).await?;
    
    let response = json!({
        "results": results,
        "count": results.len(),
        "execution_time_ms": 150
    });
    
    Ok(warp::reply::json(&response))
}

// Expose endpoint for Squirrel to call
let data_query_route = warp::path!("tools" / "data.query")
    .and(warp::post())
    .and(warp::body::json())
    .and_then(handle_data_query);
```

**Step 4: Handle Deregistration on Shutdown**

```rust
async fn deregister_from_squirrel() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    client
        .delete("http://localhost:9010/api/v1/providers/nestgate")
        .send()
        .await?;
    
    info!("✅ Deregistered from Squirrel");
    Ok(())
}

// Call during graceful shutdown
```

---

## 🌐 Level 3: Full Integration

### Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   Your Primal (Nestgate)                │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  On Startup:                                            │
│  1. Register tools with Squirrel                        │
│  2. Discover other primals' tools                       │
│                                                          │
│  During Operation:                                      │
│  3. Use AI via Squirrel (intelligent routing)           │
│  4. Call other primals' tools via Squirrel              │
│  5. Respond to tool requests from other primals         │
│                                                          │
│  ┌──────────────┐      ┌──────────────┐                │
│  │  AI Requests │      │ Tool Calls   │                │
│  │      ↓       │      │      ↓       │                │
│  │   Squirrel   │      │   Squirrel   │                │
│  │      ↓       │      │      ↓       │                │
│  │  OpenAI/     │      │  Other       │                │
│  │  Ollama      │      │  Primals     │                │
│  └──────────────┘      └──────────────┘                │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### Example: Nestgate Full Integration

```rust
use squirrel_sdk::SquirrelClient;

struct NestgateIntegration {
    squirrel: SquirrelClient,
}

impl NestgateIntegration {
    async fn new() -> Result<Self, Error> {
        let squirrel = SquirrelClient::connect("http://localhost:9010").await?;
        
        // Register our tools
        squirrel.register_tools(vec![
            Tool::new("data.query", ...),
            Tool::new("data.transform", ...),
        ]).await?;
        
        Ok(Self { squirrel })
    }
    
    // Use AI for data analysis
    async fn analyze_data(&self, data: &[Record]) -> Result<String, Error> {
        self.squirrel.generate_text(
            format!("Analyze this data: {:?}", data),
            GenerateOptions {
                max_tokens: 500,
                constraints: vec![Constraint::PreferLocal],
            }
        ).await
    }
    
    // Use another primal's tool (via Squirrel)
    async fn deploy_analysis_job(&self, config: JobConfig) -> Result<String, Error> {
        // Discover who provides compute.deploy
        let providers = self.squirrel.find_action_providers("compute.deploy").await?;
        
        // Squirrel routes to best provider (probably Toadstool)
        let result = self.squirrel.execute_action(
            "compute.deploy",
            serde_json::to_value(config)?,
            ExecuteOptions {
                constraints: vec![Constraint::OptimizeCost],
            }
        ).await?;
        
        Ok(result.job_id)
    }
    
    // Respond to tool requests (already implemented via HTTP endpoints)
    async fn handle_query_request(&self, req: QueryRequest) -> Result<QueryResponse, Error> {
        // Your implementation
        ...
    }
}
```

---

## 📋 Tool Schema Best Practices

### Good Schema Example

```json
{
  "action": "data.query",
  "input_schema": {
    "type": "object",
    "properties": {
      "query": {
        "type": "string",
        "description": "SQL query or natural language",
        "examples": [
          "SELECT * FROM users WHERE active = true",
          "Show me all active users"
        ]
      },
      "max_results": {
        "type": "integer",
        "default": 100,
        "minimum": 1,
        "maximum": 10000
      }
    },
    "required": ["query"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "results": {"type": "array"},
      "count": {"type": "integer"},
      "execution_time_ms": {"type": "integer"}
    },
    "required": ["results", "count"]
  }
}
```

### Tips

1. **Use JSON Schema** - Full validation support
2. **Provide examples** - Help consumers understand usage
3. **Set defaults** - Make optional parameters easy
4. **Document constraints** - Min/max values, enums
5. **Version your schemas** - Allow evolution

---

## 🎯 Naming Conventions

### Action Names

**Format**: `category.operation`

**Examples**:
- `data.query` - Query data
- `data.transform` - Transform data
- `compute.deploy` - Deploy compute
- `auth.verify` - Verify authentication
- `service.discover` - Discover services

**Categories**:
- `data.*` - Data operations
- `compute.*` - Compute operations
- `auth.*` - Authentication/authorization
- `service.*` - Service mesh operations
- `file.*` - File operations
- `git.*` - Version control
- `code.*` - Code analysis/generation

---

## 🔒 Security Considerations

### 1. Validate Inputs

```rust
async fn handle_query(req: QueryRequest) -> Result<Response, Error> {
    // Validate against schema
    validate_schema(&req, &QUERY_INPUT_SCHEMA)?;
    
    // Sanitize inputs
    let safe_query = sanitize_sql(&req.query)?;
    
    // Execute
    execute(safe_query).await
}
```

### 2. Rate Limiting

```rust
use governor::{Quota, RateLimiter};

let limiter = RateLimiter::direct(Quota::per_second(100));

async fn handle_request(req: Request) -> Result<Response, Error> {
    limiter.check().map_err(|_| Error::RateLimitExceeded)?;
    // ... process request ...
}
```

### 3. Authentication

```rust
async fn verify_caller(req: &Request) -> Result<CallerId, Error> {
    let token = req.headers().get("Authorization")?;
    verify_token(token).await
}
```

---

## 📊 Monitoring & Observability

### Track Tool Usage

```rust
use prometheus::{Counter, Histogram};

lazy_static! {
    static ref TOOL_CALLS: Counter = Counter::new(
        "tool_calls_total",
        "Total tool calls"
    ).unwrap();
    
    static ref TOOL_DURATION: Histogram = Histogram::new(
        "tool_duration_seconds",
        "Tool execution duration"
    ).unwrap();
}

async fn handle_tool_call(req: Request) -> Result<Response, Error> {
    TOOL_CALLS.inc();
    let start = Instant::now();
    
    let result = execute_tool(req).await;
    
    TOOL_DURATION.observe(start.elapsed().as_secs_f64());
    
    result
}
```

### Log Important Events

```rust
use tracing::{info, warn, error};

async fn execute_query(query: &str) -> Result<Vec<Row>, Error> {
    info!("Executing query: {}", query);
    
    match run_query(query).await {
        Ok(results) => {
            info!("Query succeeded: {} rows", results.len());
            Ok(results)
        }
        Err(e) => {
            error!("Query failed: {}", e);
            Err(e)
        }
    }
}
```

---

## 🧪 Testing

### Test Tool Registration

```rust
#[tokio::test]
async fn test_register_tools() {
    let client = SquirrelClient::connect("http://localhost:9010").await.unwrap();
    
    let result = client.register_tools(vec![
        Tool::new("test.action", ...),
    ]).await;
    
    assert!(result.is_ok());
    
    // Verify registration
    let actions = client.list_actions().await.unwrap();
    assert!(actions.contains(&"test.action".to_string()));
}
```

### Test Tool Execution

```rust
#[tokio::test]
async fn test_execute_tool() {
    let input = json!({"query": "SELECT * FROM test"});
    
    let result = handle_query(QueryRequest::from_value(input)).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().count, 10);
}
```

---

## 📚 Complete Example: Toadstool Integration

```rust
// File: toadstool/src/squirrel_integration.rs

use squirrel_sdk::{SquirrelClient, Tool, Constraint};
use serde::{Deserialize, Serialize};

pub struct ToadstoolIntegration {
    squirrel: SquirrelClient,
}

impl ToadstoolIntegration {
    pub async fn initialize() -> Result<Self, Error> {
        let squirrel = SquirrelClient::connect("http://localhost:9010").await?;
        
        // Register Toadstool's compute capabilities
        squirrel.register_tools(vec![
            Tool {
                action: "compute.deploy".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "image": {"type": "string"},
                        "replicas": {"type": "integer", "default": 1},
                        "resources": {
                            "type": "object",
                            "properties": {
                                "cpu": {"type": "string"},
                                "memory": {"type": "string"}
                            }
                        }
                    },
                    "required": ["image"]
                }),
                output_schema: json!({
                    "type": "object",
                    "properties": {
                        "deployment_id": {"type": "string"},
                        "status": {"type": "string"}
                    }
                }),
                cost_per_unit: Some(0.10),
                avg_latency_ms: 2000,
                quality: "high".to_string(),
                metadata: json!({
                    "category": "compute",
                    "provider": "kubernetes"
                }),
            },
            Tool {
                action: "compute.scale".to_string(),
                // ... similar definition ...
            },
        ]).await?;
        
        info!("✅ Toadstool tools registered with Squirrel");
        
        Ok(Self { squirrel })
    }
    
    // Use AI to optimize deployment
    pub async fn optimize_deployment(&self, current: Deployment) -> Result<Deployment, Error> {
        let analysis = self.squirrel.generate_text(
            format!("Optimize this deployment: {:?}", current),
            GenerateOptions {
                max_tokens: 300,
                constraints: vec![Constraint::OptimizeCost],  // Use free local AI
            }
        ).await?;
        
        // Parse AI recommendations and apply
        let optimized = parse_recommendations(&analysis)?;
        Ok(optimized)
    }
}

// Implement tool endpoints
async fn handle_deploy(req: DeployRequest) -> Result<DeployResponse, Error> {
    // Your Kubernetes deployment logic
    let deployment_id = deploy_to_k8s(&req).await?;
    
    Ok(DeployResponse {
        deployment_id,
        status: "deployed".to_string(),
    })
}
```

---

## 🎓 Best Practices

1. **Register on startup** - Make tools discoverable immediately
2. **Deregister on shutdown** - Clean up gracefully
3. **Provide detailed schemas** - Help consumers understand your tools
4. **Use semantic versioning** - Version your tools
5. **Monitor usage** - Track calls, errors, latency
6. **Handle errors gracefully** - Return useful error messages
7. **Implement timeouts** - Don't block forever
8. **Use constraints** - Optimize AI routing for your needs
9. **Think capability-based** - Not primal-specific
10. **Document everything** - Help future integrators

---

## 🚀 Next Steps

1. **Start with Level 1** - Use Squirrel for AI
2. **Move to Level 2** - Register your tools
3. **Achieve Level 3** - Full ecosystem integration
4. **Monitor & Iterate** - Track usage, optimize
5. **Contribute** - Share learnings with community

---

## 📖 Additional Resources

- **[USAGE_GUIDE.md](USAGE_GUIDE.md)** - General usage guide
- **[CURSOR_INTEGRATION_COMPLETE.md](CURSOR_INTEGRATION_COMPLETE.md)** - Cursor IDE integration
- **[docs/sessions/2026-01-15/](docs/sessions/2026-01-15/)** - Latest discoveries
- **Squirrel SDK**: `crates/sdk/` (Rust client library)

---

**Questions?** Check the [USAGE_GUIDE.md](USAGE_GUIDE.md) or [troubleshooting docs](TROUBLESHOOTING.md).

**Ready to integrate?** Follow this guide step by step!

---

*Last Updated: January 15, 2026*  
*Version: 3.0+*  
*Status: Production Ready*

