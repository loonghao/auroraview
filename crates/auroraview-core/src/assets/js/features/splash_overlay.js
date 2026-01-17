/**
 * Splash Overlay - Displays a loading overlay while the page loads
 * 
 * This script injects a splash screen overlay that covers the page during loading.
 * The overlay automatically fades out when the page is fully loaded.
 * 
 * Usage: This script should be injected via initialization_script before page load.
 */
(function() {
    'use strict';

    // Skip if already initialized
    if (window.__auroraview_splash_initialized) return;
    window.__auroraview_splash_initialized = true;

    // Create splash overlay HTML
    const splashHTML = `
        <div id="auroraview-splash-overlay" style="
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            z-index: 2147483647;
            background: linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%);
            display: flex;
            justify-content: center;
            align-items: center;
            transition: opacity 0.3s ease-out;
        ">
            <style>
                @keyframes auroraview-aurora {
                    0% { opacity: 0.5; transform: scale(1) translateY(0); }
                    100% { opacity: 1; transform: scale(1.1) translateY(-20px); }
                }
                @keyframes auroraview-float {
                    0%, 100% { transform: translateY(0); }
                    50% { transform: translateY(-10px); }
                }
                @keyframes auroraview-spin {
                    from { transform: rotate(0deg); }
                    to { transform: rotate(360deg); }
                }
                @keyframes auroraview-pulse {
                    0%, 100% { opacity: 0.4; }
                    50% { opacity: 1; }
                }
                #auroraview-splash-overlay::before {
                    content: '';
                    position: absolute;
                    top: 0;
                    left: 0;
                    right: 0;
                    bottom: 0;
                    background: 
                        radial-gradient(ellipse at 20% 80%, rgba(120, 119, 198, 0.3) 0%, transparent 50%),
                        radial-gradient(ellipse at 80% 20%, rgba(255, 119, 198, 0.2) 0%, transparent 50%),
                        radial-gradient(ellipse at 40% 40%, rgba(98, 216, 243, 0.2) 0%, transparent 50%);
                    animation: auroraview-aurora 8s ease-in-out infinite alternate;
                }
            </style>
            <div style="text-align: center; color: white; z-index: 1; position: relative;">
                <div style="width: 120px; height: 120px; margin: 0 auto 30px; position: relative; display: flex; justify-content: center; align-items: center;">
                    <!-- Aurora logo SVG -->
                    <svg style="width: 80px; height: 80px; animation: auroraview-float 3s ease-in-out infinite; filter: drop-shadow(0 0 20px rgba(98, 216, 243, 0.5));" viewBox="0 0 100 100" fill="none" xmlns="http://www.w3.org/2000/svg">
                        <defs>
                            <linearGradient id="auroraGrad" x1="0%" y1="0%" x2="100%" y2="100%">
                                <stop offset="0%" style="stop-color:#62d8f3"/>
                                <stop offset="50%" style="stop-color:#7877c6"/>
                                <stop offset="100%" style="stop-color:#ff77c6"/>
                            </linearGradient>
                        </defs>
                        <circle cx="50" cy="50" r="45" stroke="url(#auroraGrad)" stroke-width="3" fill="none"/>
                        <path d="M50 20 L65 45 L50 40 L35 45 Z" fill="url(#auroraGrad)" opacity="0.8"/>
                        <path d="M50 80 L35 55 L50 60 L65 55 Z" fill="url(#auroraGrad)" opacity="0.8"/>
                        <circle cx="50" cy="50" r="8" fill="url(#auroraGrad)"/>
                    </svg>
                    <!-- Rotating ring -->
                    <div style="
                        position: absolute;
                        width: 100%;
                        height: 100%;
                        border: 2px solid transparent;
                        border-top-color: rgba(98, 216, 243, 0.8);
                        border-right-color: rgba(120, 119, 198, 0.6);
                        border-radius: 50%;
                        animation: auroraview-spin 3s linear infinite;
                    "></div>
                </div>
                <h1 style="
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    font-size: 28px;
                    font-weight: 300;
                    letter-spacing: 4px;
                    margin-bottom: 10px;
                    background: linear-gradient(90deg, #62d8f3, #7877c6, #ff77c6);
                    -webkit-background-clip: text;
                    -webkit-text-fill-color: transparent;
                    background-clip: text;
                ">AURORAVIEW</h1>
                <p style="
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    font-size: 14px;
                    color: rgba(255, 255, 255, 0.6);
                    letter-spacing: 2px;
                    animation: auroraview-pulse 2s ease-in-out infinite;
                ">Loading...</p>
            </div>
        </div>
    `;

    // Inject splash overlay immediately
    function injectSplash() {
        // Create a temporary container
        const temp = document.createElement('div');
        temp.innerHTML = splashHTML;
        const overlay = temp.firstElementChild;
        
        // Append to body or document element
        const target = document.body || document.documentElement;
        if (target) {
            target.appendChild(overlay);
        }
    }

    // Remove splash overlay with fade animation
    function removeSplash() {
        const overlay = document.getElementById('auroraview-splash-overlay');
        if (overlay) {
            overlay.style.opacity = '0';
            setTimeout(function() {
                if (overlay.parentNode) {
                    overlay.parentNode.removeChild(overlay);
                }
            }, 300);
        }
    }

    // Inject splash as soon as possible
    if (document.body) {
        injectSplash();
    } else {
        // Wait for DOM to be ready enough to inject
        document.addEventListener('DOMContentLoaded', injectSplash, { once: true });
    }

    // Remove splash when page is fully loaded
    if (document.readyState === 'complete') {
        // Page already loaded, remove immediately (with small delay for visual)
        setTimeout(removeSplash, 100);
    } else {
        window.addEventListener('load', function() {
            // Add a small delay to ensure smooth transition
            setTimeout(removeSplash, 100);
        }, { once: true });
    }

    // Expose API for manual control
    window.__auroraview_splash = {
        show: injectSplash,
        hide: removeSplash
    };
})();
