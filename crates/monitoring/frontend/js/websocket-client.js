/**
 * WebSocket Client for Dashboard
 * 
 * This module provides functionality to connect to the dashboard WebSocket server,
 * subscribe to component updates, and handle incoming messages including batched
 * and compressed messages.
 */

// Import pako for gzip decompression
import pako from 'pako';

/**
 * Dashboard WebSocket client class
 */
class DashboardClient {
  /**
   * Create a new dashboard client
   * @param {string} url - WebSocket server URL
   * @param {Object} options - Client options
   * @param {Function} options.onMessage - Callback for messages
   * @param {Function} options.onConnect - Callback when connected
   * @param {Function} options.onDisconnect - Callback when disconnected
   * @param {Function} options.onError - Callback for errors
   * @param {number} options.reconnectInterval - Reconnect interval in ms
   */
  constructor(url, options = {}) {
    this.url = url;
    this.socket = null;
    this.connected = false;
    this.reconnecting = false;
    this.subscriptions = new Set();
    
    // Set default options
    this.options = {
      onMessage: () => {},
      onConnect: () => {},
      onDisconnect: () => {},
      onError: () => {},
      reconnectInterval: 5000,
      ...options
    };
    
    // Bind methods
    this.connect = this.connect.bind(this);
    this.disconnect = this.disconnect.bind(this);
    this.subscribe = this.subscribe.bind(this);
    this.unsubscribe = this.unsubscribe.bind(this);
    this.send = this.send.bind(this);
    this.handleMessage = this.handleMessage.bind(this);
    this.decompressMessage = this.decompressMessage.bind(this);
  }
  
  /**
   * Connect to the WebSocket server
   * @returns {Promise} - Resolves when connected
   */
  connect() {
    return new Promise((resolve, reject) => {
      if (this.socket && this.connected) {
        resolve();
        return;
      }
      
      // Create a new WebSocket connection
      this.socket = new WebSocket(this.url);
      
      // Set up event handlers
      this.socket.onopen = () => {
        console.log('Connected to dashboard WebSocket server');
        this.connected = true;
        this.reconnecting = false;
        
        // Resubscribe to all previous subscriptions
        this.subscriptions.forEach(componentId => {
          this.send({
            type: 'subscribe',
            componentId
          });
        });
        
        this.options.onConnect();
        resolve();
      };
      
      this.socket.onclose = (event) => {
        console.log('Disconnected from dashboard WebSocket server', event);
        this.connected = false;
        this.options.onDisconnect(event);
        
        // Attempt to reconnect if not manually disconnected
        if (!this.reconnecting) {
          this.reconnecting = true;
          setTimeout(() => this.connect(), this.options.reconnectInterval);
        }
      };
      
      this.socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        this.options.onError(error);
        if (!this.connected) {
          reject(error);
        }
      };
      
      this.socket.onmessage = (event) => {
        this.handleMessage(event.data);
      };
    });
  }
  
  /**
   * Disconnect from the WebSocket server
   */
  disconnect() {
    this.reconnecting = true; // Prevent auto-reconnect
    if (this.socket) {
      this.socket.close();
      this.socket = null;
    }
    this.connected = false;
  }
  
  /**
   * Subscribe to updates for a specific component
   * @param {string} componentId - Component ID to subscribe to
   */
  subscribe(componentId) {
    if (!componentId) return;
    
    this.subscriptions.add(componentId);
    
    if (this.connected) {
      this.send({
        type: 'subscribe',
        componentId
      });
    }
  }
  
  /**
   * Unsubscribe from updates for a specific component
   * @param {string} componentId - Component ID to unsubscribe from
   */
  unsubscribe(componentId) {
    if (!componentId) return;
    
    this.subscriptions.delete(componentId);
    
    if (this.connected) {
      this.send({
        type: 'unsubscribe',
        componentId
      });
    }
  }
  
  /**
   * Send a message to the WebSocket server
   * @param {Object} message - Message to send
   */
  send(message) {
    if (!this.connected || !this.socket) {
      console.warn('Cannot send message, not connected');
      return;
    }
    
    this.socket.send(JSON.stringify(message));
  }
  
  /**
   * Decompress a base64 encoded gzip message
   * @param {string} compressedData - Base64 encoded compressed data
   * @returns {Object} - Decompressed JSON object
   */
  decompressMessage(compressedData) {
    try {
      // Decode base64 string to binary data
      const binaryString = atob(compressedData);
      const len = binaryString.length;
      const bytes = new Uint8Array(len);
      
      // Convert binary string to byte array
      for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      
      // Decompress using pako
      const decompressed = pako.inflate(bytes);
      
      // Convert to string and parse JSON
      const decoder = new TextDecoder('utf-8');
      const jsonStr = decoder.decode(decompressed);
      return JSON.parse(jsonStr);
    } catch (err) {
      console.error('Error decompressing message:', err);
      return null;
    }
  }
  
  /**
   * Handle incoming WebSocket messages
   * @param {string} data - Raw message data
   */
  handleMessage(data) {
    try {
      // Parse the JSON message
      const message = JSON.parse(data);
      
      // Handle different message types
      switch (message.type) {
        case 'compressed':
          if (message.compressed && message.compressed_data) {
            // Decompress the message
            const decompressedMessage = this.decompressMessage(message.compressed_data);
            if (decompressedMessage) {
              // Forward the decompressed message to handler
              this.options.onMessage(decompressedMessage);
            }
          }
          break;
          
        case 'batch':
          // Handle batch updates
          if (message.updates && Array.isArray(message.updates)) {
            // Process each update in the batch
            message.updates.forEach(update => {
              this.options.onMessage({
                type: 'update',
                update
              });
            });
          }
          break;
          
        default:
          // Forward any other message directly
          this.options.onMessage(message);
          break;
      }
    } catch (err) {
      console.error('Error handling message:', err, data);
    }
  }
}

export default DashboardClient; 