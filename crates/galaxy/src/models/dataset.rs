//! Module defining Galaxy dataset data models
//! 
//! This module contains the data structures for representing Galaxy datasets,
//! dataset collections, and related objects.

use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::models::ResourceMetadata;

/// Represents a Galaxy dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyDataset {
    /// Common metadata for the dataset
    pub metadata: ResourceMetadata,
    
    /// The file type/format of this dataset
    pub file_ext: String,
    
    /// Size of the dataset in bytes
    pub file_size: Option<u64>,
    
    /// The state of this dataset
    pub state: DatasetState,
    
    /// Whether this dataset has been deleted
    pub deleted: bool,
    
    /// Whether this dataset has been purged
    pub purged: bool,
    
    /// Whether this dataset is visible
    pub visible: bool,
    
    /// URL to download this dataset
    pub download_url: Option<String>,
    
    /// ID of the history this dataset belongs to
    pub history_id: Option<String>,
    
    /// Whether this dataset contains sensitive information
    pub accessible: bool,
    
    /// ID of the library this dataset belongs to, if any
    pub library_folder_id: Option<String>,
    
    /// Information message about this dataset
    pub misc_info: Option<String>,
    
    /// Additional datatype-specific metadata
    pub data_type_metadata: HashMap<String, String>,
    
    /// Information about the dataset's creation
    pub creating_job: Option<String>,
    
    /// The URL of the dataset contents
    pub file_name: Option<String>,
    
    /// The display name in history
    pub display_name: Option<String>,
    
    /// Whether this dataset is on disk
    pub file_available: bool,
}

/// Represents a collection of Galaxy datasets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyDatasetCollection {
    /// Common metadata for the collection
    pub metadata: ResourceMetadata,
    
    /// The type of collection (list, paired, list:paired, etc.)
    pub collection_type: String,
    
    /// The elements in this collection
    pub elements: Vec<CollectionElement>,
    
    /// The number of elements in this collection
    pub element_count: usize,
    
    /// The ID of the history this collection belongs to
    pub history_id: Option<String>,
    
    /// Whether this collection has been populated
    pub populated: bool,
    
    /// The collecting job, if any
    pub populated_state: PopulatedState,
    
    /// The job that created this collection, if any
    pub populated_state_message: Option<String>,
}

/// Represents an element in a dataset collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionElement {
    /// The ID of this element
    pub id: String,
    
    /// The position of this element in the collection
    pub element_index: usize,
    
    /// The identifier for this element
    pub element_identifier: String,
    
    /// The dataset ID this element points to
    pub dataset_id: Option<String>,
    
    /// The child collection ID this element points to
    pub child_collection_id: Option<String>,
    
    /// The model class of this element ("HistoryDatasetAssociation" or "HistoryDatasetCollectionAssociation")
    pub model_class: String,
}

/// Represents a library dataset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDataset {
    /// The ID of this library dataset
    pub id: String,
    
    /// The name of this dataset
    pub name: String,
    
    /// File type/format of this dataset
    pub file_ext: String,
    
    /// The size of this dataset in bytes
    pub file_size: Option<u64>,
    
    /// The state of this dataset
    pub state: DatasetState,
    
    /// Whether this dataset is deleted
    pub deleted: bool,
    
    /// The folder ID this dataset belongs to
    pub folder_id: String,
    
    /// The date this dataset was created
    pub create_time: String,
    
    /// The date this dataset was last updated
    pub update_time: String,
    
    /// Access URL for this dataset
    pub file_name: Option<String>,
    
    /// Description of this dataset
    pub description: Option<String>,
    
    /// The ID of the library this dataset belongs to
    pub library_id: String,
}

/// Represents a library folder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryFolder {
    /// The ID of this folder
    pub id: String,
    
    /// The name of this folder
    pub name: String,
    
    /// Description of this folder
    pub description: Option<String>,
    
    /// The creation date of this folder
    pub create_time: String,
    
    /// The last update date of this folder
    pub update_time: String,
    
    /// The ID of the parent folder
    pub parent_id: Option<String>,
    
    /// The ID of the library this folder belongs to
    pub library_id: String,
    
    /// Whether this folder is deleted
    pub deleted: bool,
}

/// Represents a Galaxy library
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyLibrary {
    /// The ID of this library
    pub id: String,
    
    /// The name of this library
    pub name: String,
    
    /// The description of this library
    pub description: Option<String>,
    
    /// The synopsis of this library
    pub synopsis: Option<String>,
    
    /// The creation date of this library
    pub create_time: String,
    
    /// The last update date of this library
    pub update_time: String,
    
    /// The root folder ID of this library
    pub root_folder_id: String,
    
    /// Whether this library is deleted
    pub deleted: bool,
    
    /// Whether this library is public
    pub public: bool,
}

/// The possible states of a dataset
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DatasetState {
    /// The dataset is new
    #[serde(rename = "new")]
    New,
    
    /// The dataset upload has started
    #[serde(rename = "upload")]
    Upload,
    
    /// The dataset is being queued
    #[serde(rename = "queued")]
    Queued,
    
    /// The dataset is running
    #[serde(rename = "running")]
    Running,
    
    /// The dataset is being downloaded
    #[serde(rename = "download")]
    Download,
    
    /// The dataset is ready and complete
    #[serde(rename = "ok")]
    Ok,
    
    /// The dataset processing failed
    #[serde(rename = "error")]
    Error,
    
    /// The dataset is paused
    #[serde(rename = "paused")]
    Paused,
    
    /// The dataset is deferred
    #[serde(rename = "deferred")]
    Deferred,
    
    /// The dataset is discarded
    #[serde(rename = "discarded")]
    Discarded,
    
    /// Custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

/// The possible populated states of a dataset collection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PopulatedState {
    /// The collection is not yet populated (still being created)
    #[serde(rename = "new")]
    New,
    
    /// The collection is in the process of being populated
    #[serde(rename = "running")]
    Running,
    
    /// The collection has been successfully populated
    #[serde(rename = "ok")]
    Ok,
    
    /// There was an error populating the collection
    #[serde(rename = "failed")]
    Failed,
    
    /// Custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

/// Parameters for dataset upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetUploadParams {
    /// The file to upload
    pub file_content: Vec<u8>,
    
    /// The file name to use
    pub file_name: String,
    
    /// The filetype/extension
    pub file_type: String,
    
    /// The history ID to upload to
    pub history_id: Option<String>,
    
    /// Whether to create a new history
    pub create_new_history: bool,
    
    /// The ID of the library folder to upload to, if uploading to a library
    pub library_folder_id: Option<String>,
    
    /// Additional dataset attributes
    pub dbkey: String,
    
    /// Whether to extract files from archives
    pub extract_metadata: bool,
    
    /// Tag list to add to dataset
    pub tags: Option<Vec<String>>,
}

/// Parameters for dataset download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetDownloadParams {
    /// The dataset ID to download
    pub dataset_id: String,
    
    /// The history ID (if dataset is in history)
    pub history_id: Option<String>,
    
    /// The library ID (if dataset is in library)
    pub library_id: Option<String>,
    
    /// Whether to include the dataset metadata
    pub include_metadata: bool,
    
    /// The file format to download as (if conversion is supported)
    pub file_ext: Option<String>,
}

/// Information about a dataset preview
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetPreview {
    /// The dataset ID
    pub id: String,
    
    /// The preview data (first N lines)
    pub data: Vec<String>,
    
    /// The dataset metadata
    pub metadata: HashMap<String, String>,
    
    /// Whether there is more data beyond the preview
    pub has_more: bool,
    
    /// The total number of lines in the dataset
    pub total_lines: Option<usize>,
}

/// Dataset search parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetSearchParams {
    /// Text to search for
    pub query: Option<String>,
    
    /// Limit results to specific history
    pub history_id: Option<String>,
    
    /// Limit results to specific library
    pub library_id: Option<String>,
    
    /// Limit results to specific file types
    pub file_ext: Option<Vec<String>>,
    
    /// Minimum file size to return
    pub min_size: Option<u64>,
    
    /// Maximum file size to return
    pub max_size: Option<u64>,
    
    /// Limit results to datasets created after this date
    pub created_after: Option<String>,
    
    /// Limit results to datasets created before this date
    pub created_before: Option<String>,
    
    /// Tags to filter by
    pub tags: Option<Vec<String>>,
    
    /// Whether to include deleted datasets
    pub include_deleted: bool,
}

/// Alias for GalaxyDataset
pub type Dataset = GalaxyDataset;

impl GalaxyDataset {
    /// Create a new Galaxy dataset with a name and file extension
    pub fn new(name: &str, file_ext: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(name),
            file_ext: file_ext.to_string(),
            file_size: None,
            state: DatasetState::New,
            deleted: false,
            purged: false,
            visible: true,
            download_url: None,
            history_id: None,
            accessible: true,
            library_folder_id: None,
            misc_info: None,
            data_type_metadata: HashMap::new(),
            creating_job: None,
            file_name: None,
            display_name: None,
            file_available: false,
        }
    }
    
    /// Set the state of this dataset
    pub fn with_state(&mut self, state: DatasetState) -> &mut Self {
        self.state = state;
        self
    }
    
    /// Set the file size of this dataset
    pub fn with_file_size(&mut self, size: u64) -> &mut Self {
        self.file_size = Some(size);
        self
    }
    
    /// Set the history ID for this dataset
    pub fn in_history(&mut self, history_id: &str) -> &mut Self {
        self.history_id = Some(history_id.to_string());
        self
    }
    
    /// Set the library folder ID for this dataset
    pub fn in_library(&mut self, folder_id: &str) -> &mut Self {
        self.library_folder_id = Some(folder_id.to_string());
        self
    }
    
    /// Mark this dataset as deleted
    pub fn mark_deleted(&mut self, deleted: bool) -> &mut Self {
        self.deleted = deleted;
        self
    }
    
    /// Add a piece of metadata to this dataset
    pub fn add_metadata(&mut self, key: &str, value: &str) -> &mut Self {
        self.data_type_metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl GalaxyDatasetCollection {
    /// Create a new dataset collection with a name and collection type
    pub fn new(name: &str, collection_type: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(name),
            collection_type: collection_type.to_string(),
            elements: Vec::new(),
            element_count: 0,
            history_id: None,
            populated: false,
            populated_state: PopulatedState::New,
            populated_state_message: None,
        }
    }
    
    /// Add an element to this collection
    pub fn add_element(&mut self, element: CollectionElement) -> &mut Self {
        self.elements.push(element);
        self.element_count = self.elements.len();
        self
    }
    
    /// Set the history ID for this collection
    pub fn in_history(&mut self, history_id: &str) -> &mut Self {
        self.history_id = Some(history_id.to_string());
        self
    }
    
    /// Mark this collection as populated
    pub fn mark_populated(&mut self, populated: bool) -> &mut Self {
        self.populated = populated;
        if populated {
            self.populated_state = PopulatedState::Ok;
        }
        self
    }
    
    /// Set the populated state of this collection
    pub fn with_populated_state(&mut self, state: PopulatedState) -> &mut Self {
        self.populated_state = state.clone();
        self.populated = matches!(state, PopulatedState::Ok);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_dataset() {
        let mut dataset = GalaxyDataset::new("Test Dataset", "txt");
        
        dataset
            .with_file_size(1024)
            .in_history("history123")
            .with_state(DatasetState::Ok)
            .add_metadata("column_names", "col1,col2,col3");
        
        assert_eq!(dataset.metadata.name, "Test Dataset");
        assert_eq!(dataset.file_ext, "txt");
        assert_eq!(dataset.file_size, Some(1024));
        assert_eq!(dataset.history_id, Some("history123".to_string()));
        assert_eq!(dataset.state, DatasetState::Ok);
        assert_eq!(dataset.data_type_metadata.get("column_names"), Some(&"col1,col2,col3".to_string()));
    }
    
    #[test]
    fn test_create_collection() {
        let mut collection = GalaxyDatasetCollection::new("Test Collection", "list");
        
        let element = CollectionElement {
            id: "el1".to_string(),
            element_index: 0,
            element_identifier: "sample1".to_string(),
            dataset_id: Some("ds123".to_string()),
            child_collection_id: None,
            model_class: "HistoryDatasetAssociation".to_string(),
        };
        
        collection
            .add_element(element)
            .in_history("history456")
            .mark_populated(true);
        
        assert_eq!(collection.metadata.name, "Test Collection");
        assert_eq!(collection.collection_type, "list");
        assert_eq!(collection.element_count, 1);
        assert_eq!(collection.history_id, Some("history456".to_string()));
        assert_eq!(collection.populated, true);
        assert_eq!(collection.populated_state, PopulatedState::Ok);
    }
} 