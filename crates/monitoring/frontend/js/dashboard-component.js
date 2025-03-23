/**
 * Dashboard Component
 * 
 * This module demonstrates how to use the WebSocket client to display
 * real-time monitoring data in a dashboard.
 */

import DashboardClient from './websocket-client.js';
import { ref, onMounted, onUnmounted } from 'vue';

/**
 * Dashboard component for displaying real-time metrics
 * 
 * @param {Object} options - Component options
 * @param {string} options.serverUrl - WebSocket server URL
 * @param {Array} options.components - Component IDs to subscribe to
 * @returns {Object} - Vue component
 */
export function useDashboard(options = {}) {
  const {
    serverUrl = 'ws://localhost:8765/ws',
    components = ['system_cpu', 'system_memory'],
  } = options;
  
  // State refs
  const isConnected = ref(false);
  const isLoading = ref(true);
  const error = ref(null);
  const metrics = ref({});
  const updates = ref([]);
  const connectionStatus = ref('disconnected');
  
  // Create the WebSocket client
  const client = new DashboardClient(serverUrl, {
    onConnect: () => {
      isConnected.value = true;
      connectionStatus.value = 'connected';
      isLoading.value = false;
      
      // Subscribe to each component
      components.forEach(componentId => {
        client.subscribe(componentId);
      });
    },
    
    onDisconnect: () => {
      isConnected.value = false;
      connectionStatus.value = 'disconnected';
    },
    
    onError: (err) => {
      error.value = err;
      connectionStatus.value = 'error';
    },
    
    onMessage: (message) => {
      // Handle different message types
      if (message.type === 'update' && message.update) {
        const { component_id, data, timestamp } = message.update;
        
        // Store the latest metric data for this component
        metrics.value[component_id] = {
          data,
          timestamp,
          updatedAt: new Date(),
        };
        
        // Add to updates history (limited to 100 entries)
        updates.value.unshift({
          component_id,
          timestamp,
          time: new Date(),
        });
        
        // Limit updates array
        if (updates.value.length > 100) {
          updates.value = updates.value.slice(0, 100);
        }
      } else if (message.type === 'subscription_confirmed') {
        console.log(`Subscription confirmed for ${message.component_id}`);
      }
    },
  });
  
  // Connect to the WebSocket server when the component is mounted
  onMounted(() => {
    connectionStatus.value = 'connecting';
    client.connect().catch(err => {
      error.value = err;
      connectionStatus.value = 'error';
      isLoading.value = false;
    });
  });
  
  // Disconnect when the component is unmounted
  onUnmounted(() => {
    client.disconnect();
  });
  
  // Utility functions
  const reconnect = () => {
    connectionStatus.value = 'connecting';
    error.value = null;
    client.connect().catch(err => {
      error.value = err;
      connectionStatus.value = 'error';
    });
  };
  
  const subscribe = (componentId) => {
    client.subscribe(componentId);
  };
  
  const unsubscribe = (componentId) => {
    client.unsubscribe(componentId);
  };
  
  return {
    // State
    isConnected,
    isLoading,
    error,
    metrics,
    updates,
    connectionStatus,
    
    // Actions
    reconnect,
    subscribe,
    unsubscribe,
  };
}

/**
 * Format a metric value for display
 * @param {*} value - The metric value
 * @param {string} type - The type of metric
 * @returns {string} - Formatted value
 */
export function formatMetric(value, type = 'default') {
  if (value === undefined || value === null) {
    return 'N/A';
  }
  
  switch (type) {
    case 'percentage':
      return `${value.toFixed(2)}%`;
    
    case 'bytes':
      return formatBytes(value);
    
    case 'duration':
      return formatDuration(value);
    
    case 'number':
      return value.toLocaleString();
    
    default:
      return String(value);
  }
}

/**
 * Format bytes to a human-readable string
 * @param {number} bytes - Bytes value
 * @returns {string} - Formatted string
 */
function formatBytes(bytes) {
  if (bytes === 0) return '0 B';
  
  const units = ['B', 'KB', 'MB', 'GB', 'TB', 'PB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  
  return `${(bytes / Math.pow(1024, i)).toFixed(2)} ${units[i]}`;
}

/**
 * Format duration in milliseconds to a human-readable string
 * @param {number} ms - Duration in milliseconds
 * @returns {string} - Formatted string
 */
function formatDuration(ms) {
  if (ms < 1000) {
    return `${ms.toFixed(2)}ms`;
  }
  
  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(2)}s`;
  }
  
  const minutes = seconds / 60;
  if (minutes < 60) {
    return `${minutes.toFixed(2)}m`;
  }
  
  const hours = minutes / 60;
  return `${hours.toFixed(2)}h`;
} 