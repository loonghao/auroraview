"""
Thread-safe event queue for DCC (Digital Content Creation) integration.

This module provides a message queue pattern for safe communication between
WebView (running in background thread) and DCC main thread (Maya, Houdini, etc).

Usage:
    # Create event queue
    event_queue = DCCEventQueue()
    
    # Register callbacks
    event_queue.register_callback("select_object", on_select_object)
    
    # Post events from background thread (WebView)
    event_queue.post_event("select_object", "pCube1")
    
    # Process events in main thread (Maya)
    event_queue.process_events()
"""

import queue
import logging
from typing import Callable, Any, Dict, Tuple, Optional

logger = logging.getLogger(__name__)


class DCCEventQueue:
    """
    Thread-safe event queue for DCC integration.
    
    Allows safe communication between WebView (background thread) and
    DCC main thread by using a message queue pattern.
    """
    
    def __init__(self, max_size: int = 1000):
        """
        Initialize the event queue.
        
        Args:
            max_size: Maximum number of events in queue before blocking
        """
        self._queue: queue.Queue = queue.Queue(maxsize=max_size)
        self._callbacks: Dict[str, Callable] = {}
        self._error_callbacks: Dict[str, Callable] = {}
        logger.info(f"DCCEventQueue initialized with max_size={max_size}")
    
    def register_callback(self, event_name: str, callback: Callable) -> None:
        """
        Register a callback for an event.
        
        Args:
            event_name: Name of the event
            callback: Callable to execute when event is processed
        """
        self._callbacks[event_name] = callback
        logger.debug(f"Registered callback for event: {event_name}")
    
    def register_error_callback(self, event_name: str, callback: Callable) -> None:
        """
        Register an error callback for an event.
        
        Args:
            event_name: Name of the event
            callback: Callable to execute if event processing fails
        """
        self._error_callbacks[event_name] = callback
        logger.debug(f"Registered error callback for event: {event_name}")
    
    def post_event(
        self,
        event_name: str,
        *args,
        **kwargs
    ) -> bool:
        """
        Post an event to the queue (thread-safe).
        
        This method is safe to call from any thread, including background threads.
        
        Args:
            event_name: Name of the event
            *args: Positional arguments for the callback
            **kwargs: Keyword arguments for the callback
        
        Returns:
            True if event was posted successfully, False if queue is full
        """
        try:
            self._queue.put_nowait((event_name, args, kwargs))
            logger.debug(f"Posted event: {event_name}")
            return True
        except queue.Full:
            logger.warning(f"Event queue is full, dropping event: {event_name}")
            return False
    
    def process_events(self) -> int:
        """
        Process all pending events in the queue.
        
        This method should be called from the DCC main thread periodically.
        It processes all events currently in the queue and executes their
        registered callbacks.
        
        Returns:
            Number of events processed
        """
        processed_count = 0
        
        while not self._queue.empty():
            try:
                event_name, args, kwargs = self._queue.get_nowait()
                processed_count += 1
                
                if event_name not in self._callbacks:
                    logger.warning(f"No callback registered for event: {event_name}")
                    continue
                
                try:
                    callback = self._callbacks[event_name]
                    callback(*args, **kwargs)
                    logger.debug(f"Processed event: {event_name}")
                except Exception as e:
                    logger.error(
                        f"Error processing event {event_name}: {e}",
                        exc_info=True
                    )
                    
                    # Call error callback if registered
                    if event_name in self._error_callbacks:
                        try:
                            error_callback = self._error_callbacks[event_name]
                            error_callback(e, *args, **kwargs)
                        except Exception as error_e:
                            logger.error(
                                f"Error in error callback for {event_name}: {error_e}",
                                exc_info=True
                            )
            
            except queue.Empty:
                break
        
        if processed_count > 0:
            logger.debug(f"Processed {processed_count} events")
        
        return processed_count
    
    def queue_size(self) -> int:
        """Get current number of events in queue."""
        return self._queue.qsize()
    
    def clear(self) -> None:
        """Clear all events from the queue."""
        while not self._queue.empty():
            try:
                self._queue.get_nowait()
            except queue.Empty:
                break
        logger.info("Event queue cleared")
    
    def get_stats(self) -> Dict[str, Any]:
        """Get queue statistics."""
        return {
            "queue_size": self.queue_size(),
            "registered_callbacks": len(self._callbacks),
            "registered_error_callbacks": len(self._error_callbacks),
        }


# Global event queue instance
_global_event_queue: Optional[DCCEventQueue] = None


def get_event_queue() -> DCCEventQueue:
    """Get or create the global event queue."""
    global _global_event_queue
    if _global_event_queue is None:
        _global_event_queue = DCCEventQueue()
    return _global_event_queue


def set_event_queue(event_queue: DCCEventQueue) -> None:
    """Set the global event queue."""
    global _global_event_queue
    _global_event_queue = event_queue

