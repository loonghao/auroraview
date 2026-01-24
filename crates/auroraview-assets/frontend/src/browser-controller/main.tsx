import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserController } from './BrowserController'
import '@/styles/globals.css'
import './controller.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserController />
  </React.StrictMode>
)
