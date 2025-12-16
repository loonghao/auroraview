import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    globals: true,
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    exclude: ['tests/**/*.e2e.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'json-summary', 'html', 'lcov'],
      include: ['src/core/**/*.ts'],
      exclude: [
        'src/**/*.d.ts',
        'src/inject/**',
        'src/adapters/**', // Adapters require React/Vue, tested separately
        'src/index.ts', // Re-exports only
      ],
      thresholds: {
        lines: 70,
        functions: 90,
        branches: 60,
        statements: 70,
      },
    },
    reporters: ['default', 'json'],
    outputFile: {
      json: 'test-results/results.json',
    },
  },
});
