//! ID generator tests

use auroraview_core::id_generator::IdGenerator;
use std::sync::Arc;
use std::thread;

#[test]
fn test_id_generator_sequential() {
    let gen = IdGenerator::new();
    let id1 = gen.next();
    let id2 = gen.next();
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
}

#[test]
fn test_id_generator_string() {
    let gen = IdGenerator::new();
    let id = gen.next_string();
    assert!(id.starts_with("id_"));
}

#[test]
fn test_id_generator_with_prefix() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("msg");
    assert!(id.starts_with("msg_"));
}

#[test]
fn test_id_generator_thread_safe() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];

    for _ in 0..5 {
        let gen_clone = gen.clone();
        let handle = thread::spawn(move || {
            let mut ids = vec![];
            for _ in 0..10 {
                ids.push(gen_clone.next());
            }
            ids
        });
        handles.push(handle);
    }

    let mut all_ids = vec![];
    for handle in handles {
        all_ids.extend(handle.join().unwrap());
    }

    // Verify all IDs are unique
    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), 50);
}

#[test]
fn test_id_generator_with_start() {
    let gen = IdGenerator::with_start(100);
    assert_eq!(gen.next(), 100);
    assert_eq!(gen.next(), 101);
}

#[test]
fn test_current_value() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current(), 0);
    gen.next();
    assert_eq!(gen.current(), 1);
}
