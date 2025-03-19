use std::sync::Arc;
use crate::error::Result;
use super::{
    MonitoringService,
    MonitoringConfig,
    MonitoringServiceFactory,
    HealthCheckerAdapter,
    DefaultMetricCollector,
};
use super::alerts::AlertManagerAdapter;
use super::network::NetworkMonitorAdapter;
use super::alerts::NotificationManagerTrait;
use super::health::create_checker_adapter;

/// Adapter for monitoring service factory
/// 
/// This adapter wraps a monitoring service factory to provide a consistent interface
/// and additional functionality while delegating to the underlying implementation.
#[derive(Debug)]
pub struct MonitoringServiceFactoryAdapter<N: NotificationManagerTrait + 'static = ()> {
    /// Underlying factory instance that will be used to create monitoring services
    /// The adapter delegates creation requests to this inner factory.
    pub inner: Option<Arc<MonitoringServiceFactory<N>>>,
}

impl<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static> MonitoringServiceFactoryAdapter<N> {
    /// Creates a new factory adapter
    #[must_use]
    pub fn new() -> Self {
        Self { inner: None }
    }

    /// Creates a new adapter with an existing factory
    #[must_use]
    pub fn with_factory(factory: Arc<MonitoringServiceFactory<N>>) -> Self {
        Self {
            inner: Some(factory),
        }
    }

    /// Creates a service using the default configuration
    #[must_use]
    pub fn create_service(&self) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service()
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).create_service()
        }
    }

    /// Creates a service with a custom configuration
    #[must_use]
    pub fn create_service_with_config(&self, config: MonitoringConfig) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_config(config)
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).create_service_with_config(config)
        }
    }

    /// Creates a service with explicit dependencies
    #[must_use]
    pub fn create_service_with_dependencies(
        &self,
        config: MonitoringConfig,
        health_checker: Arc<HealthCheckerAdapter>,
        metric_collector: Arc<DefaultMetricCollector>,
        alert_manager: Arc<AlertManagerAdapter>,
        network_monitor: Arc<NetworkMonitorAdapter>,
    ) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_dependencies(
                config,
                health_checker,
                metric_collector,
                alert_manager,
                network_monitor,
            )
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).create_service_with_dependencies(
                config,
                health_checker,
                metric_collector,
                alert_manager,
                network_monitor,
            )
        }
    }

    /// Creates a service using adapter pattern for ongoing transition
    #[must_use]
    pub fn create_service_with_adapters(&self) -> Arc<MonitoringService> {
        if let Some(factory) = &self.inner {
            factory.create_service_with_adapters()
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).create_service_with_adapters()
        }
    }

    /// Starts a new service with the default configuration
    pub async fn start_service(&self) -> Result<Arc<MonitoringService>> {
        if let Some(factory) = &self.inner {
            factory.start_service().await
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(MonitoringConfig::default());
            Arc::new(factory).start_service().await
        }
    }

    /// Starts a new service with a custom configuration
    pub async fn start_service_with_config(&self, config: MonitoringConfig) -> Result<Arc<MonitoringService>> {
        if let Some(factory) = &self.inner {
            factory.start_service_with_config(config).await
        } else {
            // Initialize on-demand with default configuration
            let factory: MonitoringServiceFactory<()> = MonitoringServiceFactory::with_config(config.clone());
            Arc::new(factory).start_service_with_config(config).await
        }
    }
}

impl<N: NotificationManagerTrait + 'static> Default for MonitoringServiceFactoryAdapter<N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates a new monitoring service factory adapter
#[must_use]
pub fn create_factory_adapter<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static>() -> Arc<MonitoringServiceFactoryAdapter<N>> {
    Arc::new(MonitoringServiceFactoryAdapter::new())
}

/// Creates a new monitoring service factory adapter with an existing factory
#[must_use]
pub fn create_factory_adapter_with_factory<N: NotificationManagerTrait + Send + Sync + std::fmt::Debug + 'static>(
    factory: Arc<MonitoringServiceFactory<N>>
) -> Arc<MonitoringServiceFactoryAdapter<N>> {
    Arc::new(MonitoringServiceFactoryAdapter::with_factory(factory))
} 