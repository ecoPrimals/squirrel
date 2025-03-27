---
description: DEFINE standards for UI component integration with the dashboard core
---

# Dashboard Core Integration for UI Components

## Context

- When building UI components that display dashboard data
- When integrating new visualization components with existing dashboard
- When extending dashboard functionality with additional data sources
- When implementing custom metrics displays

## Integration Patterns

### Component Integration

UI components should follow these patterns when integrating with the dashboard core:

1. **Data-Driven Components**
   
   Components should be purely data-driven, accepting dashboard data as props/parameters:

   ```rust
   /// A dashboard widget for displaying metrics
   pub struct MetricsWidget<'a> {
       /// The metrics to display
       metrics: &'a MetricsSnapshot,
       
       /// Widget title
       title: &'a str,
   }
   
   impl<'a> MetricsWidget<'a> {
       /// Create a new metrics widget
       pub fn new(metrics: &'a MetricsSnapshot, title: &'a str) -> Self {
           Self { metrics, title }
       }
       
       /// Render the widget
       pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
           // Rendering implementation
       }
   }
   ```

2. **Separation of Data Access and Rendering**

   Components should separate data access from rendering:

   ```rust
   // Dashboard data access (in app.rs)
   impl App {
       pub fn dashboard_data(&self) -> Option<&DashboardData> {
           self.dashboard_data.as_ref()
       }
   }
   
   // Component rendering (in ui.rs)
   fn draw_metrics_tab(f: &mut Frame, app: &App, area: Rect) {
       if let Some(data) = app.dashboard_data() {
           let metrics_widget = MetricsWidget::new(&data.metrics, "System Metrics");
           metrics_widget.render(f, area);
       } else {
           draw_loading_state(f, area);
       }
   }
   ```

3. **Error States Handling**

   Components should handle error states gracefully:

   ```rust
   fn draw_metrics_tab(f: &mut Frame, app: &App, area: Rect) {
       match app.dashboard_data() {
           Some(data) => {
               let metrics_widget = MetricsWidget::new(&data.metrics, "System Metrics");
               metrics_widget.render(f, area);
           },
           None if app.is_loading() => {
               draw_loading_state(f, area);
           },
           None if app.has_error() => {
               draw_error_state(f, area, app.error_message());
           },
           None => {
               draw_empty_state(f, area);
           }
       }
   }
   ```

### Widget Design Standards

1. **Configuration Options**

   Widgets should be configurable via builder pattern:

   ```rust
   /// Chart widget configuration
   impl<'a> ChartWidget<'a> {
       /// Set the chart title
       pub fn title(mut self, title: &'a str) -> Self {
           self.title = title;
           self
       }
       
       /// Set the Y-axis label
       pub fn y_label(mut self, y_label: &'a str) -> Self {
           self.y_label = y_label;
           self
       }
       
       /// Set the chart type
       pub fn chart_type(mut self, chart_type: ChartType) -> Self {
           self.chart_type = chart_type;
           self
       }
       
       /// Set the time range in seconds
       pub fn time_range(mut self, seconds: u64) -> Self {
           self.time_range = seconds;
           self
       }
   }
   ```

2. **Generic Backend Support**

   Widgets should support generic rendering backends:

   ```rust
   /// Render the widget with any backend
   pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
       // Backend-agnostic rendering implementation
   }
   ```

3. **Themeable Design**

   Widgets should support theming:

   ```rust
   /// Theme-aware widget
   pub struct ThemedWidget<'a> {
       // Widget data
       data: &'a Data,
       // Theme configuration
       theme: Option<&'a Theme>,
   }
   
   impl<'a> ThemedWidget<'a> {
       /// Get the appropriate color for a value based on theme
       fn get_color(&self, value: f64) -> Color {
           if let Some(theme) = self.theme {
               // Use theme colors
               if value > theme.critical_threshold {
                   theme.critical_color
               } else if value > theme.warning_threshold {
                   theme.warning_color
               } else {
                   theme.normal_color
               }
           } else {
               // Default colors
               if value > 0.9 {
                   Color::Red
               } else if value > 0.7 {
                   Color::Yellow
               } else {
                   Color::Green
               }
           }
       }
   }
   ```

### Dashboard Data Access

UI components should access dashboard data through these standard methods:

1. **Current Data Access**

   ```rust
   // In App
   pub fn dashboard_data(&self) -> Option<&DashboardData> {
       self.dashboard_data.as_ref()
   }
   
   // In UI
   if let Some(data) = app.dashboard_data() {
       // Use current dashboard data
   }
   ```

2. **Historical Data Access**

   ```rust
   // In App
   pub fn get_metric_history(&self, metric_name: &str) -> Option<&[(DateTime<Utc>, f64)]> {
       self.metric_history.get(metric_name).map(|v| v.as_slice())
   }
   
   // In UI
   if let Some(history) = app.get_metric_history("system.cpu") {
       let chart = ChartWidget::new(history, "CPU Usage");
       // Render chart
   }
   ```

3. **Alert Data Access**

   ```rust
   // In App
   pub fn active_alerts(&self) -> &[Alert] {
       if let Some(data) = &self.dashboard_data {
           &data.alerts.active
       } else {
           &[]
       }
   }
   
   // In UI
   let alerts = app.active_alerts();
   let alerts_widget = AlertsWidget::new(alerts, "Active Alerts");
   // Render alerts
   ```

## Dashboard Core Extensions

### Custom Metrics Integration

To add custom metrics to the dashboard core:

1. **Implement Metrics Collector**

   ```rust
   /// Custom metrics collector
   pub struct CustomMetricsCollector {
       // Collector state
   }
   
   impl CustomMetricsCollector {
       /// Collect custom metrics
       pub fn collect(&self) -> HashMap<String, f64> {
           let mut metrics = HashMap::new();
           // Collect custom metrics
           metrics.insert("custom.metric1".to_string(), 42.0);
           metrics.insert("custom.metric2".to_string(), 7.0);
           metrics
       }
   }
   ```

2. **Integrate with Dashboard Service**

   ```rust
   /// Extend dashboard service with custom metrics
   pub async fn update_dashboard_with_custom_metrics(
       dashboard_service: &Arc<dyn DashboardService>,
       collector: &CustomMetricsCollector
   ) -> Result<()> {
       // Get current dashboard data
       let mut data = dashboard_service.get_dashboard_data().await?;
       
       // Collect custom metrics
       let custom_metrics = collector.collect();
       
       // Add custom metrics to dashboard data
       for (key, value) in custom_metrics {
           data.metrics.values.insert(key, value);
       }
       
       // Update dashboard
       dashboard_service.update_dashboard_data(data).await
   }
   ```

### Custom Widgets

To implement custom widgets for the dashboard:

1. **Create Widget Definition**

   ```rust
   /// Custom dashboard widget
   pub struct CustomWidget<'a> {
       /// Data to display
       data: &'a CustomData,
       /// Widget title
       title: &'a str,
   }
   ```

2. **Implement Rendering**

   ```rust
   impl<'a> CustomWidget<'a> {
       /// Create a new custom widget
       pub fn new(data: &'a CustomData, title: &'a str) -> Self {
           Self { data, title }
       }
       
       /// Render the widget
       pub fn render<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
           // Create block
           let block = Block::default()
               .borders(Borders::ALL)
               .title(self.title);
           
           // Render block
           f.render_widget(block.clone(), area);
           
           // Get inner area
           let inner = block.inner(area);
           
           // Render content
           // ...
       }
   }
   ```

3. **Integrate in Dashboard UI**

   ```rust
   /// Add custom widget to dashboard tabs
   fn draw_custom_tab(f: &mut Frame, app: &App, area: Rect) {
       if let Some(data) = app.get_custom_data() {
           let widget = CustomWidget::new(data, "Custom Data");
           widget.render(f, area);
       } else {
           // Show fallback
       }
   }
   ```

## Dashboard Service Extensions

### Extending Dashboard Updates

To handle custom update types:

1. **Add Custom Update Type**

   ```rust
   /// Extended dashboard update type
   pub enum ExtendedDashboardUpdate {
       /// Standard dashboard update
       Standard(DashboardUpdate),
       /// Custom data update
       CustomUpdate {
           data: CustomData,
           timestamp: DateTime<Utc>,
       },
   }
   ```

2. **Create Extended Dashboard Service**

   ```rust
   /// Extended dashboard service
   pub struct ExtendedDashboardService {
       /// Inner dashboard service
       inner: Arc<dyn DashboardService>,
       /// Custom data
       custom_data: RwLock<Option<CustomData>>,
       /// Update sender
       update_sender: mpsc::Sender<ExtendedDashboardUpdate>,
   }
   
   impl ExtendedDashboardService {
       /// Update custom data
       pub async fn update_custom_data(&self, data: CustomData) -> Result<()> {
           // Update custom data
           *self.custom_data.write().await = Some(data.clone());
           
           // Send update
           self.update_sender.send(ExtendedDashboardUpdate::CustomUpdate {
               data,
               timestamp: Utc::now(),
           }).await.map_err(|e| anyhow::anyhow!("Failed to send update: {}", e))?;
           
           Ok(())
       }
   }
   ```

## Integration Testing Standards

When testing UI components with the dashboard core:

1. **Mock Dashboard Service**

   ```rust
   /// Mock dashboard service for testing
   struct MockDashboardService {
       data: RwLock<DashboardData>,
   }
   
   #[async_trait]
   impl DashboardService for MockDashboardService {
       async fn get_dashboard_data(&self) -> Result<DashboardData> {
           Ok(self.data.read().await.clone())
       }
       
       // Implement other methods
   }
   ```

2. **Test Widgets with Mock Data**

   ```rust
   #[test]
   fn test_metrics_widget_renders_correctly() {
       // Create mock metrics
       let mut counters = HashMap::new();
       counters.insert("test.counter".to_string(), 42);
       
       let metrics = MetricsSnapshot {
           values: HashMap::new(),
           counters,
           gauges: HashMap::new(),
           histograms: HashMap::new(),
       };
       
       // Create widget
       let widget = MetricsWidget::new(&metrics, "Test Metrics");
       
       // Render to buffer
       let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
       widget.render(&mut buffer, Rect::new(0, 0, 80, 24));
       
       // Verify rendering
       assert!(buffer.content.iter().any(|cell| cell.symbol == "42"));
   }
   ```

3. **Integration Test App State Updates**

   ```rust
   #[tokio::test]
   async fn test_app_updates_from_dashboard_service() {
       // Create mock service
       let service = Arc::new(MockDashboardService::new());
       
       // Create test data
       let test_data = create_test_dashboard_data();
       
       // Update service data
       service.update_mock_data(test_data.clone()).await;
       
       // Create app
       let mut app = App::new(service);
       
       // Trigger update
       app.refresh().await.unwrap();
       
       // Verify app state
       assert!(app.dashboard_data().is_some());
       assert_eq!(app.dashboard_data().unwrap().system.cpu_usage, test_data.system.cpu_usage);
   }
   ```

## UI Integration Best Practices

1. **Lazy Loading**

   Only load data when it will be displayed:

   ```rust
   impl App {
       /// Load data for the selected tab
       pub async fn load_tab_data(&mut self) -> Result<()> {
           match self.selected_tab {
               0 => self.load_overview_data().await?,
               1 => self.load_system_data().await?,
               2 => self.load_protocol_data().await?,
               // Other tabs
               _ => {}
           }
           Ok(())
       }
   }
   ```

2. **Progressive Enhancement**

   Display what's available, enhance as more data arrives:

   ```rust
   fn draw_metrics_tab(f: &mut Frame, app: &App, area: Rect) {
       let data = app.dashboard_data();
       
       // Create layout
       let chunks = Layout::default()
           .direction(Direction::Vertical)
           .constraints([
               Constraint::Percentage(50),
               Constraint::Percentage(50),
           ])
           .split(area);
       
       // Always render the metrics widget
       let metrics = data.map(|d| &d.metrics)
           .unwrap_or_else(|| &EMPTY_METRICS);
       
       let metrics_widget = MetricsWidget::new(metrics, "System Metrics");
       metrics_widget.render(f, chunks[0]);
       
       // Only render history if available
       if let Some(history) = app.get_metric_history("system.cpu") {
           let chart = ChartWidget::new(history, "CPU History");
           chart.render(f, chunks[1]);
       } else {
           draw_no_history(f, chunks[1]);
       }
   }
   ```

3. **Optimistic Updates**

   Update UI immediately, then confirm with backend:

   ```rust
   impl App {
       /// Acknowledge an alert with optimistic update
       pub async fn acknowledge_alert(&mut self, alert_id: &str, user: &str) -> Result<()> {
           // Update local state optimistically
           if let Some(data) = &mut self.dashboard_data {
               for alert in &mut data.alerts.active {
                   if alert.id == alert_id {
                       alert.acknowledged = true;
                       alert.acknowledged_at = Some(Utc::now());
                       alert.acknowledged_by = Some(user.to_string());
                       break;
                   }
               }
           }
           
           // Send update to service
           match self.dashboard_service.acknowledge_alert(alert_id, user).await {
               Ok(_) => Ok(()),
               Err(e) => {
                   // Revert optimistic update
                   self.refresh().await?;
                   Err(e)
               }
           }
       }
   }
   ```

4. **Throttled Updates**

   Throttle high-frequency updates to maintain UI responsiveness:

   ```rust
   impl App {
       /// Process dashboard updates with throttling
       pub fn process_updates(&mut self) {
           // Check if throttle interval has elapsed
           let now = Instant::now();
           if now.duration_since(self.last_update) < Duration::from_millis(100) {
               // Queue update for later
               self.has_pending_updates = true;
               return;
           }
           
           // Process all pending updates
           while let Ok(update) = self.update_rx.try_recv() {
               self.handle_update(update);
           }
           
           // Reset state
           self.last_update = now;
           self.has_pending_updates = false;
       }
   }
   ```

## Implementation Checklist

When integrating new UI components with the dashboard core:

1. **Data Requirements**
   - [ ] Identify required metrics/data
   - [ ] Determine update frequency
   - [ ] Define error states

2. **Component Design**
   - [ ] Create widget struct
   - [ ] Implement rendering
   - [ ] Add configuration options
   - [ ] Handle error states

3. **Integration Points**
   - [ ] Add tab to dashboard (if needed)
   - [ ] Connect data sources
   - [ ] Implement update handling
   - [ ] Add configuration options

4. **Testing**
   - [ ] Unit test with mock data
   - [ ] Visual test for layout
   - [ ] Integration test with dashboard
   - [ ] Performance test with high-frequency updates 