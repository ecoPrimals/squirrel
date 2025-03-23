import { createApp } from 'vue';
import DashboardPanel from '../components/DashboardPanel.vue';

/**
 * Initialize the dashboard application
 * @param {Object} options - Configuration options
 * @param {string} options.mountPoint - Element ID to mount the app
 * @param {string} options.serverUrl - WebSocket server URL
 * @param {Array} options.components - Component IDs to subscribe to
 */
export function initDashboard(options = {}) {
  const {
    mountPoint = '#dashboard-app',
    serverUrl = 'ws://localhost:8765/ws',
    components = ['system_cpu', 'system_memory', 'disk_usage', 'network_traffic', 'health_status'],
  } = options;
  
  // Create the Vue app
  const app = createApp({
    components: {
      DashboardPanel
    },
    template: `
      <div class="dashboard-container">
        <header class="dashboard-header">
          <h1>Monitoring Dashboard</h1>
          <div class="dashboard-controls">
            <button @click="addComponent" class="control-button">Add Component</button>
          </div>
        </header>
        
        <DashboardPanel 
          :serverUrl="serverUrl" 
          :components="activeComponents" 
        />
      </div>
    `,
    setup() {
      const activeComponents = ref([...components]);
      
      // Function to add a new component to monitor
      const addComponent = () => {
        // In a real app, this would show a dialog to select components
        const availableComponents = [
          'system_cpu',
          'system_memory',
          'disk_usage', 
          'network_traffic',
          'health_status',
          'process_stats',
          'database_performance',
          'api_metrics',
          'queue_metrics'
        ];
        
        // Filter out already active components
        const remainingComponents = availableComponents.filter(
          c => !activeComponents.value.includes(c)
        );
        
        if (remainingComponents.length > 0) {
          // Add the first available component
          activeComponents.value.push(remainingComponents[0]);
        } else {
          alert('All available components are already being monitored');
        }
      };
      
      return {
        serverUrl,
        activeComponents,
        addComponent
      };
    }
  });
  
  // Mount the app
  app.mount(mountPoint);
  
  return app;
}

// Auto-initialize if running in browser context
if (typeof window !== 'undefined') {
  // Wait for DOM to be ready
  document.addEventListener('DOMContentLoaded', () => {
    // Look for configuration in global window object or data attributes
    const configEl = document.getElementById('dashboard-config');
    const config = configEl ? JSON.parse(configEl.textContent) : {};
    
    // Initialize with any provided configuration
    initDashboard({
      ...config,
      // Get WebSocket URL from a meta tag or use default
      serverUrl: document.querySelector('meta[name="websocket-url"]')?.content || 
                config.serverUrl || 
                `ws://${window.location.hostname}:8765/ws`
    });
  });
} 