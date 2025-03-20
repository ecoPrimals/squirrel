use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

/// Error type for resource-related operations
#[derive(Debug)]
pub struct ResourceError {
    /// Type of resource that caused the error
    pub resource_type: String,
    /// Description of the error
    pub message: String,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource error ({}): {}", self.resource_type, self.message)
    }
}

impl Error for ResourceError {}

/// Represents a limit on a specific resource
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ResourceLimit {
    /// Maximum allowed value for the resource
    pub max_value: u64,
    /// Current value of the resource
    pub current_value: u64,
    /// Optional interval after which the resource limit resets
    pub reset_interval: Option<Duration>,
    /// Last time the resource limit was reset
    pub last_reset: SystemTime,
}

#[allow(dead_code)]
impl ResourceLimit {
    /// Creates a new resource limit.
    #[must_use]
    pub fn new(max_value: u64, reset_interval: Option<Duration>) -> Self {
        Self {
            max_value,
            current_value: 0,
            reset_interval,
            last_reset: SystemTime::now(),
        }
    }

    /// Checks if the requested amount can be allocated and updates the current usage.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The requested amount would exceed the maximum limit
    /// - The current usage would overflow when adding the requested amount
    pub fn check_and_update(&mut self, amount: u64) -> Result<(), Box<dyn Error>> {
        // Check if we need to reset
        if let Some(interval) = self.reset_interval {
            let now = SystemTime::now();
            if now.duration_since(self.last_reset).unwrap_or_default() >= interval {
                self.current_value = 0;
                self.last_reset = now;
            }
        }

        // Check if we have enough resources
        if self.current_value + amount > self.max_value {
            return Err(Box::new(ResourceError {
                resource_type: "limit".to_string(),
                message: format!(
                    "Resource limit exceeded: {} + {} > {}",
                    self.current_value, amount, self.max_value
                ),
            }));
        }

        self.current_value += amount;
        Ok(())
    }
}

/// Manager for tracking and limiting resource usage
#[allow(dead_code)]
pub struct ResourceManager {
    /// Map of resource types to their limits
    limits: RwLock<HashMap<String, ResourceLimit>>,
    /// Map of resource types to their current allocations
    allocations: RwLock<HashMap<String, HashMap<String, u64>>>,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    /// Creates a new resource manager
    #[must_use]
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            limits: RwLock::new(HashMap::new()),
            allocations: RwLock::new(HashMap::new()),
        }
    }

    /// Sets a limit for a resource type
    /// 
    /// # Arguments
    /// * `resource_type` - Type of resource (e.g., "memory", "threads")
    /// * `limit` - The limit to set
    /// 
    /// # Errors
    /// Returns an error if unable to acquire write lock
    #[allow(dead_code)]
    pub fn set_limit(&self, resource_type: &str, limit: ResourceLimit) -> Result<(), Box<dyn Error>> {
        let mut limits = self.limits.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on limits".to_string(),
            })
        })?;

        limits.insert(resource_type.to_string(), limit);
        Ok(())
    }

    /// Allocates resources of a given type
    /// 
    /// # Arguments
    /// * `resource_type` - Type of resource to allocate
    /// * `amount` - Amount to allocate
    /// * `owner` - Identifier for who owns the allocation
    /// 
    /// # Errors
    /// Returns an error if allocation would exceed limits
    #[allow(dead_code)]
    pub fn allocate(&self, resource_type: &str, amount: u64, owner: &str) -> Result<(), Box<dyn Error>> {
        // Update limit
        let mut limits = self.limits.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on limits".to_string(),
            })
        })?;

        let limit = limits.get_mut(resource_type).ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Resource type not found".to_string(),
            })
        })?;

        limit.check_and_update(amount)?;

        // Update allocation
        let mut allocations = self.allocations.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on allocations".to_string(),
            })
        })?;

        let command_allocations = allocations
            .entry(owner.to_string())
            .or_insert_with(HashMap::new);

        let current_allocation = command_allocations.entry(resource_type.to_string()).or_insert(0);
        *current_allocation += amount;

        Ok(())
    }

    /// Deallocates resources of a given type
    /// 
    /// # Arguments
    /// * `resource_type` - Type of resource to deallocate
    /// * `amount` - Amount to deallocate
    /// * `owner` - Identifier for who owns the allocation
    /// 
    /// # Errors
    /// Returns an error if deallocation would result in negative allocation
    #[allow(dead_code)]
    pub fn deallocate(&self, resource_type: &str, amount: u64, owner: &str) -> Result<(), Box<dyn Error>> {
        let mut allocations = self.allocations.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on allocations".to_string(),
            })
        })?;

        let command_allocations = allocations.get_mut(owner).ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!("No allocations found for command {owner}"),
            })
        })?;

        let current_allocation = command_allocations.get_mut(resource_type).ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!(
                    "No allocation found for resource type {resource_type} in command {owner}"
                ),
            })
        })?;

        if *current_allocation < amount {
            return Err(Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!(
                    "Cannot deallocate {amount} units when only {current_allocation} are allocated"
                ),
            }));
        }

        *current_allocation -= amount;

        // Update limit
        let mut limits = self.limits.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on limits".to_string(),
            })
        })?;

        if let Some(limit) = limits.get_mut(resource_type) {
            limit.current_value = limit.current_value.saturating_sub(amount);
        }

        Ok(())
    }

    /// Gets the current allocation for a resource type and owner
    /// 
    /// # Arguments
    /// * `resource_type` - Type of resource to check
    /// * `owner` - Identifier for who owns the allocation
    /// 
    /// # Returns
    /// The current allocation amount
    /// 
    /// # Errors
    /// Returns an error if unable to acquire read lock
    #[allow(dead_code)]
    pub fn get_allocation(&self, resource_type: &str, owner: &str) -> Result<u64, Box<dyn Error>> {
        let allocations = self.allocations.read().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire read lock on allocations".to_string(),
            })
        })?;

        Ok(*allocations
            .get(owner)
            .and_then(|command_allocations| command_allocations.get(resource_type))
            .unwrap_or(&0))
    }

    /// Gets the limit for a specific resource type
    /// 
    /// # Arguments
    /// * `resource_type` - The type of resource to get the limit for
    /// 
    /// # Returns
    /// The resource limit if found
    /// 
    /// # Errors
    /// Returns an error if the resource type is not found
    #[allow(dead_code)]
    pub fn get_limit(&self, resource_type: &str) -> Result<ResourceLimit, Box<dyn Error>> {
        let limits = self.limits.read().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire read lock on limits".to_string(),
            })
        })?;

        Ok(limits.get(resource_type).cloned().ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Resource type not found".to_string(),
            })
        })?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limit() {
        let mut limit = ResourceLimit::new(100, None);
        assert!(limit.check_and_update(50).is_ok());
        assert!(limit.check_and_update(40).is_ok());
        assert!(limit.check_and_update(20).is_err());
    }

    #[test]
    fn test_resource_allocation() {
        let manager = ResourceManager::new();
        
        // Set up a limit
        manager
            .set_limit("memory", ResourceLimit::new(100, None))
            .unwrap();

        // Test allocation
        assert!(manager.allocate("memory", 50, "test_command").is_ok());
        assert_eq!(manager.get_allocation("memory", "test_command").unwrap(), 50);
        
        // Test deallocation
        assert!(manager.deallocate("memory", 30, "test_command").is_ok());
        assert_eq!(manager.get_allocation("memory", "test_command").unwrap(), 20);
    }

    #[test]
    fn test_resource_limit_reset() {
        let mut limit = ResourceLimit::new(100, Some(Duration::from_secs(1)));
        assert!(limit.check_and_update(50).is_ok());
        std::thread::sleep(Duration::from_secs(2));
        assert!(limit.check_and_update(60).is_ok()); // Should reset and allow this
    }
} 