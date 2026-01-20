import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'

const w = window as unknown as { __AURORAVIEW_CONFIG__?: { callTimeoutMs?: number; backendFailFast?: boolean; heartbeatTimeoutMs?: number } }
if (!w.__AURORAVIEW_CONFIG__) {
  w.__AURORAVIEW_CONFIG__ = {
    callTimeoutMs: 30000,
    backendFailFast: true,
    heartbeatTimeoutMs: 15000,
  }
}

createRoot(document.getElementById('root')!).render(

  <StrictMode>
    <App />
  </StrictMode>,
)
