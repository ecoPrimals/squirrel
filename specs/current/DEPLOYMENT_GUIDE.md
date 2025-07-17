# Squirrel Universal AI Primal - Deployment Guide

**Date**: January 16, 2025  
**Status**: 🚀 **PRODUCTION READY**

---

## ✅ **Pre-Deployment Verification**

### **System Requirements**
- **Rust**: 1.70+ 
- **Operating System**: Linux, macOS, or Windows
- **Memory**: 4GB RAM minimum, 8GB recommended
- **CPU**: 2+ cores
- **Storage**: 10GB free space

### **Compilation Check**
```bash
cd code/crates
cargo check --all-features
# Should complete with exit code 0 (warnings are acceptable)
```

---

## 🚀 **Production Deployment**

### **Step 1: Build Release Binary**
```bash
cd code/crates
cargo build --release --all-features
```

### **Step 2: Environment Configuration**
```bash
# Set required environment variables
export SONGBIRD_ENDPOINT="http://your-service-mesh:8081"
export RUST_LOG="info"
export SERVICE_PORT="8080"
export SERVICE_HOST="0.0.0.0"
```

### **Step 3: Start the Service**
```bash
# Run the universal system
./target/release/squirrel
```

### **Step 4: Health Verification**
```bash
# Check system health
curl http://localhost:8080/health

# Check metrics
curl http://localhost:8080/metrics

# Check service discovery
curl http://localhost:8080/api/v1/ecosystem/status
```

---

## 🐳 **Docker Deployment**

### **Dockerfile**
```dockerfile
FROM rust:1.70 AS builder

WORKDIR /app
COPY . .
RUN cd code/crates && cargo build --release --all-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/code/crates/target/release/squirrel /usr/local/bin/
EXPOSE 8080
CMD ["squirrel"]
```

### **Build and Run**
```bash
# Build Docker image
docker build -t squirrel-universal:latest .

# Run container
docker run -p 8080:8080 \
    -e SONGBIRD_ENDPOINT="http://your-service-mesh:8081" \
    -e RUST_LOG="info" \
    squirrel-universal:latest
```

---

## ⚙️ **Configuration Options**

### **Environment Variables**
| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `SONGBIRD_ENDPOINT` | Service mesh endpoint | - | Yes |
| `SERVICE_PORT` | HTTP server port | 8080 | No |
| `SERVICE_HOST` | HTTP server host | 0.0.0.0 | No |
| `RUST_LOG` | Logging level | info | No |

### **Feature Flags**
| Flag | Description | Default |
|------|-------------|---------|
| `ecosystem` | Service mesh integration | Enabled |
| `monitoring` | Health and metrics | Enabled |
| `benchmarking` | Performance testing | Disabled |

---

## 🔍 **Health Monitoring**

### **Health Check Endpoints**
- **System Health**: `GET /health`
- **Detailed Health**: `GET /api/v1/health`
- **Metrics**: `GET /metrics`
- **Ecosystem Status**: `GET /api/v1/ecosystem/status`

### **Expected Responses**
```json
// GET /health
{
  "status": "healthy",
  "timestamp": "2025-01-16T10:00:00Z"
}

// GET /api/v1/ecosystem/status
{
  "status": "operational",
  "services": [],
  "overall_health": 100.0
}
```

---

## 🔧 **Troubleshooting**

### **Common Issues**

#### **Compilation Errors**
```bash
# If you encounter compilation errors
cargo clean
cargo build --release --all-features
```

#### **Service Mesh Connection**
```bash
# Check if Songbird endpoint is accessible
curl -I http://your-service-mesh:8081/health
```

#### **Port Conflicts**
```bash
# Use different port
export SERVICE_PORT="8081"
```

### **Log Analysis**
```bash
# View logs with debug level
RUST_LOG=debug ./target/release/squirrel

# View structured logs
RUST_LOG=debug ./target/release/squirrel | jq
```

---

## 📊 **Performance Monitoring**

### **Key Metrics**
- **Request latency**: < 100ms for health checks
- **Memory usage**: < 1GB under normal load
- **CPU usage**: < 50% under normal load
- **Service discovery**: < 5 seconds registration time

### **Monitoring Commands**
```bash
# System resources
htop

# Network connections
netstat -tlnp | grep 8080

# Application metrics
curl http://localhost:8080/metrics
```

---

## 🔐 **Security Considerations**

### **Network Security**
- Use HTTPS in production
- Implement proper firewall rules
- Restrict access to management endpoints

### **Authentication**
- Configure service mesh authentication
- Use proper API keys
- Implement rate limiting

---

## 🚀 **Scaling**

### **Horizontal Scaling**
```bash
# Run multiple instances
docker run -p 8080:8080 --name squirrel-1 squirrel-universal:latest
docker run -p 8081:8080 --name squirrel-2 squirrel-universal:latest
```

### **Load Balancing**
- Use nginx or HAProxy
- Configure health checks
- Implement proper load balancing algorithm

---

## ✅ **Deployment Checklist**

- [ ] System requirements met
- [ ] Compilation successful (exit code 0)
- [ ] Environment variables configured
- [ ] Service mesh endpoint accessible
- [ ] Health checks passing
- [ ] Metrics endpoint responding
- [ ] Logging configured
- [ ] Security measures implemented
- [ ] Monitoring set up
- [ ] Backup and recovery plan

---

## 📞 **Support**

For deployment issues:
1. Check logs with `RUST_LOG=debug`
2. Verify environment variables
3. Test service mesh connectivity
4. Review the troubleshooting section
5. Check GitHub issues for similar problems

---

**The Squirrel Universal AI Primal is production-ready and successfully deployed! 🚀** 