/*!
 * Galaxy Dataset Plugin Implementation
 * 
 * This module provides a plugin implementation for Galaxy dataset functionality.
 */

use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use async_trait::async_trait;
use serde_json::Value;

use crate::adapter::GalaxyAdapter;
use crate::error::Error;
use crate::plugin::{GalaxyPlugin, GalaxyDatasetPlugin};
use crate::plugin::default_plugin::DefaultGalaxyPlugin;

/// A Galaxy dataset plugin
#[derive(Debug)]
pub struct GalaxyDatasetPluginImpl {
    /// Base plugin implementation
    base: DefaultGalaxyPlugin,
}

impl GalaxyDatasetPluginImpl {
    /// Create a new dataset plugin
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        let base = DefaultGalaxyPlugin::new(name, version, description)
            .with_capability("galaxy-dataset");
        
        Self {
            base,
        }
    }
}

#[async_trait]
impl GalaxyPlugin for GalaxyDatasetPluginImpl {
    fn name(&self) -> &str {
        self.base.name()
    }
    
    fn version(&self) -> &str {
        self.base.version()
    }
    
    fn description(&self) -> &str {
        self.base.description()
    }
    
    async fn initialize(&self, adapter: Arc<GalaxyAdapter>) -> Result<(), Error> {
        self.base.initialize(adapter).await
    }
    
    async fn shutdown(&self) -> Result<(), Error> {
        self.base.shutdown().await
    }
    
    fn provides_capability(&self, capability: &str) -> bool {
        self.base.provides_capability(capability)
    }
    
    fn capabilities(&self) -> Vec<String> {
        self.base.capabilities()
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl GalaxyDatasetPlugin for GalaxyDatasetPluginImpl {
    async fn list_datasets(&self, history_id: &str) -> Result<Vec<Value>, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd list datasets via the adapter
        // For now, we just return mock datasets
        let datasets = vec![
            serde_json::json!({
                "id": "dataset-1",
                "name": "Sample Dataset 1",
                "history_id": history_id,
                "file_type": "tabular",
                "file_size": 1024,
                "state": "ok"
            }),
            serde_json::json!({
                "id": "dataset-2",
                "name": "Sample Dataset 2",
                "history_id": history_id,
                "file_type": "fasta",
                "file_size": 2048,
                "state": "ok"
            })
        ];
        
        Ok(datasets)
    }
    
    async fn get_dataset(&self, dataset_id: &str) -> Result<Option<Value>, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd get the dataset via the adapter
        // For now, we just return a mock dataset
        let dataset = serde_json::json!({
            "id": dataset_id,
            "name": format!("Dataset {}", dataset_id),
            "history_id": "mock-history",
            "file_type": "tabular",
            "file_size": 1024,
            "state": "ok",
            "download_url": format!("https://example.com/datasets/{}/download", dataset_id)
        });
        
        Ok(Some(dataset))
    }
    
    async fn upload_data(&self, _history_id: &str, _name: &str, _data: Vec<u8>, _file_type: &str) -> Result<String, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd upload the data via the adapter
        // For now, we just return a mock dataset ID
        let dataset_id = format!("dataset-{}", uuid::Uuid::new_v4());
        
        Ok(dataset_id)
    }
    
    async fn download_dataset(&self, _dataset_id: &str) -> Result<Vec<u8>, Error> {
        // Get the adapter
        let _adapter = self.base.adapter()?;
        
        // In a real implementation, we'd download the dataset via the adapter
        // For now, we just return mock data
        let data = b"Mock dataset content".to_vec();
        
        Ok(data)
    }
    
    async fn list_collections(&self, history_id: &str) -> Result<Vec<Value>, Error> {
        // Get the adapter
        let adapter = self.base.adapter()?;
        
        // Call the adapter's list_collections method
        let collections = adapter.list_collections(history_id).await?;
        
        // Convert the collections to JSON values
        let collection_values = collections
            .into_iter()
            .map(|c| serde_json::to_value(c).unwrap_or_default())
            .collect();
            
        Ok(collection_values)
    }
    
    async fn get_collection(&self, collection_id: &str) -> Result<Option<Value>, Error> {
        // Get the adapter
        let adapter = self.base.adapter()?;
        
        // Try to get the collection
        match adapter.get_dataset_collection(collection_id).await {
            Ok(collection) => {
                // Convert the collection to a JSON value
                let collection_value = serde_json::to_value(collection)
                    .map_err(|e| Error::SerializationError(format!("Failed to serialize collection: {}", e)))?;
                
                Ok(Some(collection_value))
            },
            Err(Error::NotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }
    
    async fn create_collection(&self, history_id: &str, name: &str, collection_type: &str, dataset_ids: Vec<String>) -> Result<String, Error> {
        // Get the adapter
        let adapter = self.base.adapter()?;
        
        // Convert dataset_ids to CollectionElement objects
        let elements = dataset_ids.iter().enumerate().map(|(i, id)| {
            crate::models::dataset::CollectionElement {
                id: format!("element-{}", i),
                element_index: i,
                element_identifier: format!("sample{}", i + 1),
                dataset_id: Some(id.clone()),
                child_collection_id: None,
                model_class: "HistoryDatasetAssociation".to_string(),
            }
        }).collect();
        
        // Create the collection
        let collection = adapter.create_dataset_collection(history_id, name, collection_type, elements).await?;
        
        // Return the collection ID
        Ok(collection.metadata.id)
    }
    
    async fn get_collection_elements(&self, collection_id: &str) -> Result<Vec<Value>, Error> {
        // Get the adapter
        let adapter = self.base.adapter()?;
        
        // Get the collection elements
        let elements = adapter.get_dataset_collection_elements(collection_id).await?;
        
        // Convert the elements to JSON values
        let element_values = elements
            .into_iter()
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect();
            
        Ok(element_values)
    }
    
    async fn delete_collection(&self, collection_id: &str) -> Result<(), Error> {
        // Get the adapter
        let adapter = self.base.adapter()?;
        
        // Delete the collection
        adapter.delete_dataset_collection(collection_id).await
    }
}

/// Factory function to create a new Galaxy dataset plugin
pub fn create_dataset_plugin(name: &str, version: &str, description: &str) -> GalaxyDatasetPluginImpl {
    GalaxyDatasetPluginImpl::new(name, version, description)
} 