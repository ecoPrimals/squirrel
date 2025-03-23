//! Module defining Galaxy library data models
//! 
//! This module contains the data structures for representing Galaxy libraries,
//! library folders, and related objects.

use serde::{Serialize, Deserialize};

use crate::models::ResourceMetadata;

/// Represents a Galaxy data library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyLibrary {
    /// Common metadata for the library
    pub metadata: ResourceMetadata,
    
    /// A synopsis of this library
    pub synopsis: Option<String>,
    
    /// The root folder ID of this library
    pub root_folder_id: String,
    
    /// Whether this library is deleted
    pub deleted: bool,
    
    /// Whether this library is public
    pub public: bool,
    
    /// Library permissions
    pub permissions: Option<LibraryPermissions>,
    
    /// Creation time of this library
    pub create_time: String,
    
    /// Update time of this library
    pub update_time: String,
}

/// Represents a folder in a Galaxy library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFolder {
    /// The ID of this folder
    pub id: String,
    
    /// The name of this folder
    pub name: String,
    
    /// A description of this folder
    pub description: Option<String>,
    
    /// The ID of the parent folder
    pub parent_id: Option<String>,
    
    /// The library this folder belongs to
    pub library_id: String,
    
    /// Whether this folder is deleted
    pub deleted: bool,
    
    /// The model class for this folder
    pub model_class: String,
    
    /// The creation time of this folder
    pub create_time: String,
    
    /// The update time of this folder
    pub update_time: String,
    
    /// The genome build for datasets in this folder
    pub genome_build: Option<String>,
}

/// Represents the permissions for a library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPermissions {
    /// Users who can access this library
    pub access_library_users: Vec<String>,
    
    /// Roles that can access this library
    pub access_library_roles: Vec<String>,
    
    /// Users who can add items to this library
    pub add_library_item_users: Vec<String>,
    
    /// Roles that can add items to this library
    pub add_library_item_roles: Vec<String>,
    
    /// Users who can manage this library
    pub manage_library_users: Vec<String>,
    
    /// Roles that can manage this library
    pub manage_library_roles: Vec<String>,
    
    /// Users who can modify this library
    pub modify_library_users: Vec<String>,
    
    /// Roles that can modify this library
    pub modify_library_roles: Vec<String>,
}

/// Parameters for creating a new library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryCreateParams {
    /// The name for the new library
    pub name: String,
    
    /// A description for the library
    pub description: Option<String>,
    
    /// A synopsis for the library
    pub synopsis: Option<String>,
    
    /// Whether this library should be public
    pub public: bool,
}

/// Parameters for creating a new library folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderCreateParams {
    /// The name for the new folder
    pub name: String,
    
    /// A description for the folder
    pub description: Option<String>,
    
    /// The ID of the parent folder
    pub parent_id: String,
    
    /// The genome build for this folder
    pub genome_build: Option<String>,
}

/// Parameters for uploading a file to a library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryUploadParams {
    /// The folder ID to upload to
    pub folder_id: String,
    
    /// The file content to upload
    pub file_content: Vec<u8>,
    
    /// The name for the dataset
    pub file_name: String,
    
    /// The file type/extension
    pub file_type: String,
    
    /// A description for the dataset
    pub description: Option<String>,
    
    /// The genome build for this dataset
    pub dbkey: String,
    
    /// Whether to link to the file instead of copying
    pub link_data: bool,
    
    /// Tags to apply to the dataset
    pub tags: Option<Vec<String>>,
}

/// Parameters for searching libraries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySearchParams {
    /// Text to search for in library names and descriptions
    pub query: Option<String>,
    
    /// Include deleted libraries
    pub include_deleted: bool,
    
    /// Only include libraries the user can add to
    pub can_add_library_item: bool,
    
    /// Only include libraries the user can modify
    pub can_modify_library: bool,
    
    /// Only include libraries the user can manage
    pub can_manage_library: bool,
}

/// Parameters for setting library permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPermissionsParams {
    /// The ID of the library to modify
    pub library_id: String,
    
    /// Users who can access this library
    pub access_users: Vec<String>,
    
    /// Roles that can access this library
    pub access_roles: Vec<String>,
    
    /// Users who can add items to this library
    pub add_users: Vec<String>,
    
    /// Roles that can add items to this library
    pub add_roles: Vec<String>,
    
    /// Users who can manage this library
    pub manage_users: Vec<String>,
    
    /// Roles that can manage this library
    pub manage_roles: Vec<String>,
    
    /// Users who can modify this library
    pub modify_users: Vec<String>,
    
    /// Roles that can modify this library
    pub modify_roles: Vec<String>,
}

impl GalaxyLibrary {
    /// Create a new Galaxy library with the given name
    pub fn new(name: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(name),
            synopsis: None,
            root_folder_id: "".to_string(), // This will be set when the library is created
            deleted: false,
            public: false,
            permissions: Some(LibraryPermissions {
                access_library_users: Vec::new(),
                access_library_roles: Vec::new(),
                add_library_item_users: Vec::new(),
                add_library_item_roles: Vec::new(),
                manage_library_users: Vec::new(),
                manage_library_roles: Vec::new(),
                modify_library_users: Vec::new(),
                modify_library_roles: Vec::new(),
            }),
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Set the synopsis for this library
    pub fn with_synopsis(&mut self, synopsis: &str) -> &mut Self {
        self.synopsis = Some(synopsis.to_string());
        self
    }
    
    /// Set the public status for this library
    pub fn set_public(&mut self, public: bool) -> &mut Self {
        self.public = public;
        self
    }
    
    /// Set the root folder ID for this library
    pub fn with_root_folder(&mut self, folder_id: &str) -> &mut Self {
        self.root_folder_id = folder_id.to_string();
        self
    }
    
    /// Add a user who can access this library
    pub fn add_access_user(&mut self, user_id: &str) -> &mut Self {
        if let Some(ref mut perms) = self.permissions {
            perms.access_library_users.push(user_id.to_string());
        }
        self
    }
    
    /// Add a user who can add items to this library
    pub fn add_item_user(&mut self, user_id: &str) -> &mut Self {
        if let Some(ref mut perms) = self.permissions {
            perms.add_library_item_users.push(user_id.to_string());
        }
        self
    }
    
    /// Add a user who can manage this library
    pub fn add_manage_user(&mut self, user_id: &str) -> &mut Self {
        if let Some(ref mut perms) = self.permissions {
            perms.manage_library_users.push(user_id.to_string());
        }
        self
    }
    
    /// Mark this library as deleted or not
    pub fn mark_deleted(&mut self, deleted: bool) -> &mut Self {
        self.deleted = deleted;
        self.update_time = chrono::Utc::now().to_rfc3339();
        self
    }
}

impl LibraryFolder {
    /// Create a new library folder with the given name
    pub fn new(name: &str, library_id: &str, parent_id: Option<&str>) -> Self {
        Self {
            id: "".to_string(), // This will be set when the folder is created
            name: name.to_string(),
            description: None,
            parent_id: parent_id.map(|s| s.to_string()),
            library_id: library_id.to_string(),
            deleted: false,
            model_class: "LibraryFolder".to_string(),
            create_time: chrono::Utc::now().to_rfc3339(),
            update_time: chrono::Utc::now().to_rfc3339(),
            genome_build: None,
        }
    }
    
    /// Set the description for this folder
    pub fn with_description(&mut self, description: &str) -> &mut Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set the genome build for this folder
    pub fn with_genome_build(&mut self, genome_build: &str) -> &mut Self {
        self.genome_build = Some(genome_build.to_string());
        self
    }
    
    /// Mark this folder as deleted or not
    pub fn mark_deleted(&mut self, deleted: bool) -> &mut Self {
        self.deleted = deleted;
        self.update_time = chrono::Utc::now().to_rfc3339();
        self
    }
    
    /// Set the ID for this folder
    pub fn with_id(&mut self, id: &str) -> &mut Self {
        self.id = id.to_string();
        self
    }
}

/// Create default permissions for a new library
pub fn create_default_permissions() -> LibraryPermissions {
    LibraryPermissions {
        access_library_users: Vec::new(),
        access_library_roles: Vec::new(),
        add_library_item_users: Vec::new(),
        add_library_item_roles: Vec::new(),
        manage_library_users: Vec::new(),
        manage_library_roles: Vec::new(),
        modify_library_users: Vec::new(),
        modify_library_roles: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_library() {
        let mut library = GalaxyLibrary::new("Test Library");
        
        library
            .with_synopsis("A test library for unit tests")
            .set_public(true)
            .with_root_folder("root123")
            .add_access_user("user123")
            .add_manage_user("admin456");
        
        assert_eq!(library.metadata.name, "Test Library");
        assert_eq!(library.synopsis, Some("A test library for unit tests".to_string()));
        assert_eq!(library.public, true);
        assert_eq!(library.root_folder_id, "root123");
        assert_eq!(library.deleted, false);
        
        // Verify permissions
        if let Some(perms) = &library.permissions {
            assert_eq!(perms.access_library_users, vec!["user123"]);
            assert_eq!(perms.manage_library_users, vec!["admin456"]);
        } else {
            panic!("Library permissions should not be None");
        }
    }
    
    #[test]
    fn test_create_folder() {
        let mut folder = LibraryFolder::new("Test Folder", "lib123", Some("parent456"));
        
        folder
            .with_description("A test folder for unit tests")
            .with_genome_build("hg38")
            .with_id("folder789");
        
        assert_eq!(folder.name, "Test Folder");
        assert_eq!(folder.description, Some("A test folder for unit tests".to_string()));
        assert_eq!(folder.library_id, "lib123");
        assert_eq!(folder.parent_id, Some("parent456".to_string()));
        assert_eq!(folder.genome_build, Some("hg38".to_string()));
        assert_eq!(folder.id, "folder789");
        assert_eq!(folder.deleted, false);
        assert_eq!(folder.model_class, "LibraryFolder");
    }
} 