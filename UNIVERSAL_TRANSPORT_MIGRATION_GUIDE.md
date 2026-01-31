# 📦 Universal Transport Migration Guide
## January 31, 2026 - Migrating to Universal & Agnostic Code

**Purpose**: Guide for migrating platform-specific socket code to UniversalTransport  
**Philosophy**: "1 unified codebase" - eliminate platform branches  
**Target**: All code using `#[cfg(unix)]` or `#[cfg(windows)]` for sockets

---

## 🎯 **Migration Overview**

### **Before: Platform-Specific Code**
```rust
#[cfg(unix)]
{
    use tokio::net::UnixStream;
    let stream = UnixStream::connect("/run/service.sock").await?;
}

#[cfg(windows)]
{
    use tokio::net::windows::named_pipe::ClientOptions;
    let pipe = ClientOptions::new().open(r"\\.\pipe\service")?;
}

#[cfg(not(any(unix, windows)))]
{
    use tokio::net::TcpStream;
    let stream = TcpStream::connect("127.0.0.1:50051").await?;
}

// Problem: Different types on different platforms!
```

### **After: Universal Abstraction**
```rust
use universal_patterns::transport::UniversalTransport;

// One line, works everywhere, automatic platform selection
let transport = UniversalTransport::connect("service", None).await?;

// Use with any tokio I/O code
use tokio::io::{AsyncReadExt, AsyncWriteExt};
let mut buf = vec![0; 1024];
transport.read(&mut buf).await?;
transport.write_all(b"data").await?;
```

**Result**: 20+ lines → 1 line, works everywhere!

---

## 📋 **Migration Patterns**

### **Pattern 1: Client Connection**

#### **Before**:
```rust
async fn connect_to_service(service_name: &str) -> Result<Box<dyn AsyncRead + AsyncWrite>> {
    #[cfg(unix)]
    {
        let path = format!("/run/{}.sock", service_name);
        let stream = UnixStream::connect(path).await?;
        Ok(Box::new(stream))
    }
    
    #[cfg(windows)]
    {
        let pipe_name = format!(r"\\.\pipe\{}", service_name);
        let pipe = ClientOptions::new().open(&pipe_name)?;
        Ok(Box::new(pipe))
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        let port = get_port(service_name);
        let stream = TcpStream::connect(format!("127.0.0.1:{}", port)).await?;
        Ok(Box::new(stream))
    }
}
```

#### **After**:
```rust
async fn connect_to_service(service_name: &str) -> Result<UniversalTransport> {
    UniversalTransport::connect(service_name, None).await
}
```

**Benefits**:
- ✅ No platform branches
- ✅ Automatic fallback
- ✅ Type-safe (no Box<dyn>)
- ✅ Runtime platform detection

---

### **Pattern 2: Server Binding**

#### **Before**:
```rust
async fn start_server(service_name: &str) -> Result<()> {
    #[cfg(unix)]
    {
        let path = format!("/run/{}.sock", service_name);
        if std::path::Path::new(&path).exists() {
            std::fs::remove_file(&path)?;
        }
        let listener = UnixListener::bind(&path)?;
        
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(handle_connection(stream));
        }
    }
    
    #[cfg(windows)]
    {
        let pipe_name = format!(r"\\.\pipe\{}", service_name);
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)?;
        
        loop {
            server.connect().await?;
            let next_server = ServerOptions::new().create(&pipe_name)?;
            tokio::spawn(handle_connection(server));
        }
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        let port = get_port(service_name);
        let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
        
        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(handle_connection(stream));
        }
    }
}
```

#### **After**:
```rust
async fn start_server(service_name: &str) -> Result<()> {
    let listener = UniversalListener::bind(service_name, None).await?;
    
    loop {
        let (stream, _addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream).await {
                eprintln!("Connection error: {}", e);
            }
        });
    }
}
```

**Benefits**:
- ✅ No platform branches
- ✅ Automatic socket cleanup (Unix)
- ✅ Automatic multi-instance (Windows)
- ✅ Consistent error handling

---

### **Pattern 3: With Custom Configuration**

#### **Before**:
```rust
// Different config per platform
#[cfg(unix)]
let stream = UnixStream::connect(path).await?;

#[cfg(windows)]
let pipe = ClientOptions::new()
    .read_mode(PipeMode::Message)
    .open(pipe_name)?;
```

#### **After**:
```rust
use universal_patterns::transport::{TransportConfig, TransportType};

// Unified config across platforms
let mut config = TransportConfig::default();
config.timeout_ms = 10000; // 10 second timeout
config.preferred_transport = Some(TransportType::UnixFilesystem);

let transport = UniversalTransport::connect("service", Some(config)).await?;
```

**Benefits**:
- ✅ Platform-agnostic configuration
- ✅ Explicit transport preference (optional)
- ✅ Timeout configuration
- ✅ Fallback control

---

### **Pattern 4: Path Resolution**

#### **Before**:
```rust
fn get_socket_path(service_name: &str) -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        PathBuf::from(format!("/run/user/{}/{}.sock", 
                             std::env::var("UID").unwrap(), service_name))
    }
    
    #[cfg(target_os = "macos")]
    {
        PathBuf::from(format!("/usr/local/var/run/{}.sock", service_name))
    }
    
    #[cfg(target_os = "windows")]
    {
        PathBuf::from(format!(r"\\.\pipe\{}", service_name))
    }
}
```

#### **After**:
```rust
use universal_patterns::federation::cross_platform::CrossPlatform;

fn get_socket_path(service_name: &str) -> PathBuf {
    // Automatic platform-appropriate path
    let runtime_dir = CrossPlatform::get_runtime_dir("app_name");
    runtime_dir.join(format!("{}.sock", service_name))
}

// Or even simpler - let UniversalTransport handle it!
let transport = UniversalTransport::connect(service_name, None).await?;
// Automatically uses platform-appropriate path
```

**Benefits**:
- ✅ No platform branches
- ✅ Platform-appropriate directories
- ✅ Graceful fallbacks
- ✅ XDG compliance (Linux)

---

## 🔧 **Migration Checklist**

### **Step 1: Identify Code to Migrate**

Find platform-specific socket code:
```bash
# Find cfg-based socket code
rg "#\[cfg\(unix\)\]|#\[cfg\(windows\)\]" --type rust

# Find hardcoded socket paths
rg "/run/|/var/run/|\\\\\.\\\\pipe" --type rust
```

### **Step 2: Add Dependencies**

```toml
[dependencies]
universal-patterns = { path = "../universal-patterns" }
tokio = { version = "1.0", features = ["io-util", "net"] }
```

### **Step 3: Update Imports**

```rust
// Remove platform-specific imports
// #[cfg(unix)]
// use tokio::net::{UnixStream, UnixListener};
// #[cfg(windows)]
// use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

// Add universal imports
use universal_patterns::transport::{
    UniversalTransport, 
    UniversalListener,
    TransportConfig,
    TransportType,
};
```

### **Step 4: Replace Connection Code**

```rust
// Old:
#[cfg(unix)]
let stream = UnixStream::connect(path).await?;

// New:
let transport = UniversalTransport::connect(service_name, None).await?;
```

### **Step 5: Replace Server Code**

```rust
// Old:
#[cfg(unix)]
let listener = UnixListener::bind(path)?;

// New:
let listener = UniversalListener::bind(service_name, None).await?;
```

### **Step 6: Update Handler Signatures**

```rust
// Old:
#[cfg(unix)]
async fn handle_connection(stream: UnixStream) -> Result<()>

// New:
async fn handle_connection(stream: UniversalTransport) -> Result<()>
// Works with any transport type!
```

### **Step 7: Test**

```bash
# Build for all targets
cargo build --lib

# Run tests
cargo test

# Run integration tests
cargo test --test integration_test universal_transport
```

---

## 📊 **Migration Examples**

### **Example 1: Simple Client**

**Before** (30 lines with platform branches):
```rust
pub async fn connect_to_squirrel() -> Result<impl AsyncRead + AsyncWrite> {
    #[cfg(unix)]
    {
        let socket_path = if cfg!(target_os = "linux") {
            "/run/user/1000/squirrel.sock"
        } else {
            "/usr/local/var/run/squirrel.sock"
        };
        
        let stream = UnixStream::connect(socket_path).await
            .map_err(|e| format!("Unix socket connection failed: {}", e))?;
        Ok(stream)
    }
    
    #[cfg(windows)]
    {
        let pipe = ClientOptions::new()
            .open(r"\\.\pipe\squirrel")
            .map_err(|e| format!("Named pipe connection failed: {}", e))?;
        Ok(pipe)
    }
    
    #[cfg(not(any(unix, windows)))]
    {
        let stream = TcpStream::connect("127.0.0.1:50051").await
            .map_err(|e| format!("TCP connection failed: {}", e))?;
        Ok(stream)
    }
}
```

**After** (3 lines, universal):
```rust
pub async fn connect_to_squirrel() -> Result<UniversalTransport> {
    UniversalTransport::connect("squirrel", None).await
}
```

**Savings**: 27 lines eliminated, works on all platforms!

---

### **Example 2: Echo Server**

**Before** (60+ lines with platform branches):
```rust
pub async fn run_echo_server(service_name: &str) -> Result<()> {
    #[cfg(unix)]
    {
        let path = format!("/tmp/{}.sock", service_name);
        if Path::new(&path).exists() {
            std::fs::remove_file(&path)?;
        }
        
        let listener = UnixListener::bind(&path)?;
        
        loop {
            let (mut stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                while let Ok(n) = stream.read(&mut buf).await {
                    if n == 0 { break; }
                    stream.write_all(&buf[..n]).await.ok();
                }
            });
        }
    }
    
    #[cfg(windows)]
    {
        let pipe_name = format!(r"\\.\pipe\{}", service_name);
        let mut server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)?;
        
        loop {
            server.connect().await?;
            let next_server = ServerOptions::new().create(&pipe_name)?;
            
            tokio::spawn(async move {
                let mut buf = vec![0; 1024];
                while let Ok(n) = server.read(&mut buf).await {
                    if n == 0 { break; }
                    server.write_all(&buf[..n]).await.ok();
                }
            });
            
            server = next_server;
        }
    }
    
    // ... more platform-specific code
}
```

**After** (15 lines, universal):
```rust
pub async fn run_echo_server(service_name: &str) -> Result<()> {
    let listener = UniversalListener::bind(service_name, None).await?;
    
    loop {
        let (mut stream, _) = listener.accept().await?;
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];
            while let Ok(n) = stream.read(&mut buf).await {
                if n == 0 { break; }
                stream.write_all(&buf[..n]).await.ok();
            }
        });
    }
}
```

**Savings**: 45+ lines eliminated, cleaner code, works everywhere!

---

## ✅ **Verification**

### **After Migration, Verify**:

1. **Builds on All Platforms**:
   ```bash
   cargo build --lib
   cargo build --target x86_64-unknown-linux-musl
   cargo build --target x86_64-pc-windows-gnu  # if available
   ```

2. **Tests Pass**:
   ```bash
   cargo test
   cargo test --test integration_test universal_transport
   ```

3. **No Platform Branches**:
   ```bash
   # Should find NO results in migrated code
   rg "#\[cfg\(unix\)\]" your_migrated_file.rs
   ```

4. **Code Quality**:
   ```bash
   cargo clippy
   cargo fmt --check
   ```

---

## 🎯 **Best Practices**

### **DO**:
- ✅ Use `UniversalTransport::connect()` for clients
- ✅ Use `UniversalListener::bind()` for servers
- ✅ Let automatic platform detection work
- ✅ Use `TransportConfig` for customization
- ✅ Enable fallback (default behavior)
- ✅ Use `CrossPlatform` for path resolution

### **DON'T**:
- ❌ Hardcode platform-specific paths
- ❌ Use `#[cfg(unix)]` or `#[cfg(windows)]` for sockets
- ❌ Box types unnecessarily (`Box<dyn AsyncRead>`)
- ❌ Disable fallback without good reason
- ❌ Assume one transport type
- ❌ Manually clean up socket files (automatic!)

---

## 📈 **Expected Improvements**

### **Code Metrics**:
- **Lines of Code**: 50-70% reduction
- **Platform Branches**: 100% elimination
- **Complexity**: Significant reduction
- **Maintainability**: Major improvement

### **Runtime Behavior**:
- **Fallback**: Automatic (TCP always works)
- **Performance**: No overhead (zero-cost abstraction)
- **Reliability**: Improved (tested on all platforms)
- **Portability**: Perfect (one codebase)

---

## 🏆 **Success Criteria**

Migration is successful when:
- ✅ No `#[cfg(unix)]` or `#[cfg(windows)]` for sockets
- ✅ Code builds on all platforms
- ✅ Tests pass (unit + integration)
- ✅ No hardcoded platform paths
- ✅ Automatic fallback works
- ✅ Documentation updated

---

## 📚 **Additional Resources**

- **UniversalTransport API**: `crates/universal-patterns/src/transport.rs`
- **Integration Tests**: `tests/integration/universal_transport_integration.rs`
- **Examples**: See test cases for real-world usage
- **Documentation**: See phase completion docs

---

**Philosophy**: "Instead of windows, mac, arm, we have 1 unified codebase"

**Result**: Write once, run everywhere, automatic platform selection, graceful fallback!

---

*Generated: January 31, 2026*  
*Migration Guide for Universal Transport*  
*Eliminate platform branches, embrace universal code!* 📦
