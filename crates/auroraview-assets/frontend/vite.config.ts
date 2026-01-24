import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { resolve } from 'path'

// Multi-page application configuration
// Each page is a standalone HTML entry with its own bundle
export default defineConfig({
  plugins: [react()],
  root: 'src',
  base: './',
  build: {
    outDir: '../dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        loading: resolve(__dirname, 'src/loading/index.html'),
        error: resolve(__dirname, 'src/error/index.html'),
        browser: resolve(__dirname, 'src/browser/index.html'),
        'browser-controller': resolve(__dirname, 'src/browser-controller/index.html'),
      },
      output: {
        // Preserve folder structure
        entryFileNames: '[name]/[name].[hash].js',
        chunkFileNames: 'chunks/[name].[hash].js',
        assetFileNames: 'assets/[name].[hash][extname]',
      },
    },
    // Inline small assets for better performance
    assetsInlineLimit: 4096,
    // Generate source maps for debugging
    sourcemap: true,
    // Minify for production (use esbuild, faster than terser)
    minify: 'esbuild',
  },
  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      '@components': resolve(__dirname, 'src/components'),
      '@styles': resolve(__dirname, 'src/styles'),
      '@utils': resolve(__dirname, 'src/utils'),
    },
  },
  // Development server for preview
  server: {
    port: 5173,
    open: '/loading/index.html',
  },
})
