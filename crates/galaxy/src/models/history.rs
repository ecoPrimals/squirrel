//! Module defining Galaxy history data models
//! 
//! This module contains the data structures for representing Galaxy histories,
//! history contents, and related objects.

use serde::{Serialize, Deserialize};

use crate::models::ResourceMetadata;
use crate::models::dataset::{GalaxyDataset, GalaxyDatasetCollection};

/// Represents a Galaxy history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalaxyHistory {
    /// Common metadata for the history
    pub metadata: ResourceMetadata,
    
    /// Whether this history has been deleted
    pub deleted: bool,
    
    /// Whether this history is purged
    pub purged: bool,
    
    /// The user ID that owns this history
    pub user_id: Option<String>,
    
    /// The model class for this history
    pub model_class: String,
    
    /// The URL to access this history
    pub url: Option<String>,
    
    /// The number of datasets in this history
    pub dataset_count: usize,
    
    /// A size estimate for this history in bytes
    pub size: Option<u64>,
    
    /// Whether this history is published
    pub published: bool,
    
    /// The state of this history (active, archived, etc.)
    pub state: HistoryState,
    
    /// Tags assigned to this history
    pub tags: Vec<String>,
    
    /// The date this history was last used
    pub update_time: String,
    
    /// The slug for this history (used in URLs)
    pub slug: Option<String>,
    
    /// Whether this history is importable
    pub importable: bool,
    
    /// The ID of the genome build used in this history
    pub genome_build: Option<String>,
    
    /// The annotation for this history
    pub annotation: Option<String>,
}

/// Represents a history item (dataset or collection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    /// The ID of this history item
    pub id: String,
    
    /// The name of this history item
    pub name: String,
    
    /// The history ID this item belongs to
    pub history_id: String,
    
    /// Whether this is a dataset or collection
    pub item_type: HistoryItemType,
    
    /// The type-specific ID (dataset ID or collection ID)
    pub item_id: String,
    
    /// The file extension, if this is a dataset
    pub file_ext: Option<String>,
    
    /// The collection type, if this is a collection
    pub collection_type: Option<String>,
    
    /// Whether this item is deleted
    pub deleted: bool,
    
    /// Whether this item is visible
    pub visible: bool,
    
    /// The state of this item
    pub state: String,
    
    /// The creation time of this item
    pub create_time: String,
    
    /// The update time of this item
    pub update_time: String,
    
    /// Tags for this item
    pub tags: Vec<String>,
    
    /// A file type/format description
    pub file_ext_description: Option<String>,
}

/// Represents the type of a history item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HistoryItemType {
    /// A single dataset
    #[serde(rename = "dataset")]
    Dataset,
    
    /// A collection of datasets
    #[serde(rename = "dataset_collection")]
    Collection,
    
    /// Custom type not covered by the standard types
    #[serde(untagged)]
    Custom(String),
}

/// Represents the state of a history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HistoryState {
    /// The history is currently being used
    #[serde(rename = "active")]
    Active,
    
    /// The history has been archived
    #[serde(rename = "archived")]
    Archived,
    
    /// The history is being used in a workflow
    #[serde(rename = "running")]
    Running,
    
    /// The history is queued for processing
    #[serde(rename = "queued")]
    Queued,
    
    /// The history failed during creation or execution
    #[serde(rename = "error")]
    Error,
    
    /// The history was imported
    #[serde(rename = "imported")]
    Imported,
    
    /// Custom state not covered by the standard states
    #[serde(untagged)]
    Custom(String),
}

/// Parameters for creating a new history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryCreateParams {
    /// The name for the new history
    pub name: String,
    
    /// An optional history to copy from
    pub copy_history_id: Option<String>,
    
    /// Whether this history should be published
    pub published: bool,
    
    /// Whether to include all datasets or just non-deleted ones
    pub all_datasets: bool,
    
    /// Tags to apply to the history
    pub tags: Option<Vec<String>>,
    
    /// Annotation for the history
    pub annotation: Option<String>,
}

/// Parameters for copying a history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryCopyParams {
    /// The ID of the history to copy
    pub history_id: String,
    
    /// The name for the new history copy
    pub name: Option<String>,
    
    /// Whether to include all datasets or just non-deleted ones
    pub all_datasets: bool,
    
    /// Whether to include deleted histories when copying
    pub include_deleted: bool,
    
    /// Whether to include hidden histories when copying
    pub include_hidden: bool,
}

/// Parameters for updating a history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryUpdateParams {
    /// The new name for the history, if provided
    pub name: Option<String>,
    
    /// New tags for the history, if provided
    pub tags: Option<Vec<String>>,
    
    /// New annotation for the history, if provided
    pub annotation: Option<String>,
    
    /// New published status for the history, if provided
    pub published: Option<bool>,
    
    /// New deleted status for the history, if provided
    pub deleted: Option<bool>,
    
    /// New purged status for the history, if provided
    pub purged: Option<bool>,
}

/// Parameters for searching histories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySearchParams {
    /// Text to search for in history names and descriptions
    pub query: Option<String>,
    
    /// Filter by update time (from)
    pub update_time_gte: Option<String>,
    
    /// Filter by update time (to)
    pub update_time_lte: Option<String>,
    
    /// Include only histories with this tag
    pub tag: Option<String>,
    
    /// Include only histories with this annotation
    pub annotation: Option<String>,
    
    /// Filter by history state
    pub state: Option<HistoryState>,
    
    /// Include deleted histories
    pub include_deleted: bool,
    
    /// Show only published histories
    pub only_published: bool,
}

impl GalaxyHistory {
    /// Create a new Galaxy history with the given name
    pub fn new(name: &str) -> Self {
        Self {
            metadata: ResourceMetadata::new(name),
            deleted: false,
            purged: false,
            user_id: None,
            model_class: "History".to_string(),
            url: None,
            dataset_count: 0,
            size: Some(0),
            published: false,
            state: HistoryState::Active,
            tags: Vec::new(),
            update_time: chrono::Utc::now().to_rfc3339(),
            slug: None,
            importable: false,
            genome_build: None,
            annotation: None,
        }
    }
    
    /// Set the user ID for this history
    pub fn with_user_id(&mut self, user_id: &str) -> &mut Self {
        self.user_id = Some(user_id.to_string());
        self
    }
    
    /// Set the published state of this history
    pub fn set_published(&mut self, published: bool) -> &mut Self {
        self.published = published;
        self
    }
    
    /// Set the state of this history
    pub fn with_state(&mut self, state: HistoryState) -> &mut Self {
        self.state = state;
        self
    }
    
    /// Update the dataset count for this history
    pub fn set_dataset_count(&mut self, count: usize) -> &mut Self {
        self.dataset_count = count;
        self
    }
    
    /// Mark this history as deleted or not
    pub fn mark_deleted(&mut self, deleted: bool) -> &mut Self {
        self.deleted = deleted;
        self
    }
    
    /// Add a tag to this history
    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.push(tag.to_string());
        self
    }
    
    /// Set the annotation for this history
    pub fn with_annotation(&mut self, annotation: &str) -> &mut Self {
        self.annotation = Some(annotation.to_string());
        self
    }
    
    /// Get the effective name of this history (for display)
    pub fn get_display_name(&self) -> String {
        if self.metadata.name.is_empty() {
            "Unnamed History".to_string()
        } else {
            self.metadata.name.clone()
        }
    }
}

/// Create a new history item for a dataset
pub fn create_dataset_item(
    dataset: &GalaxyDataset,
    history_id: &str,
) -> HistoryItem {
    HistoryItem {
        id: format!("{}:{}", history_id, dataset.metadata.id),
        name: dataset.metadata.name.clone(),
        history_id: history_id.to_string(),
        item_type: HistoryItemType::Dataset,
        item_id: dataset.metadata.id.clone(),
        file_ext: Some(dataset.file_ext.clone()),
        collection_type: None,
        deleted: dataset.deleted,
        visible: dataset.visible,
        state: format!("{:?}", dataset.state).to_lowercase(),
        create_time: dataset.metadata.create_time.to_string(),
        update_time: dataset.metadata.update_time.to_string(),
        tags: dataset.metadata.tags.clone(),
        file_ext_description: Some(format!("{} file", dataset.file_ext)),
    }
}

/// Create a new history item for a collection
pub fn create_collection_item(
    collection: &GalaxyDatasetCollection,
    history_id: &str,
) -> HistoryItem {
    HistoryItem {
        id: format!("{}:{}", history_id, collection.metadata.id),
        name: collection.metadata.name.clone(),
        history_id: history_id.to_string(),
        item_type: HistoryItemType::Collection,
        item_id: collection.metadata.id.clone(),
        file_ext: None,
        collection_type: Some(collection.collection_type.clone()),
        deleted: false,
        visible: true,
        state: format!("{:?}", collection.populated_state).to_lowercase(),
        create_time: collection.metadata.create_time.to_string(),
        update_time: collection.metadata.update_time.to_string(),
        tags: collection.metadata.tags.clone(),
        file_ext_description: None,
    }
}

/// Alias for GalaxyHistory
pub type History = GalaxyHistory;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dataset::{DatasetState, PopulatedState};
    
    #[test]
    fn test_create_history() {
        let mut history = GalaxyHistory::new("Test History");
        
        history
            .with_user_id("user123")
            .set_published(true)
            .add_tag("test")
            .with_annotation("This is a test history");
        
        assert_eq!(history.metadata.name, "Test History");
        assert_eq!(history.user_id, Some("user123".to_string()));
        assert_eq!(history.published, true);
        assert_eq!(history.tags, vec!["test"]);
        assert_eq!(history.annotation, Some("This is a test history".to_string()));
        assert_eq!(history.deleted, false);
        assert_eq!(history.state, HistoryState::Active);
    }
    
    #[test]
    fn test_create_history_items() {
        // Create a test dataset
        let mut dataset = GalaxyDataset::new("Test Dataset", "txt");
        dataset.with_state(DatasetState::Ok);
        
        // Create a test collection
        let mut collection = GalaxyDatasetCollection::new("Test Collection", "list");
        collection.with_populated_state(PopulatedState::Ok);
        
        // Create history items
        let history_id = "history123";
        let dataset_item = create_dataset_item(&dataset, history_id);
        let collection_item = create_collection_item(&collection, history_id);
        
        // Verify dataset item
        assert_eq!(dataset_item.name, "Test Dataset");
        assert_eq!(dataset_item.history_id, history_id);
        assert_eq!(dataset_item.item_type, HistoryItemType::Dataset);
        assert_eq!(dataset_item.file_ext, Some("txt".to_string()));
        
        // Verify collection item
        assert_eq!(collection_item.name, "Test Collection");
        assert_eq!(collection_item.history_id, history_id);
        assert_eq!(collection_item.item_type, HistoryItemType::Collection);
        assert_eq!(collection_item.collection_type, Some("list".to_string()));
    }
} 