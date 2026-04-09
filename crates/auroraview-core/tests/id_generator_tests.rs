//! ID generator tests

use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

use auroraview_core::id_generator::IdGenerator;

// ---------------------------------------------------------------------------
// Basic sequential generation
// ---------------------------------------------------------------------------

#[test]
fn test_id_generator_sequential() {
    let gen = IdGenerator::new();
    let id1 = gen.next();
    let id2 = gen.next();
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
}

#[test]
fn test_id_generator_many_sequential() {
    let gen = IdGenerator::new();
    for i in 0u64..100 {
        assert_eq!(gen.next(), i);
    }
}

// ---------------------------------------------------------------------------
// String / prefixed IDs
// ---------------------------------------------------------------------------

#[test]
fn test_id_generator_string() {
    let gen = IdGenerator::new();
    let id = gen.next_string();
    assert!(id.starts_with("id_"));
    assert!(id.len() > 3);
}

#[test]
fn test_id_generator_string_sequential() {
    let gen = IdGenerator::new();
    let id0 = gen.next_string();
    let id1 = gen.next_string();
    assert_eq!(id0, "id_0");
    assert_eq!(id1, "id_1");
}

#[test]
fn test_id_generator_with_prefix() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("msg");
    assert!(id.starts_with("msg_"));
}

#[test]
fn test_id_generator_with_prefix_sequential() {
    let gen = IdGenerator::new();
    let id0 = gen.next_with_prefix("req");
    let id1 = gen.next_with_prefix("req");
    assert_eq!(id0, "req_0");
    assert_eq!(id1, "req_1");
}

#[test]
fn test_id_generator_different_prefixes_share_counter() {
    let gen = IdGenerator::new();
    let a = gen.next_with_prefix("a");
    let b = gen.next_with_prefix("b");
    assert_eq!(a, "a_0");
    assert_eq!(b, "b_1");
}

// ---------------------------------------------------------------------------
// with_start
// ---------------------------------------------------------------------------

#[test]
fn test_id_generator_with_start() {
    let gen = IdGenerator::with_start(100);
    assert_eq!(gen.next(), 100);
    assert_eq!(gen.next(), 101);
}

#[test]
fn test_id_generator_with_start_zero() {
    let gen = IdGenerator::with_start(0);
    assert_eq!(gen.next(), 0);
}

#[test]
fn test_id_generator_with_start_large() {
    let gen = IdGenerator::with_start(u64::MAX - 2);
    assert_eq!(gen.next(), u64::MAX - 2);
    assert_eq!(gen.next(), u64::MAX - 1);
}

// ---------------------------------------------------------------------------
// current()
// ---------------------------------------------------------------------------

#[test]
fn test_current_value() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current(), 0);
    gen.next();
    assert_eq!(gen.current(), 1);
}

#[test]
fn test_current_does_not_increment() {
    let gen = IdGenerator::new();
    gen.next();
    let c1 = gen.current();
    let c2 = gen.current();
    assert_eq!(c1, c2);
    assert_eq!(c1, 1);
}

// ---------------------------------------------------------------------------
// Default trait
// ---------------------------------------------------------------------------

#[test]
fn test_id_generator_default() {
    let gen = IdGenerator::default();
    assert_eq!(gen.next(), 0);
}

#[test]
fn test_id_generator_default_independent() {
    let gen1 = IdGenerator::default();
    let gen2 = IdGenerator::default();
    // Two defaults start independently at 0
    assert_eq!(gen1.next(), 0);
    assert_eq!(gen2.next(), 0);
}

// ---------------------------------------------------------------------------
// Thread safety
// ---------------------------------------------------------------------------

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

    all_ids.sort();
    all_ids.dedup();
    assert_eq!(all_ids.len(), 50);
}

#[test]
fn test_id_generator_high_concurrency_unique() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];
    const THREADS: usize = 20;
    const PER_THREAD: usize = 100;

    for _ in 0..THREADS {
        let g = gen.clone();
        handles.push(thread::spawn(move || {
            (0..PER_THREAD).map(|_| g.next()).collect::<Vec<_>>()
        }));
    }

    let mut all: HashSet<u64> = HashSet::new();
    for h in handles {
        for id in h.join().unwrap() {
            assert!(all.insert(id), "Duplicate ID: {}", id);
        }
    }
    assert_eq!(all.len(), THREADS * PER_THREAD);
}

#[test]
fn test_id_generator_string_thread_safe() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];

    for _ in 0..4 {
        let g = gen.clone();
        handles.push(thread::spawn(move || {
            (0..25).map(|_| g.next_string()).collect::<Vec<_>>()
        }));
    }

    let mut all: HashSet<String> = HashSet::new();
    for h in handles {
        for s in h.join().unwrap() {
            assert!(all.insert(s.clone()), "Duplicate string ID: {}", s);
        }
    }
    assert_eq!(all.len(), 100);
}
