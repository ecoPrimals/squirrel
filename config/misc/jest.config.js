/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',
  setupFilesAfterEnv: [
    '<rootDir>/jest.setup.js',
    '<rootDir>/crates/ui-tauri-react/src/test-utils/jest.setup.ts'
  ],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
    '\\.(css|less|scss|sass)$': 'identity-obj-proxy',
  },
  transform: {
    '^.+\\.(ts|tsx)$': ['ts-jest', { tsconfig: 'tsconfig.test.json', isolatedModules: true }],
  },
  testPathIgnorePatterns: [
    '/node_modules/',
    '/crates/(?!ui-tauri-react)',
    '/src/services/mocks/'
  ],
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  collectCoverageFrom: [
    'src/**/*.{ts,tsx}',
    'crates/ui-tauri-react/src/**/*.{ts,tsx}',
    '!src/**/*.d.ts',
    '!src/@types/**/*.ts',
    '!src/services/mocks/**/*.ts',
  ]
}; 