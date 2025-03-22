---
title: "Galaxy MCP Testing Framework"
description: "Comprehensive testing approach for Galaxy MCP integration"
version: "0.1.0"
last_updated: "2025-03-25"
status: "draft"
owners:
  primary: ["DataScienceBioLab", "test-team"]
  reviewers: ["qa-lead", "mcp-team"]
---

# Galaxy MCP Testing Framework

## 1. Overview

This specification defines a comprehensive testing framework for the Galaxy MCP integration, ensuring reliability, performance, and security across all components. It outlines testing methodologies, environments, test categories, and quality metrics to verify that the integration meets all functional and non-functional requirements.

## 2. Testing Principles

- **Shift-Left Testing**: Incorporate testing early in the development lifecycle
- **Continuous Testing**: Automate tests as part of the CI/CD pipeline
- **Test Isolation**: Each test should be independent and deterministic
- **Realistic Data**: Use representative bioinformatics datasets and tools
- **Security-First**: Incorporate security testing throughout the process
- **Coverage-Driven**: Aim for high test coverage across all components
- **User-Centric**: Prioritize tests that validate real user workflows

## 3. Test Environments

### 3.1 Environment Matrix

| Environment | Purpose | Infrastructure | Data | Configuration |
|-------------|---------|----------------|------|---------------|
| Development | Developer testing | Local containers | Sample data | Development config |
| Integration | Component integration | Dedicated test server | Synthetic datasets | Test API keys |
| Staging | Pre-production validation | Mirror of production | Cloned production data | Production-like |
| Production | Final verification | Production system | Production data | Production config |

### 3.2 Environment Setup

Automated environment provisioning using infrastructure as code:

```yaml
# test-environment.yaml
version: '3'
services:
  galaxy-test:
    image: galaxy:latest
    environment:
      - GALAXY_CONFIG_OVERRIDE_DATABASE_CONNECTION=postgresql://galaxy:galaxy@postgres/galaxy
    ports:
      - "8080:80"
    depends_on:
      - postgres
  
  postgres:
    image: postgres:13
    environment:
      - POSTGRES_USER=galaxy
      - POSTGRES_PASSWORD=galaxy
      - POSTGRES_DB=galaxy
    volumes:
      - postgres_data:/var/lib/postgresql/data
  
  mcp-adapter:
    image: galaxy-mcp-adapter:test
    environment:
      - GALAXY_URL=http://galaxy-test:80
      - GALAXY_API_KEY=fakekey
    ports:
      - "9000:8080"
    depends_on:
      - galaxy-test

  test-runner:
    image: galaxy-mcp-test-runner:latest
    volumes:
      - ./test-data:/test-data
      - ./test-results:/test-results
    depends_on:
      - mcp-adapter

volumes:
  postgres_data:
```

## 4. Test Categories

### 4.1 Unit Tests

Testing individual components in isolation:

| Component | Test Focus | Tools | Minimum Coverage |
|-----------|------------|-------|------------------|
| API Client | API request/response handling | Rust test, mockito | 90% |
| Message Translator | MCP message transformations | Rust test, proptest | 95% |
| Schema Validator | Schema validation logic | Rust test, json-schema | 100% |
| Authentication | Auth mechanisms | Rust test, mockito | 95% |
| Error Handling | Error conditions and recovery | Rust test, proptest | 90% |

Example unit test for message translation:

```rust
#[test]
fn test_galaxy_to_mcp_tool_translation() {
    // Arrange
    let galaxy_tool_json = r#"{
        "id": "toolshed.g2.bx.psu.edu/repos/devteam/fastqc/fastqc/0.73",
        "name": "FastQC",
        "version": "0.73",
        "description": "Read Quality reports"
    }"#;
    
    let expected_mcp_tool = r#"{
        "id": "galaxy-tool-fastqc",
        "name": "FastQC",
        "version": "0.73",
        "description": "Read Quality reports",
        "capabilities": ["execute", "get_help", "get_form"]
    }"#;
    
    // Act
    let tool_translator = ToolTranslator::new();
    let actual_mcp_tool = tool_translator.translate_to_mcp(galaxy_tool_json)?;
    
    // Assert
    assert_json_eq!(actual_mcp_tool, expected_mcp_tool);
}
```

### 4.2 Integration Tests

Testing interactions between components:

| Test Suite | Components Tested | Test Cases |
|------------|-------------------|------------|
| API Integration | Adapter + Galaxy API | Authentication, API calls, response handling |
| Message Flow | Client → Adapter → Galaxy | End-to-end message processing |
| Error Propagation | All components | Error generation and handling |
| Configuration | Adapter + Config System | Loading, validation, application |
| Security | All components | Auth, permissions, data protection |

Example integration test for tool execution:

```rust
#[tokio::test]
async fn test_tool_execution_workflow() {
    // Arrange
    let test_server = TestServer::new().await;
    let client = McpClient::new(test_server.url());
    
    // Upload test data
    let data_id = test_server.upload_test_file("test_data/sample.fastq").await?;
    
    // Act: Execute tool via MCP
    let request = ToolExecutionRequest {
        tool_id: "galaxy-tool-fastqc",
        inputs: json!({
            "input_file": data_id
        }),
        output_format: "html",
    };
    
    let response = client.execute_tool(request).await?;
    
    // Assert: Verify execution and results
    assert!(response.status.is_success());
    assert!(response.output_data.contains("fastqc_report.html"));
    
    // Verify the execution in Galaxy history
    let history = test_server.get_current_history().await?;
    assert!(history.contains("FastQC on data"));
}
```

### 4.3 System Tests

End-to-end testing of the entire system:

| Test Suite | Scope | Automation Level |
|------------|-------|------------------|
| Workflow Execution | Complete workflows | Fully automated |
| Data Management | Data upload, download, sharing | Fully automated |
| Tool Discovery | Tool listing and filtering | Fully automated |
| MCP Compatibility | Compliance with MCP standard | Semi-automated |
| Cross-platform | Multiple client environments | Semi-automated |

Example system test for workflow execution:

```python
def test_complete_analysis_workflow():
    """Test end-to-end RNA-seq analysis workflow through MCP."""
    # Setup test environment
    client = MCPClient(base_url="http://localhost:9000")
    client.authenticate(api_key="test_key")
    
    # Upload test data
    fastq_ids = client.upload_data_files([
        "test_data/sample1_R1.fastq", 
        "test_data/sample1_R2.fastq"
    ])
    
    # Execute FASTQC
    fastqc_outputs = client.execute_tool(
        tool_id="galaxy-tool-fastqc",
        inputs={"input_file": fastq_ids},
        wait_for_completion=True
    )
    
    # Execute HISAT2 alignment
    hisat_outputs = client.execute_tool(
        tool_id="galaxy-tool-hisat2",
        inputs={
            "input_1": fastq_ids[0],
            "input_2": fastq_ids[1],
            "reference_genome": "hg38"
        },
        wait_for_completion=True
    )
    
    # Execute featureCounts
    counts_output = client.execute_tool(
        tool_id="galaxy-tool-featurecounts",
        inputs={
            "alignment": hisat_outputs["bam_output"],
            "annotation": "hg38_genes"
        },
        wait_for_completion=True
    )
    
    # Verify outputs
    assert counts_output["status"] == "success"
    counts_data = client.download_data(counts_output["counts_file"])
    
    # Validate results
    gene_counts = parse_counts_file(counts_data)
    assert len(gene_counts) > 20000  # Expect human gene count
    assert "GAPDH" in gene_counts  # Common housekeeping gene
    assert gene_counts["GAPDH"] > 0  # Should have reads
```

### 4.4 Performance Tests

Testing system performance characteristics:

| Test Type | Metrics | Acceptance Criteria |
|-----------|---------|---------------------|
| Load Testing | Throughput, Response time | <500ms at 50 req/sec |
| Stress Testing | Breaking point, Error rate | <1% errors at 200 req/sec |
| Endurance Testing | Memory usage, Performance degradation | <5% degradation after 24h |
| Scalability Testing | Resource usage vs. load | Linear scaling to 100 users |

Example k6 performance test script:

```javascript
// load-test.js
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '1m', target: 10 },   // Ramp up to 10 users
    { duration: '3m', target: 50 },   // Ramp up to 50 users
    { duration: '5m', target: 50 },   // Stay at 50 users
    { duration: '1m', target: 0 },    // Ramp down to 0 users
  ],
  thresholds: {
    http_req_duration: ['p(95)<500'],  // 95% of requests must complete below 500ms
    http_req_failed: ['rate<0.01'],    // Error rate must be less than 1%
  },
};

export default function() {
  const BASE_URL = __ENV.BASE_URL || 'http://localhost:9000';
  
  // Authentication
  const authResponse = http.post(`${BASE_URL}/auth`, JSON.stringify({
    api_key: 'test_key'
  }), {
    headers: { 'Content-Type': 'application/json' },
  });
  
  check(authResponse, {
    'authenticated successfully': (r) => r.status === 200,
  });
  
  const token = authResponse.json('token');
  
  // List available tools
  const toolsResponse = http.get(`${BASE_URL}/tools`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });
  
  check(toolsResponse, {
    'tools retrieved successfully': (r) => r.status === 200,
    'tools contain FastQC': (r) => r.json().some(tool => tool.id.includes('fastqc')),
  });
  
  // Get tool details
  const toolDetailsResponse = http.get(`${BASE_URL}/tools/galaxy-tool-fastqc`, {
    headers: {
      'Authorization': `Bearer ${token}`,
    },
  });
  
  check(toolDetailsResponse, {
    'tool details retrieved successfully': (r) => r.status === 200,
    'tool has execute capability': (r) => r.json().capabilities.includes('execute'),
  });
  
  sleep(1);
}
```

### 4.5 Security Tests

Comprehensive security testing:

| Test Type | Focus Area | Tools |
|-----------|------------|-------|
| Authentication Testing | Token validation, session management | OWASP ZAP, Custom scripts |
| Authorization Testing | Access control, permission checking | Custom scripts |
| Input Validation | SQL injection, command injection | OWASP ZAP, sqlmap |
| Dependency Scanning | Vulnerable dependencies | cargo-audit, OWASP Dependency Check |
| Penetration Testing | Exploitable vulnerabilities | Metasploit, Burp Suite |
| Secrets Management | API key handling, credential storage | Custom audit scripts |

Example security test for authentication:

```python
def test_authentication_security():
    """Test security aspects of the authentication system."""
    client = MCPClient(base_url="http://localhost:9000")
    
    # Test invalid API key
    response = client.authenticate(api_key="invalid_key")
    assert response.status_code == 401
    
    # Test valid API key
    response = client.authenticate(api_key="test_key")
    assert response.status_code == 200
    token = response.json()["token"]
    
    # Test token tampering
    tampered_token = token[:-5] + "12345"  # Change last 5 chars
    response = client.execute_tool_with_token(
        token=tampered_token,
        tool_id="galaxy-tool-fastqc", 
        inputs={"input_file": "test_id"}
    )
    assert response.status_code == 401
    
    # Test expired token
    time.sleep(65)  # Wait for token to expire (60s expiry)
    response = client.execute_tool_with_token(
        token=token,
        tool_id="galaxy-tool-fastqc", 
        inputs={"input_file": "test_id"}
    )
    assert response.status_code == 401
    
    # Test CSRF protection
    headers = {"Authorization": f"Bearer {token}"}
    response = requests.post(
        "http://localhost:9000/tools/execute",
        json={"tool_id": "galaxy-tool-fastqc"},
        headers=headers,
        cookies={"session_id": "stolen_cookie"}
    )
    assert response.status_code in [401, 403]  # Should be rejected
```

## 5. Test Data Management

### 5.1 Test Data Sources

| Data Type | Source | Purpose | Example Files |
|-----------|--------|---------|--------------|
| Sample Datasets | Public repositories | Basic functionality | SRA, ENA samples |
| Synthetic Data | Data generators | Edge cases, performance | Generated FASTQ, BAM |
| Production Samples | Anonymized production | Realistic workflows | Sanitized histories |
| Reference Data | Public databases | Tool execution | Reference genomes |

### 5.2 Test Data Lifecycle

```mermaid
---
title: Test Data Lifecycle
---
flowchart TD
    Acquisition[Data Acquisition]
    Preparation[Data Preparation]
    Versioning[Data Versioning]
    Storage[Data Storage]
    Usage[Test Execution]
    Cleanup[Data Cleanup]
    
    Acquisition --> Preparation
    Preparation --> Versioning
    Versioning --> Storage
    Storage --> Usage
    Usage --> Cleanup
    Cleanup -.-> Acquisition
```

### 5.3 Test Data Repository Structure

```
/test-data/
  /unit/
    /api-client/
      api_responses.json
      error_cases.json
    /message-translator/
      galaxy_tools.json
      mcp_messages.json
  /integration/
    /workflows/
      rna_seq_workflow.ga
      variant_calling_workflow.ga
    /tools/
      fastqc_inputs.json
      hisat_inputs.json
  /system/
    /datasets/
      sample1_R1.fastq
      sample1_R2.fastq
      reference.fa
    /expected_outputs/
      expected_counts.txt
      expected_variants.vcf
  /performance/
    small_dataset.fastq
    medium_dataset.fastq
    large_dataset.fastq
```

## 6. Test Automation

### 6.1 Continuous Integration Pipeline

```mermaid
---
title: CI/CD Testing Pipeline
---
flowchart TD
    Code[Code Change]
    Build[Build]
    UnitTest[Unit Tests]
    Lint[Linting & Static Analysis]
    Security[Security Scan]
    Integration[Integration Tests]
    Performance[Performance Tests]
    Deploy[Deploy to Staging]
    System[System Tests]
    Production[Deploy to Production]
    
    Code --> Build
    Build --> UnitTest
    Build --> Lint
    Build --> Security
    UnitTest --> Integration
    Lint --> Integration
    Security --> Integration
    Integration --> Performance
    Performance --> Deploy
    Deploy --> System
    System --> Production
```

Example GitHub Actions workflow:

```yaml
# .github/workflows/ci.yml
name: Galaxy MCP CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
        components: clippy, rustfmt
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Lint
      run: |
        cargo fmt -- --check
        cargo clippy -- -D warnings
    
    - name: Security scan
      run: cargo audit
    
    - name: Build
      run: cargo build --verbose
    
    - name: Unit tests
      run: cargo test --lib -- --nocapture
    
    - name: Start test environment
      run: docker-compose -f test-environment.yaml up -d
    
    - name: Wait for services
      run: |
        ./scripts/wait-for-services.sh
        sleep 10
    
    - name: Integration tests
      run: cargo test --test '*' -- --nocapture
    
    - name: Performance tests
      run: |
        npm install -g k6
        k6 run tests/performance/load-test.js
    
    - name: Collect test results
      if: always()
      run: |
        mkdir -p test-results
        cp -r target/debug/deps/test-*.xml test-results/
    
    - name: Upload test results
      if: always()
      uses: actions/upload-artifact@v3
      with:
        name: test-results
        path: test-results
```

### 6.2 Test Reporting

Comprehensive test reporting dashboard with:

- Test execution history and trends
- Code coverage metrics and visualization
- Performance test results with graphs
- Security scan findings
- Integration with CI/CD pipeline

Example test report structure:

```
Test Summary:
- Total Tests: 437
- Passed: 432 (98.9%)
- Failed: 5 (1.1%)
- Skipped: 12

Code Coverage:
- Overall: 87.3%
- API Client: 92.1%
- Message Translator: 89.7%
- Authentication: 95.3%
- Error Handling: 83.2%

Performance Metrics:
- Average Response Time: 127ms
- 95th Percentile: 342ms
- Max Throughput: 215 req/sec
- Error Rate under Load: 0.3%

Security Scan:
- Critical Issues: 0
- High Issues: 0
- Medium Issues: 2
- Low Issues: 7
```

## 7. Test Driven Development

Guidelines for implementing TDD workflow:

1. **Write Tests First**
   - Create test cases before implementing features
   - Focus on behavior specification
   - Consider edge cases and error conditions

2. **Implementation Cycle**
   - Implement minimal code to pass tests
   - Refactor for clean design
   - Continuously run tests during development

3. **Test-Case Structure**
   - Arrange: Set up test preconditions
   - Act: Execute the system under test
   - Assert: Verify the expected outcome

Example TDD workflow:

```rust
// 1. Write test first
#[test]
fn test_tool_parameter_transformation() {
    // Arrange
    let galaxy_param = r#"{
        "name": "input_fastq",
        "type": "data",
        "format": "fastq",
        "optional": false
    }"#;
    
    // Act
    let transformer = ParameterTransformer::new();
    let mcp_param = transformer.transform(galaxy_param)?;
    
    // Assert
    assert_eq!(mcp_param.name, "input_fastq");
    assert_eq!(mcp_param.type, "file");
    assert_eq!(mcp_param.format, "fastq");
    assert_eq!(mcp_param.required, true);
}

// 2. Implement minimal code to pass
struct ParameterTransformer {}

impl ParameterTransformer {
    fn new() -> Self {
        Self {}
    }
    
    fn transform(&self, galaxy_param_json: &str) -> Result<McpParameter, TransformError> {
        let galaxy_param: GalaxyParameter = serde_json::from_str(galaxy_param_json)?;
        
        Ok(McpParameter {
            name: galaxy_param.name,
            type: map_type(galaxy_param.type),
            format: galaxy_param.format,
            required: !galaxy_param.optional,
        })
    }
}

// 3. Refactor
fn map_type(galaxy_type: &str) -> String {
    match galaxy_type {
        "data" => "file".to_string(),
        "integer" => "integer".to_string(),
        "float" => "float".to_string(),
        "boolean" => "boolean".to_string(),
        "text" => "string".to_string(),
        "select" => "enum".to_string(),
        _ => "string".to_string(),
    }
}
```

## 8. Test Quality Metrics

### 8.1 Coverage Targets

| Component | Line Coverage | Branch Coverage | Function Coverage |
|-----------|--------------|----------------|-------------------|
| Core Library | 90% | 85% | 95% |
| API Client | 85% | 80% | 90% |
| Message Processing | 90% | 85% | 95% |
| Integration Layer | 80% | 75% | 85% |
| CLI & Utilities | 70% | 65% | 80% |

### 8.2 Test Quality Checklist

- [ ] Tests are deterministic (no flaky tests)
- [ ] Tests are independent of each other
- [ ] Tests verify a single behavior or condition
- [ ] Tests have clear arrange-act-assert structure
- [ ] Tests have descriptive names and documentation
- [ ] Tests include positive and negative cases
- [ ] Tests cover edge cases and boundary conditions
- [ ] Tests are efficient and run quickly
- [ ] Tests clean up after themselves
- [ ] Test data is versioned and reproducible

### 8.3 Code Review Checklist for Tests

- [ ] Test logic matches specification requirements
- [ ] Tests are comprehensive and cover all code paths
- [ ] Test naming is clear and descriptive
- [ ] Test data is appropriate and realistic
- [ ] Mocks and stubs are used appropriately
- [ ] Tests follow project coding standards
- [ ] Tests are maintainable and not overly complex
- [ ] Assertions are specific and meaningful
- [ ] Error conditions and edge cases are tested
- [ ] Performance implications are considered

## 9. Testing Tools and Libraries

| Category | Purpose | Recommended Tools |
|----------|---------|-------------------|
| Unit Testing | Component testing | cargo test, rstest, mockito |
| Property Testing | Generative testing | proptest, quickcheck |
| Integration Testing | Component interaction | TestContainers, WireMock |
| API Testing | HTTP API validation | reqwest, surf, tokio |
| Performance Testing | Load and stress testing | k6, Gatling |
| Security Testing | Security validation | OWASP ZAP, cargo-audit |
| Coverage Analysis | Test coverage | tarpaulin, grcov |
| Mocking | Test isolation | mockall, mockito |
| Test Data | Data generation | Faker, custom generators |

## 10. Implementation Timeline

| Phase | Deliverables | Timeline |
|-------|-------------|----------|
| Setup | Test environment, CI integration | Week 1-2 |
| Unit Tests | Core component test suites | Week 3-4 |
| Integration Tests | Cross-component test suites | Week 5-6 |
| System Tests | End-to-end test workflows | Week 7-8 |
| Performance Tests | Load and stress test suites | Week 9-10 |
| Security Tests | Security validation suite | Week 11-12 |
| Documentation | Test documentation and reports | Week 13 |
| Handover | Training and knowledge transfer | Week 14 |

## 11. Related Specifications

- [Galaxy MCP Integration Plan](galaxy-mcp-integration.md)
- [API Mapping](api-mapping.md)
- [Tool Definition Schema](tool-definition-schema.md)
- [Security Model](security-model.md)
- [Configuration Management](configuration-management.md)
- [Deployment Guide](deployment-guide.md)

<version>0.1.0</version> 