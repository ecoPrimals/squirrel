use crate::security::{
    SecurityManagerImpl, SecurityManager, EnhancedRBACManager, PolicyManager,
    PolicyType, EnforcementLevel, SecurityPolicy, PolicyContext, PolicyEvaluationResult,
    PasswordPolicyEvaluator, RateLimitPolicyEvaluator, SessionPolicyEvaluator,
    Permission, Action, PermissionScope
};
use crate::types::{SecurityLevel, EncryptionFormat};
use crate::error::Result;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use chrono::Utc;
use tokio::time;

const BENCHMARK_ITERATIONS: usize = 1000;
const PERMISSION_COUNT: usize = 100;
const ROLE_COUNT: usize = 10;
const DATA_SIZE_KB: usize = 100;

/// Run a named benchmark and return the average operation time
async fn run_benchmark<F, Fut>(name: &str, iterations: usize, f: F) -> Duration 
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<()>>,
{
    println!("Starting benchmark: {}", name);
    
    let start = Instant::now();
    
    for i in 0..iterations {
        if i % (iterations / 10) == 0 && i > 0 {
            println!("  Progress: {}%", (i * 100) / iterations);
        }
        
        f().await.unwrap();
    }
    
    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations as u32;
    
    println!("Completed benchmark: {}", name);
    println!("  Total time: {:?}", total_duration);
    println!("  Average per operation: {:?}", avg_duration);
    println!("  Operations per second: {:.2}", 1.0 / avg_duration.as_secs_f64());
    
    avg_duration
}

#[tokio::test]
async fn benchmark_security_operations() -> Result<()> {
    // Create a security manager with all components for benchmarking
    let rbac_manager = Arc::new(EnhancedRBACManager::new());
    let policy_manager = Arc::new(PolicyManager::new(true));
    
    // Register policy evaluators
    policy_manager.add_evaluator(Arc::new(PasswordPolicyEvaluator::new())).await?;
    policy_manager.add_evaluator(Arc::new(RateLimitPolicyEvaluator::new())).await?;
    policy_manager.add_evaluator(Arc::new(SessionPolicyEvaluator::new())).await?;
    
    let security = SecurityManagerImpl::with_components(
        rbac_manager.clone(), 
        policy_manager.clone(),
        crate::security::encryption::create_encryption_manager()
    );
    
    // Setup test roles and permissions
    println!("Setting up test data for benchmarking...");
    
    // Create a large permission set
    let mut all_permissions = HashSet::new();
    for i in 0..PERMISSION_COUNT {
        let resource = format!("resource-{}", i % 10);
        let action = match i % 4 {
            0 => Action::Read,
            1 => Action::Write,
            2 => Action::Delete,
            _ => Action::Execute,
        };
        
        all_permissions.insert(Permission {
            id: format!("perm-{}", i),
            name: format!("Permission {}", i),
            resource: resource,
            action,
            resource_id: None,
            scope: PermissionScope::All,
            conditions: Vec::new(),
        });
    }
    
    // Create roles with various subsets of permissions
    for i in 0..ROLE_COUNT {
        let role_name = format!("role-{}", i);
        
        // Each role gets a subset of permissions
        let mut role_permissions = HashSet::new();
        for j in 0..PERMISSION_COUNT {
            if j % (i+1) == 0 {
                role_permissions.insert(all_permissions.iter().nth(j).unwrap().clone());
            }
        }
        
        let role = security.create_role(
            role_name.clone(),
            Some(format!("Role {}", i)),
            role_permissions,
            HashSet::new()
        ).await?;
        
        // Assign role to test user
        security.assign_role(format!("user-{}", i), role.id.clone()).await?;
    }
    
    // Add test policies
    let password_policy = SecurityPolicy {
        id: "benchmark-password-policy".to_string(),
        name: "Benchmark Password Policy".to_string(),
        description: Some("Password policy for benchmarking".to_string()),
        policy_type: PolicyType::Password,
        enforcement_level: EnforcementLevel::Enforced,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        settings: {
            let mut settings = HashMap::new();
            settings.insert("min_length".to_string(), "8".to_string());
            settings.insert("require_uppercase".to_string(), "true".to_string());
            settings.insert("require_lowercase".to_string(), "true".to_string());
            settings.insert("require_digit".to_string(), "true".to_string());
            settings
        },
        required_permissions: HashSet::new(),
        security_level: SecurityLevel::Standard,
        enabled: true,
    };
    
    security.add_policy(password_policy).await?;
    
    // Create sample data for encryption benchmarks - 100KB of JSON data
    let mut sample_data = serde_json::Map::new();
    sample_data.insert("id".to_string(), serde_json::Value::String("benchmark-data".to_string()));
    sample_data.insert("type".to_string(), serde_json::Value::String("performance-test".to_string()));
    
    // Add large arrays to increase the data size
    let mut array_data = Vec::new();
    for i in 0..(DATA_SIZE_KB * 10) {
        array_data.push(serde_json::Value::String(format!("Item {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nullam euismod, nisl eget aliquam ultricies.", i)));
    }
    
    sample_data.insert("data".to_string(), serde_json::Value::Array(array_data));
    let large_json_data = serde_json::Value::Object(sample_data);
    
    println!("Test data setup complete. Data size: {} bytes", serde_json::to_string(&large_json_data)?.len());
    
    // 1. Benchmark permission checks
    let permission_check_duration = run_benchmark(
        "Permission Checks",
        BENCHMARK_ITERATIONS,
        || {
            let security = security.clone();
            let i = rand::random::<usize>() % ROLE_COUNT;
            let j = rand::random::<usize>() % PERMISSION_COUNT;
            let permission = all_permissions.iter().nth(j).unwrap().clone();
            
            async move {
                let _ = security.has_permission(&format!("user-{}", i), &permission).await;
                Ok(())
            }
        }
    ).await;
    
    // 2. Benchmark policy evaluation
    let policy_evaluation_duration = run_benchmark(
        "Policy Evaluation",
        BENCHMARK_ITERATIONS,
        || {
            let security = security.clone();
            
            async move {
                let mut context = PolicyContext::default();
                let mut request_info = HashMap::new();
                
                // Randomly alternate between valid and invalid passwords
                if rand::random::<bool>() {
                    request_info.insert("password".to_string(), "Password123!".to_string());
                } else {
                    request_info.insert("password".to_string(), "weak".to_string());
                }
                
                context.request_info = request_info;
                
                let _ = security.evaluate_policy("benchmark-password-policy", &context).await;
                Ok(())
            }
        }
    ).await;
    
    // 3. Benchmark AES-256-GCM encryption
    let aes_encryption_duration = run_benchmark(
        "AES-256-GCM Encryption",
        BENCHMARK_ITERATIONS / 10, // Reduce iterations for large data
        || {
            let security = security.clone();
            let data = large_json_data.clone();
            
            async move {
                let _ = security.encrypt("benchmark-session", &data, Some(EncryptionFormat::Aes256Gcm)).await?;
                Ok(())
            }
        }
    ).await;
    
    // 4. Benchmark ChaCha20-Poly1305 encryption
    let chacha_encryption_duration = run_benchmark(
        "ChaCha20-Poly1305 Encryption",
        BENCHMARK_ITERATIONS / 10, // Reduce iterations for large data
        || {
            let security = security.clone();
            let data = large_json_data.clone();
            
            async move {
                let _ = security.encrypt("benchmark-session", &data, Some(EncryptionFormat::ChaCha20Poly1305)).await?;
                Ok(())
            }
        }
    ).await;
    
    // 5. Benchmark AES-256-GCM decryption
    // First encrypt some data
    let encrypted_aes = security.encrypt("benchmark-session", &large_json_data, Some(EncryptionFormat::Aes256Gcm)).await?;
    
    let aes_decryption_duration = run_benchmark(
        "AES-256-GCM Decryption",
        BENCHMARK_ITERATIONS / 10, // Reduce iterations for large data
        || {
            let security = security.clone();
            let encrypted = encrypted_aes.clone();
            
            async move {
                let _ = security.decrypt("benchmark-session", &encrypted, Some(EncryptionFormat::Aes256Gcm)).await?;
                Ok(())
            }
        }
    ).await;
    
    // 6. Benchmark role creation and assignment
    let role_management_duration = run_benchmark(
        "Role Management",
        BENCHMARK_ITERATIONS / 100, // Reduced iterations since this is a heavier operation
        || {
            let security = security.clone();
            let i = rand::random::<usize>() % 1000;
            
            async move {
                let mut permissions = HashSet::new();
                for j in 0..5 {
                    let perm_index = (i + j) % PERMISSION_COUNT;
                    permissions.insert(all_permissions.iter().nth(perm_index).unwrap().clone());
                }
                
                let role = security.create_role(
                    format!("benchmark-role-{}", i),
                    Some(format!("Benchmark Role {}", i)),
                    permissions,
                    HashSet::new()
                ).await?;
                
                security.assign_role(format!("benchmark-user-{}", i), role.id).await?;
                Ok(())
            }
        }
    ).await;
    
    // Calculate and report performance metrics
    println!("\nSecurity Performance Benchmark Summary:");
    println!("---------------------------------------------------------");
    println!("Operation                  | Duration (µs) | Ops/Second");
    println!("---------------------------------------------------------");
    println!("Permission Check           | {:13} | {:10.2}", 
             permission_check_duration.as_micros(), 
             1.0 / permission_check_duration.as_secs_f64());
    
    println!("Policy Evaluation          | {:13} | {:10.2}", 
             policy_evaluation_duration.as_micros(), 
             1.0 / policy_evaluation_duration.as_secs_f64());
    
    println!("AES-256-GCM Encryption     | {:13} | {:10.2}", 
             aes_encryption_duration.as_micros(), 
             1.0 / aes_encryption_duration.as_secs_f64());
    
    println!("ChaCha20-Poly1305 Encrypt  | {:13} | {:10.2}", 
             chacha_encryption_duration.as_micros(), 
             1.0 / chacha_encryption_duration.as_secs_f64());
    
    println!("AES-256-GCM Decryption     | {:13} | {:10.2}", 
             aes_decryption_duration.as_micros(), 
             1.0 / aes_decryption_duration.as_secs_f64());
    
    println!("Role Management            | {:13} | {:10.2}", 
             role_management_duration.as_micros(), 
             1.0 / role_management_duration.as_secs_f64());
    println!("---------------------------------------------------------");
    
    // Verify against performance targets
    let target_ops_per_sec = HashMap::from([
        ("Permission Check", 10000.0),
        ("Policy Evaluation", 5000.0),
        ("AES-256-GCM Encryption", 500.0),
        ("ChaCha20-Poly1305 Encryption", 400.0),
        ("AES-256-GCM Decryption", 500.0),
        ("Role Management", 100.0),
    ]);
    
    println!("\nPerformance Target Verification:");
    println!("---------------------------------------------------------");
    println!("Operation                  | Target Ops/s | Meets Target");
    println!("---------------------------------------------------------");
    
    let verify_target = |name: &str, duration: Duration| {
        let target = target_ops_per_sec.get(name).unwrap_or(&0.0);
        let actual = 1.0 / duration.as_secs_f64();
        let meets_target = actual >= *target;
        
        println!("{:26} | {:12.2} | {}", 
                 name, *target, 
                 if meets_target { "✅ YES" } else { "❌ NO" });
        
        meets_target
    };
    
    let mut all_targets_met = true;
    
    all_targets_met &= verify_target("Permission Check", permission_check_duration);
    all_targets_met &= verify_target("Policy Evaluation", policy_evaluation_duration);
    all_targets_met &= verify_target("AES-256-GCM Encryption", aes_encryption_duration);
    all_targets_met &= verify_target("ChaCha20-Poly1305 Encryption", chacha_encryption_duration);
    all_targets_met &= verify_target("AES-256-GCM Decryption", aes_decryption_duration);
    all_targets_met &= verify_target("Role Management", role_management_duration);
    
    println!("---------------------------------------------------------");
    println!("Overall Performance: {}", 
             if all_targets_met { "✅ MEETS ALL TARGETS" } else { "❌ FAILS SOME TARGETS" });
    
    Ok(())
} 