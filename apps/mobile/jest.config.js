const moduleNameMapper = {
  '^@config/(.*)$': '<rootDir>/config/$1',
  '^@services/(.*)$': '<rootDir>/services/$1',
  '^@hooks/(.*)$': '<rootDir>/hooks/$1',
  '^@store/(.*)$': '<rootDir>/store/$1',
  '^@providers/(.*)$': '<rootDir>/providers/$1',
  '^@lib/(.*)$': '<rootDir>/../web/lib/$1',
  '^@utils/(.*)$': '<rootDir>/utils/$1',
  '^@components/(.*)$': '<rootDir>/components/$1',
  '^@/(.*)$': '<rootDir>/$1',
};

/** Logic/unit tests: plain node environment with manual transforms. */
const unit = {
  displayName: 'unit',
  // Don't use jest-expo preset — expo-modules-core is not fully installed.
  // We configure transforms manually below.
  testEnvironment: 'node',
  setupFiles: ['<rootDir>/__mocks__/jestSetup.js'],

  transform: {
    '^.+\\.[jt]sx?$': [
      'babel-jest',
      { configFile: require('path').resolve(__dirname, 'babel.config.js') },
    ],
  },

  // Transform expo/* packages since they ship ESM
  transformIgnorePatterns: [
    'node_modules/(?!(expo|@expo|expo-notifications|expo-device|expo-constants|expo-secure-store|expo-modules-core|react-native|@react-native))',
  ],

  // Manual mocks for native/expo modules
  moduleNameMapper: {
    ...moduleNameMapper,
    // Mock assets
    '\\.(png|jpg|jpeg|gif|svg|ico|webp|ttf|otf)$': '<rootDir>/__mocks__/fileMock.js',
  },

  testMatch: ['**/__tests__/**/*.test.{ts,tsx}'],
  testPathIgnorePatterns: ['/node_modules/', '<rootDir>/__tests__/components/'],
};

/**
 * Component tests: rendered with @testing-library/react-native, which needs
 * the React Native jest environment provided by the jest-expo preset.
 */
const components = {
  displayName: 'components',
  preset: 'jest-expo',
  testMatch: ['<rootDir>/__tests__/components/**/*.test.{ts,tsx}'],
  moduleNameMapper,
};

/** @type {import('jest').Config} */
module.exports = {
  projects: [unit, components],

  collectCoverageFrom: [
    '**/*.{ts,tsx,js,jsx}',
    '!**/node_modules/**',
    '!**/__tests__/**',
    '!**/__mocks__/**',
    '!**/*.config.{js,ts}',
    '!coverage/**',
    '!**/.expo/**',
    '!path-alias.js',
  ],

  coverageThreshold: {
    global: {
      lines: 80,
      functions: 80,
      branches: 80,
      statements: 80,
    },
  },
};
