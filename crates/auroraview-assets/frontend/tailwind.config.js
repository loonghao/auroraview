/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./src/**/*.{html,js,ts,jsx,tsx}",
  ],
  darkMode: 'media',
  theme: {
    extend: {
      colors: {
        // Aurora theme colors
        aurora: {
          cyan: '#62d8f3',
          purple: '#a78bfa',
          pink: '#f472b6',
          red: '#f36262',
          bg: {
            dark: '#0f0c29',
            mid: '#302b63',
            light: '#24243e',
          }
        },
        // Edge-like browser theme
        browser: {
          light: {
            primary: '#ffffff',
            secondary: '#f3f3f3',
            tertiary: '#f9f9f9',
            border: '#e5e5e5',
            text: '#1a1a1a',
            'text-secondary': '#444444',
          },
          dark: {
            primary: '#202020',
            secondary: '#2d2d2d',
            tertiary: '#383838',
            border: '#3d3d3d',
            text: '#ffffff',
            'text-secondary': '#b0b0b0',
          }
        }
      },
      animation: {
        'aurora': 'aurora 8s ease-in-out infinite alternate',
        'float': 'float 3s ease-in-out infinite',
        'spin-slow': 'spin 3s linear infinite',
        'shimmer': 'shimmer 2s ease-in-out infinite',
        'pulse-glow': 'pulse-glow 2s ease-in-out infinite',
      },
      keyframes: {
        aurora: {
          '0%': { opacity: '0.5', transform: 'scale(1) translateY(0)' },
          '100%': { opacity: '1', transform: 'scale(1.1) translateY(-20px)' },
        },
        float: {
          '0%, 100%': { transform: 'translateY(0)' },
          '50%': { transform: 'translateY(-10px)' },
        },
        shimmer: {
          '0%, 100%': { opacity: '1' },
          '50%': { opacity: '0.7' },
        },
        'pulse-glow': {
          '0%, 100%': { boxShadow: '0 0 20px rgba(98, 216, 243, 0.5)' },
          '50%': { boxShadow: '0 0 40px rgba(167, 139, 250, 0.8)' },
        },
      },
      fontFamily: {
        sans: [
          '-apple-system',
          'BlinkMacSystemFont',
          'Segoe UI',
          'Roboto',
          'Oxygen',
          'Ubuntu',
          'Cantarell',
          'sans-serif',
        ],
        mono: [
          'SF Mono',
          'Monaco',
          'Cascadia Code',
          'Consolas',
          'monospace',
        ],
      },
    },
  },
  plugins: [],
}
