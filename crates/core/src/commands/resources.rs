use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use std::time::{Duration, SystemTime};

#[derive(Debug)]
pub struct ResourceError {
    pub resource_type: String,
    pub message: String,
}

impl std::fmt::Display for ResourceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Resource error ({}): {}", self.resource_type, self.message)
    }
}

impl Error for ResourceError {}

#[derive(Debug, Clone)]
pub struct ResourceLimit {
    pub max_value: u64,
    pub current_value: u64,
    pub reset_interval: Option<Duration>,
    pub last_reset: SystemTime,
}

impl ResourceLimit {
    pub fn new(max_value: u64, reset_interval: Option<Duration>) -> Self {
        Self {
            max_value,
            current_value: 0,
            reset_interval,
            last_reset: SystemTime::now(),
        }
    }

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

pub struct ResourceManager {
    limits: RwLock<HashMap<String, ResourceLimit>>,
    allocations: RwLock<HashMap<String, HashMap<String, u64>>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            limits: RwLock::new(HashMap::new()),
            allocations: RwLock::new(HashMap::new()),
        }
    }

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

    pub fn allocate(
        &self,
        command_name: &str,
        resource_type: &str,
        amount: u64,
    ) -> Result<(), Box<dyn Error>> {
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
            .entry(command_name.to_string())
            .or_insert_with(HashMap::new);

        let current_allocation = command_allocations.entry(resource_type.to_string()).or_insert(0);
        *current_allocation += amount;

        Ok(())
    }

    pub fn deallocate(
        &self,
        command_name: &str,
        resource_type: &str,
        amount: u64,
    ) -> Result<(), Box<dyn Error>> {
        let mut allocations = self.allocations.write().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire write lock on allocations".to_string(),
            })
        })?;

        let command_allocations = allocations.get_mut(command_name).ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!("No allocations found for command {}", command_name),
            })
        })?;

        let current_allocation = command_allocations.get_mut(resource_type).ok_or_else(|| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!(
                    "No allocation found for resource type {} in command {}",
                    resource_type, command_name
                ),
            })
        })?;

        if *current_allocation < amount {
            return Err(Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: format!(
                    "Cannot deallocate {} units when only {} are allocated",
                    amount, current_allocation
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

    pub fn get_allocation(
        &self,
        command_name: &str,
        resource_type: &str,
    ) -> Result<u64, Box<dyn Error>> {
        let allocations = self.allocations.read().map_err(|_| {
            Box::new(ResourceError {
                resource_type: resource_type.to_string(),
                message: "Failed to acquire read lock on allocations".to_string(),
            })
        })?;

        Ok(*allocations
            .get(command_name)
            .and_then(|command_allocations| command_allocations.get(resource_type))
            .unwrap_or(&0))
    }

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

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
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
        assert!(manager.allocate("test_command", "memory", 50).is_ok());
        assert_eq!(manager.get_allocation("test_command", "memory").unwrap(), 50);
        
        // Test deallocation
        assert!(manager.deallocate("test_command", "memory", 30).is_ok());
        assert_eq!(manager.get_allocation("test_command", "memory").unwrap(), 20);
    }

    #[test]
    fn test_resource_limit_reset() {
        let mut limit = ResourceLimit::new(100, Some(Duration::from_secs(1)));
        assert!(limit.check_and_update(50).is_ok());
        std::thread::sleep(Duration::from_secs(2));
        assert!(limit.check_and_update(60).is_ok()); // Should reset and allow this
    }
} 