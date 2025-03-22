<template>
  <div class="dashboard-panel">
    <div class="connection-status" :class="connectionStatus">
      <span class="status-indicator"></span>
      <span class="status-text">{{ connectionStatusText }}</span>
      <button v-if="connectionStatus === 'error' || connectionStatus === 'disconnected'" 
              @click="reconnect" 
              class="reconnect-button">
        Reconnect
      </button>
    </div>
    
    <div v-if="isLoading" class="loading">
      <span class="loading-spinner"></span>
      <span>Loading dashboard data...</span>
    </div>
    
    <div v-else-if="error" class="error-panel">
      <h3>Connection Error</h3>
      <p>{{ error.message || 'Failed to connect to monitoring server' }}</p>
    </div>
    
    <div v-else class="metrics-grid">
      <div v-for="(metric, componentId) in metrics" 
           :key="componentId" 
           class="metric-card">
        <h3 class="component-title">{{ formatComponentName(componentId) }}</h3>
        
        <div class="metric-value" v-if="metric.data">
          <template v-if="isSystemMetric(componentId)">
            <div class="system-metrics">
              <div v-if="componentId === 'system_cpu'" class="cpu-usage">
                <div class="gauge">
                  <div class="gauge-fill" :style="{ width: `${metric.data.usage || 0}%` }"></div>
                </div>
                <div class="gauge-value">{{ formatMetric(metric.data.usage, 'percentage') }}</div>
                <div class="metric-details">
                  <div>Cores: {{ metric.data.cores || 'N/A' }}</div>
                  <div>Load Avg: {{ metric.data.load_avg?.[0] || 'N/A' }}</div>
                </div>
              </div>
              
              <div v-else-if="componentId === 'system_memory'" class="memory-usage">
                <div class="gauge">
                  <div class="gauge-fill" :style="{ width: `${(metric.data.used / metric.data.total) * 100 || 0}%` }"></div>
                </div>
                <div class="gauge-label">
                  <span>{{ formatMetric(metric.data.used, 'bytes') }} / {{ formatMetric(metric.data.total, 'bytes') }}</span>
                </div>
                <div class="metric-details">
                  <div>Free: {{ formatMetric(metric.data.free, 'bytes') }}</div>
                  <div>Swap: {{ formatMetric(metric.data.swap_used, 'bytes') }} / {{ formatMetric(metric.data.swap_total, 'bytes') }}</div>
                </div>
              </div>
              
              <div v-else-if="componentId === 'disk_usage'" class="disk-usage">
                <div v-for="(disk, name) in metric.data.disks" :key="name" class="disk-item">
                  <div class="disk-name">{{ name }}</div>
                  <div class="gauge">
                    <div class="gauge-fill" :style="{ width: `${disk.used_percent || 0}%` }"></div>
                  </div>
                  <div class="gauge-value">{{ formatMetric(disk.used_percent, 'percentage') }}</div>
                  <div class="gauge-label">
                    {{ formatMetric(disk.used, 'bytes') }} / {{ formatMetric(disk.total, 'bytes') }}
                  </div>
                </div>
              </div>
              
              <div v-else-if="componentId === 'network_traffic'" class="network-usage">
                <div class="network-stats">
                  <div class="stat">
                    <div class="stat-label">Rx</div>
                    <div class="stat-value">{{ formatMetric(metric.data.rx_rate, 'bytes') }}/s</div>
                  </div>
                  <div class="stat">
                    <div class="stat-label">Tx</div>
                    <div class="stat-value">{{ formatMetric(metric.data.tx_rate, 'bytes') }}/s</div>
                  </div>
                </div>
                <div class="metric-details">
                  <div>Total Rx: {{ formatMetric(metric.data.rx_total, 'bytes') }}</div>
                  <div>Total Tx: {{ formatMetric(metric.data.tx_total, 'bytes') }}</div>
                </div>
              </div>
              
              <div v-else class="generic-metric">
                <pre>{{ JSON.stringify(metric.data, null, 2) }}</pre>
              </div>
            </div>
          </template>
          <template v-else-if="componentId === 'health_status'">
            <div class="health-status">
              <div class="status-indicator" :class="getHealthClass(metric.data.status)"></div>
              <div class="status-text">{{ metric.data.status }}</div>
              <div class="components-health">
                <div v-for="(component, name) in metric.data.components" 
                     :key="name"
                     class="health-component"
                     :class="getHealthClass(component.status)">
                  <span class="component-name">{{ name }}</span>
                  <span class="component-status">{{ component.status }}</span>
                </div>
              </div>
            </div>
          </template>
          <template v-else>
            <pre class="json-data">{{ JSON.stringify(metric.data, null, 2) }}</pre>
          </template>
        </div>
        
        <div class="metric-footer">
          <span class="timestamp">{{ formatTime(metric.updatedAt) }}</span>
          <button @click="unsubscribe(componentId)" class="unsubscribe-button">
            Unsubscribe
          </button>
        </div>
      </div>
    </div>
    
    <div class="updates-panel">
      <h3>Recent Updates</h3>
      <div class="updates-list">
        <div v-for="(update, index) in updates" 
             :key="index"
             class="update-item">
          <span class="update-component">{{ formatComponentName(update.component_id) }}</span>
          <span class="update-time">{{ formatTime(update.time) }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { useDashboard, formatMetric } from '../js/dashboard-component.js';

export default {
  name: 'DashboardPanel',
  props: {
    serverUrl: {
      type: String,
      default: 'ws://localhost:8765/ws'
    },
    components: {
      type: Array,
      default: () => ['system_cpu', 'system_memory', 'disk_usage', 'network_traffic', 'health_status']
    }
  },
  
  setup(props) {
    const {
      isConnected,
      isLoading,
      error,
      metrics,
      updates,
      connectionStatus,
      reconnect,
      subscribe,
      unsubscribe
    } = useDashboard({
      serverUrl: props.serverUrl,
      components: props.components
    });
    
    const connectionStatusText = computed(() => {
      switch (connectionStatus.value) {
        case 'connected': return 'Connected';
        case 'connecting': return 'Connecting...';
        case 'disconnected': return 'Disconnected';
        case 'error': return 'Connection Error';
        default: return 'Unknown';
      }
    });
    
    const formatComponentName = (componentId) => {
      return componentId
        .replace(/_/g, ' ')
        .replace(/\b\w/g, l => l.toUpperCase());
    };
    
    const formatTime = (timestamp) => {
      if (!timestamp) return 'N/A';
      
      const date = new Date(timestamp);
      return date.toLocaleTimeString();
    };
    
    const isSystemMetric = (componentId) => {
      return [
        'system_cpu',
        'system_memory',
        'disk_usage',
        'network_traffic'
      ].includes(componentId);
    };
    
    const getHealthClass = (status) => {
      switch (status?.toLowerCase()) {
        case 'healthy':
          return 'health-healthy';
        case 'degraded':
          return 'health-degraded';
        case 'unhealthy':
          return 'health-unhealthy';
        default:
          return 'health-unknown';
      }
    };
    
    return {
      isConnected,
      isLoading,
      error,
      metrics,
      updates,
      connectionStatus,
      connectionStatusText,
      reconnect,
      subscribe,
      unsubscribe,
      formatMetric,
      formatComponentName,
      formatTime,
      isSystemMetric,
      getHealthClass
    };
  }
};
</script>

<style scoped>
.dashboard-panel {
  font-family: system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, 'Open Sans', 'Helvetica Neue', sans-serif;
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.connection-status {
  display: flex;
  align-items: center;
  margin-bottom: 20px;
  padding: 10px;
  border-radius: 4px;
  background-color: #f5f5f5;
}

.connection-status.connected {
  background-color: #e6f7e6;
}

.connection-status.error {
  background-color: #fce8e6;
}

.connection-status.disconnected {
  background-color: #fef6e6;
}

.status-indicator {
  display: inline-block;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  margin-right: 8px;
}

.connected .status-indicator {
  background-color: #34a853;
}

.connecting .status-indicator {
  background-color: #fbbc05;
  animation: pulse 1.5s infinite;
}

.disconnected .status-indicator {
  background-color: #fbbc05;
}

.error .status-indicator {
  background-color: #ea4335;
}

.status-text {
  font-weight: 500;
}

.reconnect-button {
  margin-left: auto;
  padding: 6px 12px;
  background-color: #4285f4;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
}

.loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px;
  background-color: #f5f5f5;
  border-radius: 8px;
}

.loading-spinner {
  display: inline-block;
  width: 30px;
  height: 30px;
  border: 3px solid rgba(0, 0, 0, 0.1);
  border-radius: 50%;
  border-top-color: #4285f4;
  animation: spin 1s linear infinite;
  margin-bottom: 16px;
}

.error-panel {
  padding: 20px;
  background-color: #fce8e6;
  border-radius: 8px;
  margin-bottom: 20px;
}

.metrics-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 20px;
  margin-bottom: 30px;
}

.metric-card {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
  overflow: hidden;
}

.component-title {
  margin: 0;
  padding: 15px;
  background-color: #f5f5f5;
  font-size: 16px;
  font-weight: 500;
}

.metric-value {
  padding: 20px;
}

.gauge {
  height: 8px;
  background-color: #e6e6e6;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 8px;
}

.gauge-fill {
  height: 100%;
  background-color: #4285f4;
  border-radius: 4px;
  transition: width 0.3s ease;
}

.gauge-value {
  font-size: 24px;
  font-weight: 500;
  margin-bottom: 10px;
}

.gauge-label {
  font-size: 14px;
  color: #666;
  margin-bottom: 10px;
}

.metric-details {
  margin-top: 10px;
  font-size: 14px;
  color: #666;
}

.metric-footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 15px;
  background-color: #f5f5f5;
  font-size: 12px;
  color: #666;
}

.unsubscribe-button {
  padding: 4px 8px;
  background-color: transparent;
  border: 1px solid #ccc;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.unsubscribe-button:hover {
  background-color: #f0f0f0;
}

.updates-panel {
  background-color: white;
  border-radius: 8px;
  box-shadow: 0 2px 10px rgba(0, 0, 0, 0.08);
  padding: 15px;
}

.updates-panel h3 {
  margin-top: 0;
  margin-bottom: 15px;
  font-size: 16px;
  font-weight: 500;
}

.updates-list {
  max-height: 200px;
  overflow-y: auto;
}

.update-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #eee;
}

.update-item:last-child {
  border-bottom: none;
}

.update-component {
  font-weight: 500;
}

.update-time {
  color: #666;
  font-size: 13px;
}

.health-status {
  padding: 15px;
}

.health-status .status-indicator {
  width: 16px;
  height: 16px;
  display: inline-block;
  margin-right: 8px;
}

.health-status .status-text {
  font-size: 18px;
  font-weight: 500;
  display: inline-block;
  margin-bottom: 15px;
}

.components-health {
  margin-top: 15px;
}

.health-component {
  display: flex;
  justify-content: space-between;
  padding: 8px 12px;
  margin-bottom: 8px;
  border-radius: 4px;
  background-color: #f5f5f5;
}

.health-healthy {
  background-color: #e6f7e6;
  color: #0d652d;
}

.health-degraded {
  background-color: #fef6e6;
  color: #b06000;
}

.health-unhealthy {
  background-color: #fce8e6;
  color: #c5221f;
}

.health-unknown {
  background-color: #f5f5f5;
  color: #666;
}

.json-data {
  background-color: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
  font-family: monospace;
  font-size: 13px;
  overflow-x: auto;
  max-height: 300px;
}

@keyframes pulse {
  0% { opacity: 0.5; }
  50% { opacity: 1; }
  100% { opacity: 0.5; }
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style> 