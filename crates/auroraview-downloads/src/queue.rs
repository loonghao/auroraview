//! Download queue management

use crate::DownloadId;
use std::collections::VecDeque;

/// Download queue - manages concurrent downloads
#[derive(Debug, Default)]
pub struct DownloadQueue {
    /// Pending downloads
    pending: VecDeque<DownloadId>,
    /// Active downloads
    active: Vec<DownloadId>,
    /// Maximum concurrent downloads
    max_concurrent: usize,
}

impl DownloadQueue {
    /// Default maximum concurrent downloads
    const DEFAULT_MAX_CONCURRENT: usize = 3;

    /// Create a new download queue
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            active: Vec::new(),
            max_concurrent: Self::DEFAULT_MAX_CONCURRENT,
        }
    }

    /// Create a queue with custom max concurrent downloads
    pub fn with_max_concurrent(max: usize) -> Self {
        Self {
            pending: VecDeque::new(),
            active: Vec::new(),
            max_concurrent: max.max(1),
        }
    }

    /// Set maximum concurrent downloads
    pub fn set_max_concurrent(&mut self, max: usize) {
        self.max_concurrent = max.max(1);
    }

    /// Get maximum concurrent downloads
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }

    /// Enqueue a download
    pub fn enqueue(&mut self, download_id: DownloadId) {
        if !self.contains(&download_id) {
            self.pending.push_back(download_id);
        }
    }

    /// Enqueue with priority (at front)
    pub fn enqueue_priority(&mut self, download_id: DownloadId) {
        if !self.contains(&download_id) {
            self.pending.push_front(download_id);
        }
    }

    /// Check if a download is in the queue
    pub fn contains(&self, download_id: &DownloadId) -> bool {
        self.pending.iter().any(|id| id == download_id)
            || self.active.iter().any(|id| id == download_id)
    }

    /// Remove a download from the queue
    pub fn remove(&mut self, download_id: &DownloadId) -> bool {
        let pending_removed = {
            let pos = self.pending.iter().position(|id| id == download_id);
            if let Some(pos) = pos {
                self.pending.remove(pos);
                true
            } else {
                false
            }
        };

        let active_removed = {
            let pos = self.active.iter().position(|id| id == download_id);
            if let Some(pos) = pos {
                self.active.remove(pos);
                true
            } else {
                false
            }
        };

        pending_removed || active_removed
    }

    /// Mark a download as active (started)
    pub fn mark_active(&mut self, download_id: &DownloadId) {
        // Remove from pending if present
        if let Some(pos) = self.pending.iter().position(|id| id == download_id) {
            self.pending.remove(pos);
        }

        // Add to active if not already there
        if !self.active.contains(download_id) {
            self.active.push(download_id.clone());
        }
    }

    /// Mark a download as completed/finished
    pub fn mark_finished(&mut self, download_id: &DownloadId) {
        self.active.retain(|id| id != download_id);
        self.pending.retain(|id| id != download_id);
    }

    /// Get next download to start (if any slot available)
    pub fn next(&mut self) -> Option<DownloadId> {
        if self.active.len() >= self.max_concurrent {
            return None;
        }

        self.pending.pop_front().map(|id| {
            self.active.push(id.clone());
            id
        })
    }

    /// Get all downloads that can be started
    pub fn next_batch(&mut self) -> Vec<DownloadId> {
        let available_slots = self.max_concurrent.saturating_sub(self.active.len());
        let mut batch = Vec::new();

        for _ in 0..available_slots {
            if let Some(id) = self.pending.pop_front() {
                self.active.push(id.clone());
                batch.push(id);
            } else {
                break;
            }
        }

        batch
    }

    /// Check if can start new download
    pub fn can_start(&self) -> bool {
        !self.pending.is_empty() && self.active.len() < self.max_concurrent
    }

    /// Get number of pending downloads
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get number of active downloads
    pub fn active_count(&self) -> usize {
        self.active.len()
    }

    /// Get total queued downloads
    pub fn total_count(&self) -> usize {
        self.pending.len() + self.active.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty() && self.active.is_empty()
    }

    /// Get pending download IDs
    pub fn pending(&self) -> Vec<DownloadId> {
        self.pending.iter().cloned().collect()
    }

    /// Get active download IDs
    pub fn active(&self) -> Vec<DownloadId> {
        self.active.clone()
    }

    /// Clear all pending downloads
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }

    /// Clear everything
    pub fn clear(&mut self) {
        self.pending.clear();
        self.active.clear();
    }

    /// Move a download up in the queue
    pub fn move_up(&mut self, download_id: &DownloadId) {
        if let Some(pos) = self.pending.iter().position(|id| id == download_id) {
            if pos > 0 {
                self.pending.swap(pos, pos - 1);
            }
        }
    }

    /// Move a download down in the queue
    pub fn move_down(&mut self, download_id: &DownloadId) {
        if let Some(pos) = self.pending.iter().position(|id| id == download_id) {
            if pos < self.pending.len() - 1 {
                self.pending.swap(pos, pos + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_queue_basic() {
        let mut queue = DownloadQueue::new();

        queue.enqueue("d1".to_string());
        queue.enqueue("d2".to_string());
        queue.enqueue("d3".to_string());

        assert_eq!(queue.pending_count(), 3);
        assert_eq!(queue.active_count(), 0);
    }

    #[test]
    fn test_queue_next() {
        let mut queue = DownloadQueue::with_max_concurrent(2);

        queue.enqueue("d1".to_string());
        queue.enqueue("d2".to_string());
        queue.enqueue("d3".to_string());

        assert_eq!(queue.next(), Some("d1".to_string()));
        assert_eq!(queue.next(), Some("d2".to_string()));
        assert_eq!(queue.next(), None); // Max concurrent reached

        queue.mark_finished(&"d1".to_string());
        assert_eq!(queue.next(), Some("d3".to_string()));
    }

    #[test]
    fn test_queue_priority() {
        let mut queue = DownloadQueue::new();

        queue.enqueue("d1".to_string());
        queue.enqueue("d2".to_string());
        queue.enqueue_priority("d3".to_string()); // Should go to front

        assert_eq!(queue.next(), Some("d3".to_string()));
    }

    #[test]
    fn test_queue_remove() {
        let mut queue = DownloadQueue::new();

        queue.enqueue("d1".to_string());
        queue.enqueue("d2".to_string());

        assert!(queue.remove(&"d1".to_string()));
        assert!(!queue.contains(&"d1".to_string()));
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn test_queue_batch() {
        let mut queue = DownloadQueue::with_max_concurrent(3);

        queue.enqueue("d1".to_string());
        queue.enqueue("d2".to_string());
        queue.enqueue("d3".to_string());
        queue.enqueue("d4".to_string());

        let batch = queue.next_batch();
        assert_eq!(batch.len(), 3);
        assert_eq!(queue.active_count(), 3);
        assert_eq!(queue.pending_count(), 1);
    }
}
