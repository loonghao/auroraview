import React from 'react'
import ReactDOM from 'react-dom/client'
import { LoadingScreen } from './LoadingScreen'
import '@/styles/globals.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <LoadingScreen />
  </React.StrictMode>
)
