/**
 * AuroraView Context Menu Disabler
 * 
 * Disables the native browser context menu (right-click menu).
 * This allows applications to implement custom context menus.
 * 
 * @module context_menu
 */

(function() {
    'use strict';
    
    console.log('[AuroraView] Disabling native context menu...');

    /**
     * Prevent context menu from appearing
     */
    document.addEventListener('contextmenu', function(e) {
        e.preventDefault();
        console.log('[AuroraView] Native context menu disabled');
        return false;
    }, false);

    console.log('[AuroraView] ✓ Context menu disabled');
})();
