// Enhanced role inheritance for RBAC system
//
// This module provides advanced role inheritance capabilities for the RBAC system,
// including hierarchical inheritance, conditional inheritance, and delegation.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc, Timelike};

use crate::error::{MCPError, Result, SecurityError};
use crate::security::rbac::{Permission, Role, PermissionContext};

/// Inheritance relationship type between roles
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InheritanceType {
    /// Direct inheritance (child inherits all permissions from parent)
    Direct,
    
    /// Filtered inheritance (child inherits specific permissions)
    Filtered {
        /// Explicitly included permissions
        include: HashSet<String>,
        
        /// Explicitly excluded permissions
        exclude: HashSet<String>,
    },
    
    /// Conditional inheritance (inheritance depends on context)
    Conditional {
        /// Condition expression
        condition: String,
    },
    
    /// Delegated inheritance (temporarily granted)
    Delegated {
        /// Delegator ID
        delegator_id: String,
        
        /// Expiration time, if any
        expires_at: Option<DateTime<Utc>>,
    },
}

impl fmt::Display for InheritanceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Direct => write!(f, "Direct"),
            Self::Filtered { .. } => write!(f, "Filtered"),
            Self::Conditional { .. } => write!(f, "Conditional"),
            Self::Delegated { .. } => write!(f, "Delegated"),
        }
    }
}

/// Role inheritance node in the inheritance graph
#[derive(Debug, Clone)]
pub(super) struct InheritanceNode {
    /// Role ID
    pub role_id: String,
    
    /// Parent roles
    pub parents: HashMap<String, InheritanceType>,
    
    /// Child roles
    pub children: HashMap<String, InheritanceType>,
    
    /// Inheritance depth (0 for root roles)
    pub depth: u32,
}

/// Role inheritance graph
#[derive(Debug)]
pub(super) struct InheritanceGraph {
    /// Map of role IDs to inheritance nodes
    nodes: HashMap<String, InheritanceNode>,
}

impl InheritanceGraph {
    /// Create a new inheritance graph
    pub(super) fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
    
    /// Add a role to the graph
    pub(super) fn add_role(&mut self, role_id: &str) {
        if self.nodes.contains_key(role_id) {
            return;
        }
        
        self.nodes.insert(
            role_id.to_string(),
            InheritanceNode {
                role_id: role_id.to_string(),
                parents: HashMap::new(),
                children: HashMap::new(),
                depth: 0,
            },
        );
    }
    
    /// Add an inheritance relationship
    pub(super) fn add_inheritance(
        &mut self,
        parent_id: &str,
        child_id: &str,
        inheritance_type: InheritanceType,
    ) -> Result<()> {
        // Ensure both roles exist
        self.add_role(parent_id);
        self.add_role(child_id);
        
        // Check for cycles
        if self.would_create_cycle(parent_id, child_id) {
            return Err(MCPError::Security(SecurityError::RBACError(
                format!("Adding inheritance from {parent_id} to {child_id} would create a cycle")
            )));
        }
        
        // Add parent-child relationship
        if let Some(child_node) = self.nodes.get_mut(child_id) {
            child_node.parents.insert(parent_id.to_string(), inheritance_type.clone());
        }
        
        if let Some(parent_node) = self.nodes.get_mut(parent_id) {
            parent_node.children.insert(child_id.to_string(), inheritance_type);
        }
        
        // Update inheritance depths
        self.update_depths();
        
        Ok(())
    }
    
    /// Remove an inheritance relationship
    pub(super) fn remove_inheritance(&mut self, parent_id: &str, child_id: &str) {
        // Remove parent from child's parents
        if let Some(child_node) = self.nodes.get_mut(child_id) {
            child_node.parents.remove(parent_id);
        }
        
        // Remove child from parent's children
        if let Some(parent_node) = self.nodes.get_mut(parent_id) {
            parent_node.children.remove(child_id);
        }
        
        // Update inheritance depths
        self.update_depths();
    }
    
    /// Update inheritance depths for all nodes
    fn update_depths(&mut self) {
        // Reset all depths
        for node in self.nodes.values_mut() {
            node.depth = 0;
        }
        
        // Find root nodes (no parents)
        let root_ids: Vec<String> = self.nodes
            .values()
            .filter(|node| node.parents.is_empty())
            .map(|node| node.role_id.clone())
            .collect();
        
        // BFS to update depths
        for root_id in &root_ids {
            let mut queue = VecDeque::new();
            queue.push_back((root_id.clone(), 0));
            
            while let Some((node_id, depth)) = queue.pop_front() {
                // Update node depth
                if let Some(node) = self.nodes.get_mut(&node_id) {
                    node.depth = depth;
                    
                    // Enqueue children
                    for child_id in node.children.keys() {
                        queue.push_back((child_id.clone(), depth + 1));
                    }
                }
            }
        }
    }
    
    /// Check if adding an inheritance relationship would create a cycle
    fn would_create_cycle(&self, parent_id: &str, child_id: &str) -> bool {
        // If parent and child are the same, it's a cycle
        if parent_id == child_id {
            return true;
        }
        
        // Check if child is already an ancestor of parent
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(child_id.to_string());
        
        while let Some(current_id) = queue.pop_front() {
            if current_id == parent_id {
                return true;
            }
            
            if let Some(current_node) = self.nodes.get(&current_id) {
                for child_id in current_node.children.keys() {
                    if !visited.contains(child_id) {
                        visited.insert(child_id.clone());
                        queue.push_back(child_id.clone());
                    }
                }
            }
        }
        
        false
    }
    
    /// Get all ancestors of a role (parent roles)
    pub(super) fn get_ancestors(&self, role_id: &str) -> HashSet<String> {
        let mut ancestors = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with direct parents
        if let Some(node) = self.nodes.get(role_id) {
            for parent_id in node.parents.keys() {
                visited.insert(parent_id.clone());
                queue.push_back(parent_id.clone());
            }
        }
        
        // BFS to find all ancestors
        while let Some(current_id) = queue.pop_front() {
            ancestors.insert(current_id.clone());
            
            if let Some(current_node) = self.nodes.get(&current_id) {
                for parent_id in current_node.parents.keys() {
                    if !visited.contains(parent_id) {
                        visited.insert(parent_id.clone());
                        queue.push_back(parent_id.clone());
                    }
                }
            }
        }
        
        ancestors
    }
    
    /// Get all descendants of a role (child roles)
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to get descendants for
    ///
    /// # Returns
    /// A `HashSet` containing the IDs of all descendant roles
    pub(super) fn get_descendants(&self, role_id: &str) -> HashSet<String> {
        let mut descendants = HashSet::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        
        // Start with direct children
        if let Some(node) = self.nodes.get(role_id) {
            for child_id in node.children.keys() {
                visited.insert(child_id.clone());
                queue.push_back(child_id.clone());
            }
        }
        
        // BFS to find all descendants
        while let Some(current_id) = queue.pop_front() {
            descendants.insert(current_id.clone());
            
            if let Some(current_node) = self.nodes.get(&current_id) {
                for child_id in current_node.children.keys() {
                    if !visited.contains(child_id) {
                        visited.insert(child_id.clone());
                        queue.push_back(child_id.clone());
                    }
                }
            }
        }
        
        descendants
    }
    
    /// Check if a role inherits from another role
    pub(super) fn inherits_from(&self, child_id: &str, parent_id: &str) -> bool {
        let ancestors = self.get_ancestors(child_id);
        ancestors.contains(parent_id)
    }
    
    /// Get the inheritance relationship between parent and child
    pub(super) fn get_inheritance_type(
        &self,
        parent_id: &str,
        child_id: &str,
    ) -> Option<InheritanceType> {
        self.nodes.get(child_id).and_then(|child_node| child_node.parents.get(parent_id).cloned())
    }
    
    /// Get all permissions inherited by a role
    ///
    /// Retrieves all permissions that a role inherits from its ancestor roles,
    /// considering inheritance types and conditions.
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to get inherited permissions for
    /// * `role_map` - Map of role IDs to Role objects
    /// * `context` - Optional permission context for evaluating conditional inheritance
    ///
    /// # Returns
    /// A `HashSet` containing all inherited permissions
    pub(super) fn get_inherited_permissions(
        &self,
        role_id: &str,
        role_map: &HashMap<String, Role>,
        context: Option<&PermissionContext>,
    ) -> HashSet<Permission> {
        let mut permissions = HashSet::new();
        let ancestors = self.get_ancestors(role_id);
        
        for ancestor_id in ancestors {
            role_map.get(&ancestor_id).map(|ancestor_role| {
                let inheritance_type = self.get_inheritance_type(&ancestor_id, role_id);
                
                match inheritance_type {
                    Some(InheritanceType::Direct) => {
                        // Direct inheritance: include all permissions
                        permissions.extend(ancestor_role.permissions.clone());
                    }
                    
                    Some(InheritanceType::Filtered {
                        include,
                        exclude,
                    }) => {
                        // Filtered inheritance: include only specified permissions
                        for permission in &ancestor_role.permissions {
                            if (include.is_empty() || include.contains(&permission.id)) &&
                               !exclude.contains(&permission.id) {
                                permissions.insert(permission.clone());
                            }
                        }
                    }
                    
                    Some(InheritanceType::Conditional { condition }) => {
                        // Conditional inheritance: evaluate condition
                        if let Some(ctx) = context {
                            if Self::evaluate_condition(&condition, ctx) {
                                permissions.extend(ancestor_role.permissions.clone());
                            }
                        }
                    }
                    
                    Some(InheritanceType::Delegated { expires_at, .. }) => {
                        // Delegated inheritance: check expiration
                        let now = Utc::now();
                        if expires_at.map_or(true, |exp_time| exp_time > now) {
                            permissions.extend(ancestor_role.permissions.clone());
                        }
                    }
                    
                    None => {
                        // No direct inheritance, but ancestor is in the ancestry graph
                        // This means there's an indirect inheritance
                        permissions.extend(ancestor_role.permissions.clone());
                    }
                }
            });
        }
        
        permissions
    }
    
    /// Evaluate a condition expression
    fn evaluate_condition(condition: &str, context: &PermissionContext) -> bool {
        // Simple condition evaluation based on context attributes
        // In a real implementation, this would use a proper expression evaluator
        
        // Handle time-based conditions like "time_between(9:00,17:00)"
        if condition.starts_with("time_between(") {
            if let Some(time) = context.current_time {
                let parts: Vec<&str> = condition
                    .trim_start_matches("time_between(")
                    .trim_end_matches(')')
                    .split(',')
                    .collect();
                
                if parts.len() == 2 {
                    // Basic parsing of time ranges
                    // In a real implementation, use a proper time parsing library
                    let start_parts: Vec<&str> = parts[0].split(':').collect();
                    let end_parts: Vec<&str> = parts[1].split(':').collect();
                    
                    if start_parts.len() == 2 && end_parts.len() == 2 {
                        let current_hour = time.hour();
                        let current_minute = time.minute();
                        
                        let start_hour: u32 = start_parts[0].parse().unwrap_or(0);
                        let start_minute: u32 = start_parts[1].parse().unwrap_or(0);
                        let end_hour: u32 = end_parts[0].parse().unwrap_or(0);
                        let end_minute: u32 = end_parts[1].parse().unwrap_or(0);
                        
                        let current_mins = current_hour * 60 + current_minute;
                        let start_mins = start_hour * 60 + start_minute;
                        let end_mins = end_hour * 60 + end_minute;
                        
                        return current_mins >= start_mins && current_mins <= end_mins;
                    }
                }
            }
            return false;
        }
        
        // Handle attribute-based conditions like "attribute(department)=Engineering"
        if condition.starts_with("attribute(") {
            let parts: Vec<&str> = condition.split('=').collect();
            if parts.len() == 2 {
                let attr_name = parts[0]
                    .trim_start_matches("attribute(")
                    .trim_end_matches(')')
                    .trim();
                
                let attr_value = parts[1].trim();
                
                if let Some(actual_value) = context.attributes.get(attr_name) {
                    return actual_value == attr_value;
                }
            }
            return false;
        }
        
        // Default to false for unknown conditions
        false
    }
    
    /// Get a visual representation of the inheritance graph
    pub(super) fn to_dot(&self) -> String {
        let mut dot = String::from("digraph RoleInheritance {\n");
        
        // Add nodes
        for node in self.nodes.values() {
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\nDepth: {}\"];\n",
                node.role_id, node.role_id, node.depth
            ));
        }
        
        // Add edges
        for node in self.nodes.values() {
            for (parent_id, inheritance_type) in &node.parents {
                dot.push_str(&format!(
                    "  \"{}\" -> \"{}\" [label=\"{}\"];\n",
                    parent_id, node.role_id, inheritance_type
                ));
            }
        }
        
        dot.push_str("}\n");
        dot
    }
    
    /// Get all roles that a role inherits from (directly or indirectly)
    pub(super) fn get_inherited_roles(&self, role_id: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        
        self.dfs_parents(role_id, &mut result, &mut visited);
        
        result
    }
    
    /// Performs a depth-first search to find all parent roles recursively
    /// 
    /// # Arguments
    /// 
    /// * `role_id` - The role ID to start the search from
    /// * `result` - Set to collect all parent role IDs
    /// * `visited` - Set to track visited roles to avoid cycles
    fn dfs_parents(&self, role_id: &str, result: &mut HashSet<String>, visited: &mut HashSet<String>) {
        if !visited.contains(role_id) {
            visited.insert(role_id.to_string());
            
            if let Some(node) = self.nodes.get(role_id) {
                for parent_id in node.parents.keys() {
                    result.insert(parent_id.clone());
                    self.dfs_parents(parent_id, result, visited);
                }
            }
        }
    }
    
    /// Check if a role exists in the graph
    pub(super) fn has_role(&self, role_id: &str) -> bool {
        self.nodes.contains_key(role_id)
    }
    
    /// Check if there is a direct inheritance relationship between parent and child
    pub(super) fn has_inheritance(&self, parent_id: &str, child_id: &str) -> bool {
        self.nodes.get(child_id).map_or(false, |child_node| child_node.parents.contains_key(parent_id))
    }

    /// Check if a role has a direct inheritance relationship with another role
    pub(super) fn has_direct_inheritance(&self, child_id: &str, parent_id: &str) -> bool {
        self.nodes.get(child_id).map_or(false, |child_node| 
            child_node.parents.contains_key(parent_id)
        )
    }

    /// Get all roles that inherit from a role (directly or indirectly)
    pub(super) fn get_inherited_by_roles(&self, role_id: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut visited = HashSet::new();
        
        self.dfs_children(role_id, &mut result, &mut visited);
        
        result
    }

    /// Perform depth-first search to find all children of a role
    fn dfs_children(&self, role_id: &str, result: &mut HashSet<String>, visited: &mut HashSet<String>) {
        if !visited.insert(role_id.to_string()) {
            return;
        }

        if let Some(node) = self.nodes.get(role_id) {
            for child_id in node.children.keys() {
                result.insert(child_id.to_string());
                self.dfs_children(child_id, result, visited);
            }
        }
    }
}

/// Thread-safe inheritance graph manager
#[derive(Debug)]
pub struct InheritanceManager {
    /// Inheritance graph
    graph: RwLock<InheritanceGraph>,
}

impl Default for InheritanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InheritanceManager {
    /// Create a new inheritance manager
    #[must_use] pub fn new() -> Self {
        Self {
            graph: RwLock::new(InheritanceGraph::new()),
        }
    }
    
    /// Add a role to the inheritance graph
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to add
    ///
    /// # Returns
    /// `Ok(())` if the role was successfully added
    ///
    /// # Errors
    /// This function will return an error if:
    /// * An internal graph operation fails
    /// * There is a failure in acquiring the write lock
    pub async fn add_role(&self, role_id: &str) -> Result<()> {
        self.graph.write().await.add_role(role_id);
        Ok(())
    }
    
    /// Add a direct inheritance relationship
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    ///
    /// # Returns
    /// `Ok(())` if the inheritance relationship was successfully created
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The parent role does not exist
    /// * The child role does not exist
    /// * Adding this relationship would create a cycle in the inheritance graph
    /// * There is a failure in acquiring the write lock
    pub async fn add_direct_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
    ) -> Result<()> {
        self.graph.write().await.add_inheritance(parent_id, child_id, InheritanceType::Direct)?;
        Ok(())
    }
    
    /// Add a filtered inheritance relationship
    ///
    /// Creates an inheritance relationship where the child role only inherits
    /// specific permissions from the parent role, controlled by inclusion and
    /// exclusion lists.
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    /// * `include` - Set of permission IDs to include (empty means include all)
    /// * `exclude` - Set of permission IDs to exclude
    ///
    /// # Returns
    /// `Ok(())` if the filtered inheritance relationship was successfully created
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The parent role does not exist
    /// * The child role does not exist
    /// * Adding this relationship would create a cycle in the inheritance graph
    /// * There is a failure in acquiring the write lock
    pub async fn add_filtered_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        include: HashSet<String>,
        exclude: HashSet<String>,
    ) -> Result<()> {
        self.graph.write().await.add_inheritance(
            parent_id,
            child_id,
            InheritanceType::Filtered {
                include,
                exclude,
            },
        )?;
        Ok(())
    }
    
    /// Add a conditional inheritance relationship
    ///
    /// Creates an inheritance relationship where the child role only inherits
    /// permissions from the parent role when a specific condition is met.
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    /// * `condition` - String expression representing the condition for inheritance
    ///
    /// # Returns
    /// `Ok(())` if the conditional inheritance relationship was successfully created
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The parent role does not exist
    /// * The child role does not exist
    /// * Adding this relationship would create a cycle in the inheritance graph
    /// * The condition string is invalid or malformed
    /// * There is a failure in acquiring the write lock
    pub async fn add_conditional_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        condition: String,
    ) -> Result<()> {
        self.graph.write().await.add_inheritance(
            parent_id,
            child_id,
            InheritanceType::Conditional {
                condition,
            },
        )?;
        Ok(())
    }
    
    /// Add a delegated inheritance relationship
    ///
    /// Creates a temporary inheritance relationship where the child role inherits
    /// permissions from the parent role through delegation by another user.
    /// This can be used for temporary access grants or specific delegation scenarios.
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    /// * `delegator_id` - ID of the user who delegated the permissions
    /// * `expires_at` - Optional expiration time for the delegation
    ///
    /// # Returns
    /// `Ok(())` if the delegated inheritance relationship was successfully created
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The parent role does not exist
    /// * The child role does not exist
    /// * The delegator does not have delegation permissions
    /// * Adding this relationship would create a cycle in the inheritance graph
    /// * There is a failure in acquiring the write lock
    pub async fn add_delegated_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
        delegator_id: String,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        self.graph.write().await.add_inheritance(
            parent_id,
            child_id,
            InheritanceType::Delegated {
                delegator_id,
                expires_at,
            },
        )?;
        Ok(())
    }
    
    /// Remove an inheritance relationship
    ///
    /// Removes an existing inheritance relationship between the parent and child roles.
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    ///
    /// # Returns
    /// `Ok(())` if the inheritance relationship was successfully removed
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The parent role does not exist
    /// * The child role does not exist
    /// * No inheritance relationship exists between the roles
    /// * There is a failure in acquiring the write lock
    pub async fn remove_inheritance(
        &self,
        parent_id: &str,
        child_id: &str,
    ) -> Result<()> {
        self.graph.write().await.remove_inheritance(parent_id, child_id);
        Ok(())
    }
    
    /// Get all ancestors of a role
    ///
    /// Retrieves all roles that the specified role inherits from, directly or indirectly.
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to get ancestors for
    ///
    /// # Returns
    /// A `HashSet` containing the IDs of all ancestor roles
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The role does not exist
    /// * The inheritance graph cannot be traversed
    /// * There is a failure in acquiring the read lock
    pub async fn get_ancestors(&self, role_id: &str) -> HashSet<String> {
        let graph = self.graph.read().await;
        graph.get_ancestors(role_id)
    }
    
    /// Get all descendants of a role
    ///
    /// Retrieves all roles that inherit from the specified role, directly or indirectly.
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to get descendants for
    ///
    /// # Returns
    /// A `HashSet` containing the IDs of all descendant roles
    ///
    /// # Errors
    /// This function will return an error if:
    /// * The role does not exist
    /// * The inheritance graph cannot be traversed
    /// * There is a failure in acquiring the read lock
    pub async fn get_descendants(&self, role_id: &str) -> HashSet<String> {
        let graph = self.graph.read().await;
        graph.get_descendants(role_id)
    }
    
    /// Check if a role inherits from another role
    ///
    /// Determines whether a child role inherits from a parent role,
    /// either directly or through inheritance chains.
    ///
    /// # Arguments
    /// * `child_id` - ID of the child role to check
    /// * `parent_id` - ID of the parent role to check
    ///
    /// # Returns
    /// `Ok(true)` if the child role inherits from the parent role, `Ok(false)` otherwise
    ///
    /// # Errors
    /// This function will return an error if:
    /// * Either the child or parent role does not exist
    /// * There is a failure in acquiring the read lock
    /// * The inheritance graph cannot be traversed
    pub async fn inherits_from(
        &self,
        child_id: &str,
        parent_id: &str,
    ) -> Result<bool> {
        let graph = self.graph.read().await;
        Ok(graph.inherits_from(child_id, parent_id))
    }
    
    /// Get inheritance diagram as DOT format
    pub async fn to_dot(&self) -> String {
        let graph = self.graph.read().await;
        graph.to_dot()
    }

    /// Get the inheritance type between two roles
    ///
    /// # Arguments
    /// * `parent_id` - ID of the parent role
    /// * `child_id` - ID of the child role
    ///
    /// # Returns
    /// The inheritance type if a relationship exists, None otherwise
    ///
    /// # Errors
    /// This function will return an error if there is a failure in acquiring the read lock
    pub async fn get_inheritance_type(&self, parent_id: &str, child_id: &str) -> Result<Option<InheritanceType>> {
        let graph = self.graph.read().await;
        Ok(graph.get_inheritance_type(parent_id, child_id))
    }

    /// Get all permissions inherited by a role
    ///
    /// # Arguments
    /// * `role_id` - ID of the role to get permissions for
    /// * `role_map` - Map of role IDs to roles
    /// * `context` - Optional permission context for evaluating conditional inheritance
    ///
    /// # Returns
    /// Set of inherited permissions
    ///
    /// # Errors
    /// This function will return an error if there is a failure in acquiring the read lock
    pub async fn get_inherited_permissions(
        &self,
        role_id: &str,
        role_map: &HashMap<String, Role>,
        context: Option<&PermissionContext>,
    ) -> Result<HashSet<Permission>> {
        let graph = self.graph.read().await;
        Ok(graph.get_inherited_permissions(role_id, role_map, context))
    }
}