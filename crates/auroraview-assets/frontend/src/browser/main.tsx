import React from 'react'
import ReactDOM from 'react-dom/client'
import { Browser } from './Browser'
import '@/styles/globals.css'
import './browser.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Browser />
  </React.StrictMode>
)
