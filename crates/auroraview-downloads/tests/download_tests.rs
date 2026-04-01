use auroraview_downloads::{DownloadError, DownloadItem, DownloadManager, DownloadQueue, DownloadState};
use rstest::*;
use std::path::Path;
use tempfile::TempDir;

// ========== DownloadItem Struct Tests ==========

#[test]
fn test_item_new() {
    let item = DownloadItem::new("https://example.com/file.zip", "file.zip");
    assert_eq!(item.filename, "file.zip");
    assert_eq!(item.url, "https://example.com/file.zip");
    assert_eq!(item.state, DownloadState::Pending);
    assert_eq!(item.received_bytes, 0);
    assert!(item.total_bytes.is_none());
    assert!(item.speed.is_none());
    assert!(item.error.is_none());
}

#[test]
fn test_item_with_id() {
    let item = DownloadItem::with_id("custom-id", "https://example.com/f.zip", "f.zip");
    assert_eq!(item.id, "custom-id");
}

#[test]
fn test_item_builder_methods() {
    let item = DownloadItem::new("https://example.com/file.zip", "file.zip")
        .with_save_path("/tmp/file.zip")
        .with_mime_type("application/zip")
        .with_total_bytes(1024 * 1024);

    assert!(item.save_path.is_some());
    assert_eq!(item.mime_type, Some("application/zip".to_string()));
    assert_eq!(item.total_bytes, Some(1024 * 1024));
}

// ========== State Transition Tests ==========

#[test]
fn test_item_start_from_pending() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[test]
fn test_item_start_from_paused() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.pause();
    item.start();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[test]
fn test_item_pause() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.update_speed(500);
    item.pause();

    assert_eq!(item.state, DownloadState::Paused);
    assert!(item.speed.is_none()); // speed cleared on pause
}

#[test]
fn test_item_resume() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.pause();
    item.resume();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[test]
fn test_item_complete() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(1000);
    item.start();
    item.update_speed(100);
    item.complete();

    assert_eq!(item.state, DownloadState::Completed);
    assert!(item.completed_at.is_some());
    assert!(item.speed.is_none());
    assert_eq!(item.received_bytes, 1000); // set to total on complete
}

#[test]
fn test_item_fail() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.fail("Connection timeout");

    assert_eq!(item.state, DownloadState::Failed);
    assert_eq!(item.error, Some("Connection timeout".to_string()));
}

#[test]
fn test_item_cancel() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.cancel();
    assert_eq!(item.state, DownloadState::Cancelled);
}

// ========== State Query Tests ==========

#[rstest]
#[case(DownloadState::Downloading, true, false, false)]
#[case(DownloadState::Paused, false, true, false)]
#[case(DownloadState::Completed, false, false, true)]
#[case(DownloadState::Failed, false, false, true)]
#[case(DownloadState::Cancelled, false, false, true)]
fn test_item_state_queries(
    #[case] state: DownloadState,
    #[case] is_active: bool,
    #[case] can_resume: bool,
    #[case] is_finished: bool,
) {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    match state {
        DownloadState::Downloading => item.start(),
        DownloadState::Paused => { item.start(); item.pause(); }
        DownloadState::Completed => { item.start(); item.complete(); }
        DownloadState::Failed => { item.start(); item.fail("err"); }
        DownloadState::Cancelled => { item.start(); item.cancel(); }
        DownloadState::Pending => {}
    }
    assert_eq!(item.is_active(), is_active);
    assert_eq!(item.can_resume(), can_resume);
    assert_eq!(item.is_finished(), is_finished);
}

// ========== Progress & ETA Tests ==========

#[rstest]
#[case(0, 1000, 0)]
#[case(500, 1000, 50)]
#[case(1000, 1000, 100)]
#[case(750, 1000, 75)]
fn test_item_progress(#[case] received: u64, #[case] total: u64, #[case] expected_pct: u8) {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(total);
    item.update_progress(received, None);
    assert_eq!(item.progress(), Some(expected_pct));
}

#[test]
fn test_item_progress_unknown_total() {
    let item = DownloadItem::new("https://a.com/f", "f");
    assert!(item.progress().is_none());
}

#[test]
fn test_item_eta_calculation() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(1000);
    item.update_progress(500, None);
    item.update_speed(100); // 100 bytes/sec

    // 500 remaining / 100 bytes/sec = 5 seconds
    assert_eq!(item.eta(), Some(5));
}

#[test]
fn test_item_eta_no_speed() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(1000);
    item.update_progress(500, None);
    assert!(item.eta().is_none());
}

// ========== Domain & Extension Tests ==========

#[rstest]
#[case("https://cdn.example.com/files/f.zip", Some("cdn.example.com"))]
#[case("http://another.org/path/file.tar.gz", Some("another.org"))]
fn test_item_domain(#[case] url: &str, #[case] expected: Option<&str>) {
    let item = DownloadItem::new(url, "f");
    assert_eq!(item.domain(), expected);
}

#[rstest]
#[case("file.zip", Some("zip"))]
#[case("archive.tar.gz", Some("gz"))]
#[case("noext", Some("noext"))]
fn test_item_extension(#[case] filename: &str, #[case] expected: Option<&str>) {
    let item = DownloadItem::new("https://a.com/f", filename);
    assert_eq!(item.extension(), expected);
}

// ========== DownloadManager CRUD Tests ==========

#[test]
fn test_manager_add() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://example.com/file.zip", "file.zip");

    let item = manager.get(&id).unwrap();
    assert_eq!(item.filename, "file.zip");
    assert_eq!(item.state, DownloadState::Pending);
}

#[test]
fn test_manager_add_item() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let custom = DownloadItem::with_id("my-id", "https://a.com/f.zip", "f.zip");
    let id = manager.add_item(custom);

    assert_eq!(id, "my-id");
    assert!(manager.get(&id).is_some());
}

#[test]
fn test_manager_remove() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    assert!(manager.remove(&id));
    assert!(manager.get(&id).is_none());
    assert!(!manager.remove(&id)); // second time = false
}

#[test]
fn test_manager_all() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");
    manager.add("https://a.com/3", "3");

    assert_eq!(manager.count(), 3);
    assert_eq!(manager.all().len(), 3);
}

// ========== Download Control Tests ==========

#[test]
fn test_manager_start() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    manager.start(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);
}

#[test]
fn test_manager_pause_resume() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    manager.start(&id).unwrap();
    manager.pause(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Paused);

    manager.resume(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);
}

#[test]
fn test_manager_cancel() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.cancel(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Cancelled);
}

#[test]
fn test_manager_complete() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.complete(&id).unwrap();

    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Completed);
}

#[test]
fn test_manager_fail() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.fail(&id, "Server error").unwrap();

    let item = manager.get(&id).unwrap();
    assert_eq!(item.state, DownloadState::Failed);
    assert_eq!(item.error, Some("Server error".to_string()));
}

// ========== Error State Tests ==========

#[test]
fn test_manager_start_nonexistent_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let result = manager.start(&"bad-id".to_string());
    assert!(matches!(result, Err(DownloadError::NotFound(_))));
}

#[test]
fn test_manager_pause_nondownloading_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    // Still Pending, cannot pause
    let result = manager.pause(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

#[test]
fn test_manager_resume_nonpaused_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    // Already Downloading, cannot resume
    let result = manager.resume(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

#[test]
fn test_manager_cancel_finished_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.complete(&id).unwrap();

    let result = manager.cancel(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

// ========== Progress & Speed Tests ==========

#[test]
fn test_manager_update_progress() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.update_progress(&id, 750, Some(1000));
    let item = manager.get(&id).unwrap();
    assert_eq!(item.received_bytes, 750);
    assert_eq!(item.total_bytes, Some(1000));
    assert_eq!(item.progress(), Some(75));
}

#[test]
fn test_manager_update_speed() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.update_speed(&id, 512);
    let item = manager.get(&id).unwrap();
    assert_eq!(item.speed, Some(512));
}

// ========== Filter Tests ==========

#[test]
fn test_manager_by_state() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id1 = manager.add("https://a.com/1", "1");
    let id2 = manager.add("https://a.com/2", "2");
    let _id3 = manager.add("https://a.com/3", "3");

    manager.start(&id1).unwrap();
    manager.start(&id2).unwrap();
    manager.complete(&id2).unwrap();

    assert_eq!(manager.active().len(), 1);
    assert_eq!(manager.completed().len(), 1);
    assert_eq!(manager.pending().len(), 1);
}

#[test]
fn test_manager_clear_completed() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id1 = manager.add("https://a.com/1", "1");
    let _id2 = manager.add("https://a.com/2", "2");

    manager.start(&id1).unwrap();
    manager.complete(&id1).unwrap();

    assert_eq!(manager.count(), 2);
    manager.clear_completed();
    assert_eq!(manager.count(), 1);
}

#[test]
fn test_manager_clear() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");

    manager.clear();
    assert_eq!(manager.count(), 0);
}

// ========== Statistics Tests ==========

#[test]
fn test_manager_total_bytes_downloaded() {
    let manager = DownloadManager::new(Some(Path::new(".")));

    let id1 = manager.add("https://a.com/1", "1");
    manager.start(&id1).unwrap();
    manager.update_progress(&id1, 0, Some(1024));
    manager.complete(&id1).unwrap();

    let id2 = manager.add("https://a.com/2", "2");
    manager.start(&id2).unwrap();
    manager.update_progress(&id2, 0, Some(2048));
    manager.complete(&id2).unwrap();

    // Pending download should not be counted
    manager.add("https://a.com/3", "3");

    assert_eq!(manager.total_bytes_downloaded(), 1024 + 2048);
}

// ========== Queue Tests ==========

#[test]
fn test_manager_queue_stats() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id1 = manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");

    let (pending, active, total) = manager.queue_stats();
    assert_eq!(total, 2);
    assert_eq!(pending, 2);
    assert_eq!(active, 0);

    manager.start(&id1).unwrap();
    let (pending, active, total) = manager.queue_stats();
    assert_eq!(pending, 1);
    assert_eq!(active, 1);
    assert_eq!(total, 2);
}

#[test]
fn test_manager_can_start_new() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    assert!(manager.can_start_new());
}

#[test]
fn test_manager_next_to_start() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/1", "1");

    let next = manager.next_to_start();
    assert_eq!(next, Some(id));
}

#[test]
fn test_manager_set_max_concurrent() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.set_max_concurrent(1);

    let id1 = manager.add("https://a.com/1", "1");
    let id2 = manager.add("https://a.com/2", "2");

    manager.start(&id1).unwrap();
    assert_eq!(manager.active().len(), 1);

    // With max=1, cannot start id2 while id1 is active
    assert!(!manager.can_start_new());
    let _ = id2; // suppress unused
}

// ========== Persistence Tests ==========

#[test]
fn test_manager_export() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");

    let json = manager.export().unwrap();
    assert!(json.contains("https://a.com/1"));
}

#[test]
fn test_manager_persistence_round_trip() {
    let dir = TempDir::new().unwrap();
    let dl_dir = dir.path().join("downloads");
    let data_dir = dir.path().join("data");
    std::fs::create_dir_all(&dl_dir).unwrap();
    std::fs::create_dir_all(&data_dir).unwrap();

    {
        let manager = DownloadManager::with_persistence(Some(&dl_dir), &data_dir);
        let id = manager.add("https://example.com/f.zip", "f.zip");
        manager.start(&id).unwrap();
        manager.complete(&id).unwrap();
    }

    // Reload
    let manager2 = DownloadManager::with_persistence(Some(&dl_dir), &data_dir);
    let all = manager2.all();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].filename, "f.zip");
    assert_eq!(all[0].state, DownloadState::Completed);
}

// ========== Clone Tests ==========

#[test]
fn test_manager_clone_shares_state() {
    let m1 = DownloadManager::new(Some(Path::new(".")));
    let m2 = m1.clone();

    let id = m1.add("https://a.com/f", "f");
    assert!(m2.get(&id).is_some());
}

// ========== DownloadQueue Unit Tests ==========

#[test]
fn test_queue_enqueue_dedup() {
    let mut q = DownloadQueue::new();
    q.enqueue("d1".to_string());
    q.enqueue("d1".to_string()); // duplicate
    assert_eq!(q.pending_count(), 1);
}

#[test]
fn test_queue_priority_goes_to_front() {
    let mut q = DownloadQueue::new();
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());
    q.enqueue_priority("d3".to_string());

    assert_eq!(q.next_pending(), Some("d3".to_string()));
}

#[test]
fn test_queue_respects_max_concurrent() {
    let mut q = DownloadQueue::with_max_concurrent(2);
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());
    q.enqueue("d3".to_string());

    assert_eq!(q.next_pending(), Some("d1".to_string()));
    assert_eq!(q.next_pending(), Some("d2".to_string()));
    assert_eq!(q.next_pending(), None); // slots full
}

#[test]
fn test_queue_mark_finished_frees_slot() {
    let mut q = DownloadQueue::with_max_concurrent(1);
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());

    assert_eq!(q.next_pending(), Some("d1".to_string()));
    assert_eq!(q.next_pending(), None);

    q.mark_finished(&"d1".to_string());
    assert_eq!(q.next_pending(), Some("d2".to_string()));
}

#[test]
fn test_queue_next_batch() {
    let mut q = DownloadQueue::with_max_concurrent(3);
    for i in 0..5 {
        q.enqueue(format!("d{}", i));
    }

    let batch = q.next_batch();
    assert_eq!(batch.len(), 3);
    assert_eq!(q.active_count(), 3);
    assert_eq!(q.pending_count(), 2);
}

#[test]
fn test_queue_move_up_down() {
    let mut q = DownloadQueue::new();
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());
    q.enqueue("d3".to_string());

    q.move_down(&"d1".to_string()); // d2, d1, d3
    assert_eq!(q.next_pending(), Some("d2".to_string()));

    let mut q2 = DownloadQueue::new();
    q2.enqueue("d1".to_string());
    q2.enqueue("d2".to_string());
    q2.enqueue("d3".to_string());
    q2.move_up(&"d3".to_string()); // d1, d3, d2
    assert_eq!(q2.next_pending(), Some("d1".to_string()));
    assert_eq!(q2.next_pending(), Some("d3".to_string()));
}
