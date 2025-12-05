/**
 * AuroraView BOM - Browsing Data Management
 * 
 * This script provides utilities for clearing browsing data including
 * localStorage, sessionStorage, IndexedDB, and cookies.
 */
(function() {
    'use strict';

    /**
     * Clear all browsing data accessible from JavaScript
     */
    window.__auroraview_clearAllBrowsingData = function() {
        // Clear localStorage
        try { localStorage.clear(); } catch(e) {}
        
        // Clear sessionStorage
        try { sessionStorage.clear(); } catch(e) {}
        
        // Clear IndexedDB databases
        if (indexedDB && indexedDB.databases) {
            indexedDB.databases().then(function(dbs) {
                dbs.forEach(function(db) {
                    try { indexedDB.deleteDatabase(db.name); } catch(e) {}
                });
            }).catch(function() {});
        }
        
        // Clear accessible cookies
        document.cookie.split(";").forEach(function(c) {
            document.cookie = c.replace(/^ +/, "")
                .replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
        });
        
        console.log('[AuroraView BOM] Browsing data cleared');
    };

    /**
     * Clear localStorage only
     */
    window.__auroraview_clearLocalStorage = function() {
        try { 
            localStorage.clear(); 
            console.log('[AuroraView BOM] localStorage cleared');
        } catch(e) {
            console.error('[AuroraView BOM] Failed to clear localStorage:', e);
        }
    };

    /**
     * Clear sessionStorage only
     */
    window.__auroraview_clearSessionStorage = function() {
        try { 
            sessionStorage.clear(); 
            console.log('[AuroraView BOM] sessionStorage cleared');
        } catch(e) {
            console.error('[AuroraView BOM] Failed to clear sessionStorage:', e);
        }
    };

    /**
     * Clear all IndexedDB databases
     */
    window.__auroraview_clearIndexedDB = function() {
        if (indexedDB && indexedDB.databases) {
            indexedDB.databases().then(function(dbs) {
                dbs.forEach(function(db) {
                    try { 
                        indexedDB.deleteDatabase(db.name); 
                    } catch(e) {}
                });
                console.log('[AuroraView BOM] IndexedDB cleared');
            }).catch(function(e) {
                console.error('[AuroraView BOM] Failed to clear IndexedDB:', e);
            });
        }
    };

    /**
     * Clear all cookies accessible from JavaScript
     */
    window.__auroraview_clearCookies = function() {
        document.cookie.split(";").forEach(function(c) {
            document.cookie = c.replace(/^ +/, "")
                .replace(/=.*/, "=;expires=" + new Date().toUTCString() + ";path=/");
        });
        console.log('[AuroraView BOM] Cookies cleared');
    };

    console.log('[AuroraView BOM] Browsing data utilities initialized');
})();

