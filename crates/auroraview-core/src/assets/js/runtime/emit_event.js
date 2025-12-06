/**
 * Emit event to JavaScript using window.auroraview.trigger()
 * 
 * This template is used by Rust to dispatch events from Python to JavaScript.
 * It ensures compatibility with window.auroraview.on() listeners.
 * 
 * Template variables:
 * - {EVENT_NAME}: Name of the event to trigger
 * - {EVENT_DATA}: JSON string of event data
 */
(function() {
    if (window.auroraview && window.auroraview.trigger) {
        window.auroraview.trigger('{EVENT_NAME}', JSON.parse('{EVENT_DATA}'));
    } else {
        console.error('[AuroraView] Event bridge not ready, cannot emit event: {EVENT_NAME}');
    }
})();

