/**
 * AuroraView Plugins - JavaScript API Bundle
 * 
 * This file loads all plugin APIs and attaches them to window.auroraview.
 * 
 * Available APIs after loading:
 *   - auroraview.fs       - File system operations
 *   - auroraview.dialog   - Native file/folder dialogs
 *   - auroraview.clipboard - System clipboard access
 *   - auroraview.shell    - Shell commands and URL opening
 * 
 * Usage:
 *   // Include this script after auroraview core is loaded
 *   <script src="auroraview-plugins.js"></script>
 *   
 *   // Then use the APIs
 *   const file = await auroraview.dialog.openFile();
 *   const content = await auroraview.fs.readFile(file.path);
 */

// This file serves as documentation and entry point.
// In production, all plugin JS files are concatenated or loaded individually.

console.log('[AuroraView] Plugins bundle loaded');
