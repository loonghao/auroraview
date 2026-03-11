import react from '@vitejs/plugin-react'
import { loadEnv } from 'vite'


function applySentryEnv(): void {
  const sentryEnv = loadEnv('sentry', process.cwd(), '')

  for (const [key, value] of Object.entries(sentryEnv)) {
    if (process.env[key] === undefined) {
      process.env[key] = value
    }
  }
}

applySentryEnv()

// https://vite.dev/config/
export default {
  plugins: [react()],
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
  },
  resolve: {
    dedupe: ['react', 'react-dom'],
  },
  optimizeDeps: {
    include: ['react', 'react-dom'],
  },
}
