import React from 'react'

interface AuroraLogoProps {
  className?: string
  size?: number
}

/**
 * Aurora Logo SVG Component
 * Stylized 'A' with aurora waves
 */
export function AuroraLogo({ className = '', size = 80 }: AuroraLogoProps) {
  return (
    <svg
      className={`${className}`}
      style={{ width: size, height: size }}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="auroraGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" style={{ stopColor: '#62d8f3' }} />
          <stop offset="50%" style={{ stopColor: '#a78bfa' }} />
          <stop offset="100%" style={{ stopColor: '#f472b6' }} />
        </linearGradient>
        <filter id="glow">
          <feGaussianBlur stdDeviation="2" result="coloredBlur" />
          <feMerge>
            <feMergeNode in="coloredBlur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
      {/* Stylized 'A' for Aurora */}
      <path
        d="M50 15 L20 85 L35 85 L42 65 L58 65 L65 85 L80 85 L50 15Z M50 35 L55 55 L45 55 L50 35Z"
        fill="url(#auroraGrad)"
        filter="url(#glow)"
      />
      {/* Aurora wave accent */}
      <path
        d="M15 50 Q30 40, 50 50 T85 50"
        stroke="url(#auroraGrad)"
        strokeWidth="3"
        fill="none"
        opacity="0.6"
      >
        <animate
          attributeName="d"
          values="M15 50 Q30 40, 50 50 T85 50;M15 50 Q30 60, 50 50 T85 50;M15 50 Q30 40, 50 50 T85 50"
          dur="3s"
          repeatCount="indefinite"
        />
      </path>
    </svg>
  )
}

/**
 * Logo container with rotating ring effect
 */
export function LogoContainer({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`relative w-[120px] h-[120px] flex items-center justify-center ${className}`}>
      {/* Rotating ring */}
      <div className="absolute inset-0 rounded-full border-2 border-transparent border-t-aurora-cyan/80 border-r-aurora-purple/60 animate-spin-slow">
        <div className="absolute inset-[-2px] rounded-full border-2 border-transparent border-b-aurora-pink/60 border-l-aurora-cyan/40 animate-[spin_2s_linear_infinite_reverse]" />
      </div>
      {children}
    </div>
  )
}

/**
 * Error icon SVG
 */
export function ErrorIcon({ className = '', size = 80 }: AuroraLogoProps) {
  return (
    <svg
      className={`${className}`}
      style={{ width: size, height: size }}
      viewBox="0 0 100 100"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <defs>
        <linearGradient id="errorGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" style={{ stopColor: '#f36262' }} />
          <stop offset="50%" style={{ stopColor: '#f472b6' }} />
          <stop offset="100%" style={{ stopColor: '#a78bfa' }} />
        </linearGradient>
        <filter id="errorGlow">
          <feGaussianBlur stdDeviation="2" result="coloredBlur" />
          <feMerge>
            <feMergeNode in="coloredBlur" />
            <feMergeNode in="SourceGraphic" />
          </feMerge>
        </filter>
      </defs>
      {/* Warning triangle */}
      <path
        d="M50 15 L85 80 L15 80 Z"
        stroke="url(#errorGrad)"
        strokeWidth="4"
        fill="none"
        filter="url(#errorGlow)"
        strokeLinejoin="round"
      />
      {/* Exclamation mark */}
      <line
        x1="50"
        y1="40"
        x2="50"
        y2="55"
        stroke="url(#errorGrad)"
        strokeWidth="5"
        strokeLinecap="round"
      />
      <circle cx="50" cy="67" r="3" fill="url(#errorGrad)" />
    </svg>
  )
}
