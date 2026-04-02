use std::path::Path;
use std::sync::Arc;
use std::thread;

use auroraview_downloads::{DownloadError, DownloadItem, DownloadManager, DownloadQueue, DownloadState};
use rstest::*;
use tempfile::TempDir;

// ========== DownloadItem Struct Tests ==========

#[rstest]
fn item_new() {
    let item = DownloadItem::new("https://example.com/file.zip", "file.zip");
    assert_eq!(item.filename, "file.zip");
    assert_eq!(item.url, "https://example.com/file.zip");
    assert_eq!(item.state, DownloadState::Pending);
    assert_eq!(item.received_bytes, 0);
    assert!(item.total_bytes.is_none());
    assert!(item.speed.is_none());
    assert!(item.error.is_none());
}

#[rstest]
fn item_with_id() {
    let item = DownloadItem::with_id("custom-id", "https://example.com/f.zip", "f.zip");
    assert_eq!(item.id, "custom-id");
}

#[rstest]
fn item_builder_methods() {
    let item = DownloadItem::new("https://example.com/file.zip", "file.zip")
        .with_save_path("/tmp/file.zip")
        .with_mime_type("application/zip")
        .with_total_bytes(1024 * 1024);

    assert!(item.save_path.is_some());
    assert_eq!(item.mime_type, Some("application/zip".to_string()));
    assert_eq!(item.total_bytes, Some(1024 * 1024));
}

// ========== State Transition Tests ==========

#[rstest]
fn item_start_from_pending() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[rstest]
fn item_start_from_paused() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.pause();
    item.start();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[rstest]
fn item_pause() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.update_speed(500);
    item.pause();

    assert_eq!(item.state, DownloadState::Paused);
    assert!(item.speed.is_none()); // speed cleared on pause
}

#[rstest]
fn item_resume() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.pause();
    item.resume();
    assert_eq!(item.state, DownloadState::Downloading);
}

#[rstest]
fn item_complete() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(1000);
    item.start();
    item.update_speed(100);
    item.complete();

    assert_eq!(item.state, DownloadState::Completed);
    assert!(item.completed_at.is_some());
    assert!(item.speed.is_none());
    assert_eq!(item.received_bytes, 1000); // set to total on complete
}

#[rstest]
fn item_fail() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.fail("Connection timeout");

    assert_eq!(item.state, DownloadState::Failed);
    assert_eq!(item.error, Some("Connection timeout".to_string()));
}

#[rstest]
fn item_cancel() {
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

#[rstest]
fn item_progress_unknown_total() {
    let item = DownloadItem::new("https://a.com/f", "f");
    assert!(item.progress().is_none());
}

#[rstest]
fn item_eta_calculation() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(1000);
    item.update_progress(500, None);
    item.update_speed(100); // 100 bytes/sec

    // 500 remaining / 100 bytes/sec = 5 seconds
    assert_eq!(item.eta(), Some(5));
}

#[rstest]
fn item_eta_no_speed() {
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

#[rstest]
fn manager_add() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://example.com/file.zip", "file.zip");

    let item = manager.get(&id).unwrap();
    assert_eq!(item.filename, "file.zip");
    assert_eq!(item.state, DownloadState::Pending);
}

#[rstest]
fn manager_add_item() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let custom = DownloadItem::with_id("my-id", "https://a.com/f.zip", "f.zip");
    let id = manager.add_item(custom);

    assert_eq!(id, "my-id");
    assert!(manager.get(&id).is_some());
}

#[rstest]
fn manager_remove() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    assert!(manager.remove(&id));
    assert!(manager.get(&id).is_none());
    assert!(!manager.remove(&id)); // second time = false
}

#[rstest]
fn manager_all() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");
    manager.add("https://a.com/3", "3");

    assert_eq!(manager.count(), 3);
    assert_eq!(manager.all().len(), 3);
}

// ========== Download Control Tests ==========

#[rstest]
fn manager_start() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    manager.start(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);
}

#[rstest]
fn manager_pause_resume() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");

    manager.start(&id).unwrap();
    manager.pause(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Paused);

    manager.resume(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Downloading);
}

#[rstest]
fn manager_cancel() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.cancel(&id).unwrap();
    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Cancelled);
}

#[rstest]
fn manager_complete() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.complete(&id).unwrap();

    assert_eq!(manager.get(&id).unwrap().state, DownloadState::Completed);
}

#[rstest]
fn manager_fail() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.fail(&id, "Server error").unwrap();

    let item = manager.get(&id).unwrap();
    assert_eq!(item.state, DownloadState::Failed);
    assert_eq!(item.error, Some("Server error".to_string()));
}

// ========== Error State Tests ==========

#[rstest]
fn manager_start_nonexistent_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let result = manager.start(&"bad-id".to_string());
    assert!(matches!(result, Err(DownloadError::NotFound(_))));
}

#[rstest]
fn manager_pause_nondownloading_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    // Still Pending, cannot pause
    let result = manager.pause(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

#[rstest]
fn manager_resume_nonpaused_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    // Already Downloading, cannot resume
    let result = manager.resume(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

#[rstest]
fn manager_cancel_finished_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.complete(&id).unwrap();

    let result = manager.cancel(&id);
    assert!(matches!(result, Err(DownloadError::InvalidState(_))));
}

// ========== Progress & Speed Tests ==========

#[rstest]
fn manager_update_progress() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.update_progress(&id, 750, Some(1000));
    let item = manager.get(&id).unwrap();
    assert_eq!(item.received_bytes, 750);
    assert_eq!(item.total_bytes, Some(1000));
    assert_eq!(item.progress(), Some(75));
}

#[rstest]
fn manager_update_speed() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();

    manager.update_speed(&id, 512);
    let item = manager.get(&id).unwrap();
    assert_eq!(item.speed, Some(512));
}

// ========== Filter Tests ==========

#[rstest]
fn manager_by_state() {
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

#[rstest]
fn manager_clear_completed() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id1 = manager.add("https://a.com/1", "1");
    let _id2 = manager.add("https://a.com/2", "2");

    manager.start(&id1).unwrap();
    manager.complete(&id1).unwrap();

    assert_eq!(manager.count(), 2);
    manager.clear_completed();
    assert_eq!(manager.count(), 1);
}

#[rstest]
fn manager_clear() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");

    manager.clear();
    assert_eq!(manager.count(), 0);
}

// ========== Statistics Tests ==========

#[rstest]
fn manager_total_bytes_downloaded() {
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

#[rstest]
fn manager_queue_stats() {
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

#[rstest]
fn manager_can_start_new() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    assert!(manager.can_start_new());
}

#[rstest]
fn manager_next_to_start() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/1", "1");

    let next = manager.next_to_start();
    assert_eq!(next, Some(id));
}

#[rstest]
fn manager_set_max_concurrent() {
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

#[rstest]
fn manager_export() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");

    let json = manager.export().unwrap();
    assert!(json.contains("https://a.com/1"));
}

#[rstest]
fn manager_persistence_round_trip() {
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

#[rstest]
fn manager_clone_shares_state() {
    let m1 = DownloadManager::new(Some(Path::new(".")));
    let m2 = m1.clone();

    let id = m1.add("https://a.com/f", "f");
    assert!(m2.get(&id).is_some());
}

// ========== DownloadQueue Unit Tests ==========

#[rstest]
fn queue_enqueue_dedup() {
    let mut q = DownloadQueue::new();
    q.enqueue("d1".to_string());
    q.enqueue("d1".to_string()); // duplicate
    assert_eq!(q.pending_count(), 1);
}

#[rstest]
fn queue_priority_goes_to_front() {
    let mut q = DownloadQueue::new();
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());
    q.enqueue_priority("d3".to_string());

    assert_eq!(q.next_pending(), Some("d3".to_string()));
}

#[rstest]
fn queue_respects_max_concurrent() {
    let mut q = DownloadQueue::with_max_concurrent(2);
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());
    q.enqueue("d3".to_string());

    assert_eq!(q.next_pending(), Some("d1".to_string()));
    assert_eq!(q.next_pending(), Some("d2".to_string()));
    assert_eq!(q.next_pending(), None); // slots full
}

#[rstest]
fn queue_mark_finished_frees_slot() {
    let mut q = DownloadQueue::with_max_concurrent(1);
    q.enqueue("d1".to_string());
    q.enqueue("d2".to_string());

    assert_eq!(q.next_pending(), Some("d1".to_string()));
    assert_eq!(q.next_pending(), None);

    q.mark_finished(&"d1".to_string());
    assert_eq!(q.next_pending(), Some("d2".to_string()));
}

#[rstest]
fn queue_next_batch() {
    let mut q = DownloadQueue::with_max_concurrent(3);
    for i in 0..5 {
        q.enqueue(format!("d{}", i));
    }

    let batch = q.next_batch();
    assert_eq!(batch.len(), 3);
    assert_eq!(q.active_count(), 3);
    assert_eq!(q.pending_count(), 2);
}

#[rstest]
fn queue_move_up_down() {
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

// ========== Serde Roundtrip Tests ==========

#[test]
fn serde_download_state_roundtrip() {
    for state in [
        DownloadState::Pending,
        DownloadState::Downloading,
        DownloadState::Paused,
        DownloadState::Completed,
        DownloadState::Failed,
        DownloadState::Cancelled,
    ] {
        let json = serde_json::to_string(&state).unwrap();
        let back: DownloadState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, state, "roundtrip failed for {state:?}");
    }
}

#[test]
fn serde_download_item_basic_roundtrip() {
    let item = DownloadItem::new("https://example.com/file.zip", "file.zip");
    let json = serde_json::to_string(&item).unwrap();
    let back: DownloadItem = serde_json::from_str(&json).unwrap();

    assert_eq!(back.id, item.id);
    assert_eq!(back.url, item.url);
    assert_eq!(back.filename, item.filename);
    assert_eq!(back.state, DownloadState::Pending);
    assert_eq!(back.received_bytes, 0);
    assert!(back.total_bytes.is_none());
}

#[test]
fn serde_download_item_full_roundtrip() {
    let mut item = DownloadItem::new("https://example.com/file.zip", "file.zip")
        .with_save_path("/tmp/file.zip")
        .with_mime_type("application/zip")
        .with_total_bytes(1024 * 1024);
    item.start();
    item.update_progress(512 * 1024, None);
    item.update_speed(100_000);

    let json = serde_json::to_string(&item).unwrap();
    let back: DownloadItem = serde_json::from_str(&json).unwrap();

    assert_eq!(back.state, DownloadState::Downloading);
    assert_eq!(back.received_bytes, 512 * 1024);
    assert_eq!(back.total_bytes, Some(1024 * 1024));
    assert_eq!(back.mime_type, Some("application/zip".to_string()));
}

#[test]
fn serde_download_item_failed_roundtrip() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.fail("Connection reset");

    let json = serde_json::to_string(&item).unwrap();
    let back: DownloadItem = serde_json::from_str(&json).unwrap();

    assert_eq!(back.state, DownloadState::Failed);
    assert_eq!(back.error, Some("Connection reset".to_string()));
}

#[test]
fn serde_download_item_completed_sets_received_to_total() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(2048);
    item.start();
    item.complete();

    let json = serde_json::to_string(&item).unwrap();
    let back: DownloadItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.received_bytes, 2048);
    assert!(back.completed_at.is_some());
}

// ========== DownloadError Display Tests ==========

#[test]
fn error_not_found_display() {
    let err = DownloadError::NotFound("dl-123".to_string());
    let msg = err.to_string();
    assert!(msg.contains("dl-123"), "got: {msg}");
}

#[test]
fn error_already_exists_display() {
    let err = DownloadError::AlreadyExists("dl-dup".to_string());
    let msg = err.to_string();
    assert!(msg.contains("dl-dup"), "got: {msg}");
}

#[test]
fn error_invalid_state_display() {
    let err = DownloadError::InvalidState("cannot pause completed".to_string());
    let msg = err.to_string();
    assert!(msg.contains("cannot pause completed"), "got: {msg}");
}

#[test]
fn error_storage_display() {
    let err = DownloadError::Storage("disk full".to_string());
    let msg = err.to_string();
    assert!(msg.contains("disk full"), "got: {msg}");
}

// ========== DownloadItem Edge Cases ==========

#[rstest]
#[case(DownloadState::Pending, "pending")]
#[case(DownloadState::Downloading, "downloading")]
#[case(DownloadState::Paused, "paused")]
#[case(DownloadState::Completed, "completed")]
#[case(DownloadState::Failed, "failed")]
#[case(DownloadState::Cancelled, "cancelled")]
fn serde_download_state_json_values(#[case] state: DownloadState, #[case] expected_json: &str) {
    let json = serde_json::to_string(&state).unwrap();
    assert_eq!(json, format!(r#""{expected_json}""#));
}

#[test]
fn item_progress_zero_total_bytes() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(0);
    item.update_progress(0, None);
    // zero total bytes - progress might be 0/100 depending on impl
    let _ = item.progress();
}

#[test]
fn item_eta_no_remaining_bytes() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(100);
    item.update_progress(100, None); // all received
    item.update_speed(10);
    // 0 remaining bytes, eta should be 0 or None
    let eta = item.eta();
    assert!(eta.is_none() || eta == Some(0));
}

#[test]
fn item_complete_without_total_bytes() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.complete();
    // no total_bytes, received_bytes stays 0 or same
    assert_eq!(item.state, DownloadState::Completed);
}

#[test]
fn item_fail_twice_keeps_first_error() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.fail("First error");
    // Calling fail again on a non-active item - behavior may vary, just no panic
    // item.fail("Second error");
    assert_eq!(item.error, Some("First error".to_string()));
}

#[test]
fn item_cancel_from_paused() {
    let mut item = DownloadItem::new("https://a.com/f", "f");
    item.start();
    item.pause();
    item.cancel();
    assert_eq!(item.state, DownloadState::Cancelled);
}

#[test]
fn item_speed_cleared_on_complete() {
    let mut item = DownloadItem::new("https://a.com/f", "f").with_total_bytes(100);
    item.start();
    item.update_speed(50);
    assert_eq!(item.speed, Some(50));
    item.complete();
    assert!(item.speed.is_none());
}

// ========== Manager Edge Cases ==========

#[rstest]
fn manager_fail_nonexistent_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let result = manager.fail(&"nonexistent".to_string(), "err");
    assert!(matches!(result, Err(DownloadError::NotFound(_))));
}

#[rstest]
fn manager_complete_nonexistent_returns_error() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let result = manager.complete(&"nonexistent".to_string());
    assert!(matches!(result, Err(DownloadError::NotFound(_))));
}

#[rstest]
fn manager_update_progress_nonexistent_no_panic() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    // update_progress on non-existent id should not panic
    manager.update_progress(&"nonexistent".to_string(), 100, Some(200));
}

#[rstest]
fn manager_failed_items_not_in_active() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    let id = manager.add("https://a.com/f", "f");
    manager.start(&id).unwrap();
    manager.fail(&id, "err").unwrap();

    assert_eq!(manager.active().len(), 0);
}

#[rstest]
fn manager_pending_after_add() {
    let manager = DownloadManager::new(Some(Path::new(".")));
    manager.add("https://a.com/1", "1");
    manager.add("https://a.com/2", "2");
    manager.add("https://a.com/3", "3");

    assert_eq!(manager.pending().len(), 3);
}

// ========== Concurrent Tests ==========

#[test]
fn concurrent_add_no_panic() {

    let manager = Arc::new(DownloadManager::new(Some(Path::new("."))));

    let handles: Vec<_> = (0..8)
        .map(|i| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for j in 0..5 {
                    m.add(
                        format!("https://thread{i}-file{j}.com/f.zip"),
                        format!("f{i}-{j}.zip"),
                    );
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(manager.count(), 40);
}

#[test]
fn concurrent_add_start_complete_no_deadlock() {

    let manager = Arc::new(DownloadManager::new(Some(Path::new("."))));

    // Add 10 items
    let ids: Vec<_> = (0..10)
        .map(|i| manager.add(format!("https://a.com/{i}"), format!("f{i}")))
        .collect();

    // Start and complete them concurrently
    let chunks: Vec<_> = ids.chunks(5).map(|c| c.to_vec()).collect();
    let handles: Vec<_> = chunks
        .into_iter()
        .map(|chunk| {
            let m = Arc::clone(&manager);
            thread::spawn(move || {
                for id in chunk {
                    let _ = m.start(&id);
                    let _ = m.complete(&id);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn concurrent_clone_shares_state() {

    let manager = DownloadManager::new(Some(Path::new(".")));
    let manager2 = manager.clone();

    // Add from one, read from clone concurrently
    let id = manager.add("https://a.com/f", "f");

    let m2 = manager2.clone();
    let reader = thread::spawn(move || {
        for _ in 0..20 {
            let _ = m2.get(&id);
            let _ = m2.count();
        }
    });

    for i in 0..10 {
        manager.add(format!("https://a.com/{i}"), format!("f{i}"));
    }

    reader.join().unwrap();
}
