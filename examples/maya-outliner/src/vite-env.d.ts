/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue'
  const component: DefineComponent<{}, {}, any>
  export default component
}

// AuroraView uses CustomEvent for IPC
// No need for window.auroraview - use window.dispatchEvent() and window.addEventListener()

