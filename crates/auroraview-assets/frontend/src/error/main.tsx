import React from 'react'
import ReactDOM from 'react-dom/client'
import { ErrorOverlay } from './ErrorOverlay'
import '@/styles/globals.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <ErrorOverlay />
  </React.StrictMode>
)
