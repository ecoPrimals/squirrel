# Project Setup Specification

## Repository Structure
```yaml
repository:
  root:
    - .github/            # GitHub specific configurations
      - workflows/        # GitHub Actions workflows
        - code-analysis.yml
        - integration-tests.yml
      - scripts/         # CI/CD scripts
        - generate_report.py
        - validate_specs.py
    
    - src/               # Source code
      - analysis/        # Code analysis tools
      - reporting/       # Report generation
      - mcp/            # MCP integration tools
      - web/            # Client portal
    
    - templates/         # Service templates
      - code_review/    # Code review templates
      - consulting/     # Consulting templates
      - reports/        # Report templates
    
    - tests/            # Test suites
      - unit/
      - integration/
      - e2e/
    
    - docs/             # Documentation
      - api/
      - guides/
      - examples/
```

## Development Environment Setup

### Required Tools
```yaml
tools:
  - git: "^2.25.0"
  - rust: "^1.70.0"
  - node: "^18.0.0"
  - python: "^3.10.0"
  - docker: "^24.0.0"
  - cargo-watch: "latest"
  - rust-analyzer: "latest"
```

### VS Code Extensions
```yaml
extensions:
  - rust-analyzer.rust-analyzer
  - serayuzgur.crates
  - vadimcn.vscode-lldb
  - tamasfe.even-better-toml
  - github.copilot
  - eamodio.gitlens
```

## Initial Infrastructure Setup

### GitHub Repository Configuration
```yaml
settings:
  default_branch: main
  merge_strategy: squash
  protection_rules:
    - require_reviews: true
    - require_status_checks: true
    - require_linear_history: true
  
  branch_protection:
    main:
      - require_pull_request
      - require_code_owner_reviews
      - dismiss_stale_reviews
```

### GitHub Actions Workflows

#### Code Analysis Pipeline
```yaml
triggers:
  - pull_request
  - push:
      branches: [main]
  - workflow_dispatch

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - code_checkout
      - setup_rust
      - setup_python
      - run_analysis
      - generate_report
      - upload_artifacts
```

#### Integration Tests
```yaml
triggers:
  - pull_request
  - push:
      branches: [main]
  - schedule:
      - cron: '0 0 * * *'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - code_checkout
      - setup_environment
      - run_tests
      - report_coverage
```

## Local Development Setup

### Environment Configuration
```yaml
env_vars:
  RUST_LOG: "debug"
  RUST_BACKTRACE: 1
  NODE_ENV: "development"
  DATABASE_URL: "postgresql://localhost:5432/income_dev"
```

### Development Scripts
```yaml
scripts:
  setup:
    - install_dependencies
    - setup_database
    - generate_configs
  
  watch:
    - cargo watch -x check -x test
    - cargo watch -x run
  
  test:
    - cargo test --all-features
    - cargo test --doc
```

## Initial Dependencies

### Rust Dependencies
```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
```

### Development Dependencies
```toml
[dev-dependencies]
tokio-test = "0.4"
mockall = "0.12"
criterion = "0.5"
pretty_assertions = "1.4"
```

## Documentation Setup

### API Documentation
```yaml
structure:
  - OpenAPI specification
  - Endpoint documentation
  - Authentication flows
  - Error responses
```

### Development Guides
```yaml
guides:
  - Setup guide
  - Development workflow
  - Testing strategy
  - Deployment process
```

## Initial Milestones

### Week 1
```yaml
goals:
  - Repository setup
  - CI/CD configuration
  - Development environment
  - Initial documentation

deliverables:
  - Working development environment
  - Automated CI/CD pipeline
  - Basic project structure
  - Setup documentation
```

### Week 2
```yaml
goals:
  - Core service setup
  - Testing framework
  - Initial API structure
  - Basic tooling

deliverables:
  - Working code analysis
  - Test infrastructure
  - API foundations
  - Development tools
```

## Security Configuration

### Repository Security
```yaml
security:
  - Dependabot alerts
  - Code scanning
  - Secret scanning
  - Security policy

branch_protection:
  - Signed commits required
  - Status checks required
  - Force push disabled
```

### Development Security
```yaml
practices:
  - Secure dependency management
  - Environment variable protection
  - Local security scanning
  - Code review guidelines
``` 