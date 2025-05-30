// Import Jest DOM for DOM testing utilities
require('@testing-library/jest-dom');

// Ensure jest-dom matchers are added to Jest's expect
const matchers = require('@testing-library/jest-dom/matchers');
expect.extend(matchers);

// Suppress React act warnings
const originalConsoleError = console.error;
console.error = (...args) => {
  if (/Warning: An update to .+ inside a test was not wrapped in act/.test(args[0])) {
    return;
  }
  originalConsoleError(...args);
};

// Mock the Tauri API
jest.mock('@tauri-apps/api/tauri', () => ({
  invoke: jest.fn().mockImplementation((command, args) => {
    console.log(`Mocked invoke call: ${command}`, args);
    return Promise.resolve(null);
  }),
}));

jest.mock('@tauri-apps/api/event', () => ({
  listen: jest.fn().mockImplementation(() => {
    return Promise.resolve(() => {});
  }),
}));

jest.mock('@tauri-apps/api/dialog', () => ({
  open: jest.fn().mockImplementation(() => {
    return Promise.resolve('/path/to/file');
  }),
}));

// Mock Chart.js to avoid issues with canvas rendering
jest.mock('react-chartjs-2', () => ({
  Line: () => '<div data-testid="line-chart">Line Chart</div>',
  Bar: () => '<div data-testid="bar-chart">Bar Chart</div>',
  Pie: () => '<div data-testid="pie-chart">Pie Chart</div>',
  Doughnut: () => '<div data-testid="doughnut-chart">Doughnut Chart</div>',
}));

// Mock for ChartJS registration
jest.mock('chart.js', () => ({
  Chart: {
    register: jest.fn(),
  },
  CategoryScale: jest.fn(),
  LinearScale: jest.fn(),
  PointElement: jest.fn(),
  LineElement: jest.fn(),
  BarElement: jest.fn(),
  Title: jest.fn(),
  Tooltip: jest.fn(),
  Legend: jest.fn(),
  TimeScale: jest.fn(),
}));

// Mock for date-fns adapter
jest.mock('chartjs-adapter-date-fns', () => ({}));

// Mock the EventEmitter
jest.mock('events', () => {
  class MockEventEmitter {
    constructor() {
      this.events = {};
    }
    
    on = jest.fn().mockImplementation((event, listener) => {
      if (!this.events[event]) {
        this.events[event] = [];
      }
      this.events[event].push(listener);
      return this;
    });
    
    off = jest.fn().mockImplementation((event, listener) => {
      if (!this.events[event]) return this;
      const idx = this.events[event].indexOf(listener);
      if (idx !== -1) {
        this.events[event].splice(idx, 1);
      }
      return this;
    });
    
    emit = jest.fn().mockImplementation((event, ...args) => {
      if (!this.events[event]) return false;
      this.events[event].forEach(listener => listener(...args));
      return true;
    });
    
    once = jest.fn();
    removeAllListeners = jest.fn().mockImplementation((event) => {
      if (event) {
        delete this.events[event];
      } else {
        this.events = {};
      }
      return this;
    });
    addListener = jest.fn();
    removeListener = jest.fn();
  }
  
  return { 
    EventEmitter: MockEventEmitter,
  };
});

// Setup global mocks for UUID generation
jest.mock('uuid', () => ({
  v4: jest.fn().mockReturnValue('test-uuid'),
}));

// Set up global test variables
global.ResizeObserver = jest.fn().mockImplementation(() => ({
  observe: jest.fn(),
  unobserve: jest.fn(),
  disconnect: jest.fn(),
}));

// Mock window.matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: jest.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: jest.fn(),
    removeListener: jest.fn(),
    addEventListener: jest.fn(),
    removeEventListener: jest.fn(),
    dispatchEvent: jest.fn(),
  })),
});

// Set up custom jest matchers
expect.extend({
  toBeInRange(received, floor, ceiling) {
    const pass = received >= floor && received <= ceiling;
    if (pass) {
      return {
        message: () => `expected ${received} not to be within range ${floor} - ${ceiling}`,
        pass: true,
      };
    } else {
      return {
        message: () => `expected ${received} to be within range ${floor} - ${ceiling}`,
        pass: false,
      };
    }
  },
});

// Clean up mocks after each test
afterEach(() => {
  jest.clearAllMocks();
});

// Register central Tauri API mocks
try {
  const { registerTauriAPIMocks } = require('./crates/ui-tauri-react/src/test-utils/tauri-api-mock-system');
  if (typeof registerTauriAPIMocks === 'function') {
    registerTauriAPIMocks();
    console.log('✅ Tauri API mocks registered successfully');
  } else {
    console.warn('⚠️ registerTauriAPIMocks is not a function');
  }
} catch (error) {
  console.warn('⚠️ Could not register Tauri API mocks:', error.message);
}

// Suppress React 18 act() warnings when necessary
const originalError = console.error;
console.error = function(...args) {
  if (
    typeof args[0] === 'string' &&
    args[0].includes('Warning: An update to') &&
    args[0].includes('inside a test was not wrapped in act')
  ) {
    // Filter out some act() warnings that may occur due to async events
    return;
  }
  return originalError.apply(console, args);
}; 