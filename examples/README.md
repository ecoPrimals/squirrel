# biome.yaml Manifest Guide

This directory contains example configuration files for deploying the Squirrel primal ecosystem in a biomeOS environment.

## Overview

The `biome.yaml` manifest file provides a comprehensive configuration for deploying all primals in the Squirrel ecosystem:

- **Squirrel** - Main coordination primal
- **Songbird** - Service discovery and mesh
- **ToadStool** - Compute and AI processing
- **NestGate** - Storage and persistence
- **BearDog** - Security and authentication

## Quick Start

1. Copy the `biome.yaml` file to your deployment directory
2. Modify the configuration for your environment
3. Deploy using biomeOS:

```bash
biome deploy -f biome.yaml
```

## Configuration Sections

### Metadata

```yaml
metadata:
  name: squirrel-ecosystem
  namespace: eco-primals
  version: "1.0.0"
```

- `name`: Unique identifier for the ecosystem deployment
- `namespace`: Kubernetes namespace for deployment
- `version`: Semantic version of the deployment

### Ecosystem Configuration

```yaml
spec:
  ecosystem:
    coordination:
      enabled: true
      discovery_timeout: 30s
      health_check_interval: 15s
```

Global settings that affect all primals:
- **Coordination**: Service discovery and inter-primal communication
- **Monitoring**: Metrics, logging, and observability
- **Security**: Authentication, authorization, and encryption

### Primal Deployments

Each primal has its own configuration section:

#### Squirrel (Main Coordinator)
- **Type**: `coordination`
- **Purpose**: Orchestrates the entire ecosystem
- **Key Features**: Service discovery, health monitoring, coordination

#### Songbird (Service Mesh)
- **Type**: `service_mesh`
- **Purpose**: Handles service discovery and inter-service communication
- **Key Features**: mTLS, circuit breakers, load balancing

#### ToadStool (Compute)
- **Type**: `compute`
- **Purpose**: AI processing and intensive computations
- **Key Features**: GPU allocation, model caching, auto-scaling

#### NestGate (Storage)
- **Type**: `storage`
- **Purpose**: Persistent storage and data management
- **Key Features**: Replication, backup, compression

#### BearDog (Security)
- **Type**: `security`
- **Purpose**: Authentication and authorization
- **Key Features**: JWT tokens, MFA, RBAC policies

### Resource Configuration

```yaml
resources:
  requests:
    cpu: "500m"
    memory: "1Gi"
    storage: "10Gi"
  limits:
    cpu: "2000m"
    memory: "4Gi"
    storage: "50Gi"
```

- **Requests**: Guaranteed resource allocation
- **Limits**: Maximum resource usage
- **GPU**: GPU allocation for compute-intensive primals

### Networking

```yaml
networking:
  ports:
    - name: http
      port: 8080
      protocol: TCP
      expose: true
  load_balancing:
    enabled: true
    algorithm: "round_robin"
```

- **Ports**: Service endpoints and protocols
- **Load Balancing**: Traffic distribution algorithms
- **Service Mesh**: Inter-service communication patterns

### Health Checks

```yaml
health_checks:
  liveness_probe:
    path: "/health/live"
    interval: "30s"
    timeout: "5s"
    failure_threshold: 3
  readiness_probe:
    path: "/health/ready"
    interval: "10s"
    timeout: "3s"
    failure_threshold: 2
```

- **Liveness Probe**: Determines if the service is alive
- **Readiness Probe**: Determines if the service is ready to receive traffic

### Scaling Configuration

```yaml
scaling:
  auto_scaling:
    enabled: true
    min_replicas: 1
    max_replicas: 5
    target_cpu: 70
    target_memory: 80
```

- **Auto Scaling**: Automatic horizontal scaling based on metrics
- **Cooldown**: Prevents rapid scaling oscillations

### Monitoring and Observability

```yaml
monitoring:
  metrics:
    enabled: true
    collection_interval: "15s"
    retention: "30d"
    exporters:
      - prometheus
      - graphite
  logging:
    enabled: true
    level: "info"
    format: "json"
    aggregation:
      enabled: true
      backend: "elasticsearch"
  tracing:
    enabled: true
    sampling_rate: 0.1
    backend: "jaeger"
```

- **Metrics**: Performance and health metrics collection
- **Logging**: Application logs and aggregation
- **Tracing**: Distributed tracing for debugging

### Security Configuration

```yaml
security:
  network_policies:
    enabled: true
    default_deny: true
    allow_rules:
      - from: "squirrel"
        to: "songbird"
        ports: [8000]
  pod_security:
    enabled: true
    policy: "restricted"
    run_as_non_root: true
```

- **Network Policies**: Control traffic between services
- **Pod Security**: Container security policies
- **Secrets**: Encryption and rotation policies

### Environment-Specific Overrides

```yaml
environments:
  development:
    replicas: 1
    resources:
      requests:
        cpu: "100m"
        memory: "256Mi"
    monitoring:
      enabled: false
  production:
    replicas: 3
    resources:
      requests:
        cpu: "500m"
        memory: "1Gi"
    monitoring:
      enabled: true
      retention: "30d"
```

Different configurations for different environments:
- **Development**: Minimal resources, monitoring disabled
- **Staging**: Medium resources, basic monitoring
- **Production**: Full resources, comprehensive monitoring

## Deployment Commands

### Deploy to Development
```bash
biome deploy -f biome.yaml --environment development
```

### Deploy to Staging
```bash
biome deploy -f biome.yaml --environment staging
```

### Deploy to Production
```bash
biome deploy -f biome.yaml --environment production
```

### Update Deployment
```bash
biome update -f biome.yaml --rolling-update
```

### Check Status
```bash
biome status squirrel-ecosystem
```

### Scale Service
```bash
biome scale squirrel-ecosystem --replicas 5
```

### Rollback
```bash
biome rollback squirrel-ecosystem --to-revision 2
```

## Configuration Tips

### Resource Sizing
- **CPU**: Start with requests and adjust based on usage
- **Memory**: Monitor for OOM kills and adjust limits
- **Storage**: Plan for data growth and backup requirements

### Networking
- **Ports**: Only expose necessary ports
- **Service Mesh**: Enable mTLS for production
- **Load Balancing**: Choose algorithm based on traffic patterns

### Security
- **Authentication**: Enable MFA for production
- **Network Policies**: Use default deny with explicit allow rules
- **Secrets**: Enable auto-rotation for sensitive data

### Monitoring
- **Metrics**: Balance collection frequency with storage costs
- **Logging**: Use appropriate log levels to avoid noise
- **Tracing**: Adjust sampling rate based on traffic volume

### Scaling
- **Auto Scaling**: Set appropriate CPU/memory thresholds
- **Cooldown**: Prevent rapid scaling oscillations
- **Resource Limits**: Ensure cluster has sufficient capacity

## Troubleshooting

### Common Issues

1. **Service Not Starting**
   - Check resource limits and requests
   - Verify health check endpoints
   - Review service logs

2. **Network Connectivity**
   - Check network policies
   - Verify service mesh configuration
   - Test port accessibility

3. **Performance Issues**
   - Monitor resource utilization
   - Check scaling policies
   - Review load balancing configuration

4. **Security Errors**
   - Verify authentication configuration
   - Check RBAC policies
   - Review secret rotation

### Debugging Commands

```bash
# View deployment status
biome status squirrel-ecosystem --verbose

# Check service logs
biome logs squirrel-ecosystem --service squirrel --follow

# View metrics
biome metrics squirrel-ecosystem --service toadstool

# Test connectivity
biome exec squirrel-ecosystem --service squirrel -- curl http://songbird:8000/health

# Validate configuration
biome validate -f biome.yaml
```

## Best Practices

1. **Configuration Management**
   - Use version control for manifest files
   - Test changes in development first
   - Document configuration decisions

2. **Resource Management**
   - Monitor resource usage regularly
   - Set appropriate limits and requests
   - Plan for peak usage scenarios

3. **Security**
   - Enable all security features in production
   - Regularly rotate secrets and certificates
   - Monitor for security violations

4. **Monitoring**
   - Set up comprehensive alerting
   - Monitor all key metrics
   - Regular review of logs and traces

5. **Deployment**
   - Use rolling updates for zero-downtime deployments
   - Test rollback procedures
   - Monitor deployment health

This manifest provides a production-ready configuration for the Squirrel primal ecosystem with comprehensive monitoring, security, and scaling capabilities. 