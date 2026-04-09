//! ID generator tests

use std::collections::HashSet;
use std::sync::Arc;
use std::thread;

use auroraview_core::id_generator::IdGenerator;

// ---------------------------------------------------------------------------
// Basic sequential generation
// ---------------------------------------------------------------------------

#[test]
fn id_generator_sequential() {
    let gen = IdGenerator::new();
    let id1 = gen.next();
    let id2 = gen.next();
    assert_eq!(id1, 0);
    assert_eq!(id2, 1);
}

#[test]
fn id_generator_many_sequential() {
    let gen = IdGenerator::new();
    for i in 0u64..100 {
        assert_eq!(gen.next(), i);
    }
}

#[test]
fn id_generator_monotonically_increasing() {
    let gen = IdGenerator::new();
    let mut last = gen.next();
    for _ in 0..50 {
        let next = gen.next();
        assert!(next > last, "IDs must be strictly increasing");
        last = next;
    }
}

// ---------------------------------------------------------------------------
// String / prefixed IDs
// ---------------------------------------------------------------------------

#[test]
fn id_generator_string() {
    let gen = IdGenerator::new();
    let id = gen.next_string();
    assert!(id.starts_with("id_"));
    assert!(id.len() > 3);
}

#[test]
fn id_generator_string_sequential() {
    let gen = IdGenerator::new();
    let id0 = gen.next_string();
    let id1 = gen.next_string();
    assert_eq!(id0, "id_0");
    assert_eq!(id1, "id_1");
}

#[test]
fn id_generator_with_prefix() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("msg");
    assert!(id.starts_with("msg_"));
}

#[test]
fn id_generator_with_prefix_sequential() {
    let gen = IdGenerator::new();
    let id0 = gen.next_with_prefix("req");
    let id1 = gen.next_with_prefix("req");
    assert_eq!(id0, "req_0");
    assert_eq!(id1, "req_1");
}

#[test]
fn id_generator_different_prefixes_share_counter() {
    let gen = IdGenerator::new();
    let a = gen.next_with_prefix("a");
    let b = gen.next_with_prefix("b");
    assert_eq!(a, "a_0");
    assert_eq!(b, "b_1");
}

#[test]
fn id_generator_string_unique_across_calls() {
    let gen = IdGenerator::new();
    let ids: Vec<String> = (0..20).map(|_| gen.next_string()).collect();
    let unique: HashSet<&String> = ids.iter().collect();
    assert_eq!(unique.len(), 20, "All string IDs must be unique");
}

#[test]
fn id_generator_prefix_empty_string() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("");
    // Empty prefix: should produce "_0"
    assert!(id.starts_with('_'));
}

#[test]
fn id_generator_prefix_unicode() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("前端");
    assert!(id.starts_with("前端_"));
}

// ---------------------------------------------------------------------------
// with_start
// ---------------------------------------------------------------------------

#[test]
fn id_generator_with_start() {
    let gen = IdGenerator::with_start(100);
    assert_eq!(gen.next(), 100);
    assert_eq!(gen.next(), 101);
}

#[test]
fn id_generator_with_start_zero() {
    let gen = IdGenerator::with_start(0);
    assert_eq!(gen.next(), 0);
}

#[test]
fn id_generator_with_start_large() {
    let gen = IdGenerator::with_start(u64::MAX - 2);
    assert_eq!(gen.next(), u64::MAX - 2);
    assert_eq!(gen.next(), u64::MAX - 1);
}

#[test]
fn id_generator_with_start_arbitrary() {
    let gen = IdGenerator::with_start(42);
    assert_eq!(gen.next(), 42);
    assert_eq!(gen.next(), 43);
    assert_eq!(gen.current(), 44);
}

// ---------------------------------------------------------------------------
// current()
// ---------------------------------------------------------------------------

#[test]
fn current_value() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current(), 0);
    gen.next();
    assert_eq!(gen.current(), 1);
}

#[test]
fn current_does_not_increment() {
    let gen = IdGenerator::new();
    gen.next();
    let c1 = gen.current();
    let c2 = gen.current();
    assert_eq!(c1, c2);
    assert_eq!(c1, 1);
}

#[test]
fn current_after_many_nexts() {
    let gen = IdGenerator::new();
    for _ in 0..50 {
        gen.next();
    }
    assert_eq!(gen.current(), 50);
}

#[test]
fn current_matches_next_string_count() {
    let gen = IdGenerator::new();
    let _s0 = gen.next_string();
    let _s1 = gen.next_string();
    let _s2 = gen.next_string();
    assert_eq!(gen.current(), 3);
}

// ---------------------------------------------------------------------------
// Default trait
// ---------------------------------------------------------------------------

#[test]
fn id_generator_default() {
    let gen = IdGenerator::default();
    assert_eq!(gen.next(), 0);
}

#[test]
fn id_generator_default_independent() {
    let gen1 = IdGenerator::default();
    let gen2 = IdGenerator::default();
    // Two defaults start independently at 0
    assert_eq!(gen1.next(), 0);
    assert_eq!(gen2.next(), 0);
}

#[test]
fn two_generators_independent_counters() {
    let gen1 = IdGenerator::new();
    let gen2 = IdGenerator::new();
    gen1.next();
    gen1.next();
    // gen2 should still start at 0
    assert_eq!(gen2.next(), 0);
}

// ---------------------------------------------------------------------------
// Thread safety
// ---------------------------------------------------------------------------

#[test]
fn id_generator_thread_safe() {
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
fn id_generator_high_concurrency_unique() {
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
fn id_generator_string_thread_safe() {
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


#[test]
fn id_generator_current_reflects_concurrent_writes() {
    let gen = Arc::new(IdGenerator::new());
    let mut handles = vec![];
    const THREADS: usize = 8;
    const PER_THREAD: usize = 50;

    for _ in 0..THREADS {
        let g = gen.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..PER_THREAD {
                g.next();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(
        gen.current(),
        (THREADS * PER_THREAD) as u64,
        "current() should reflect all increments"
    );
}

// ---------------------------------------------------------------------------
// New: Send+Sync
// ---------------------------------------------------------------------------

#[test]
fn id_generator_is_send_sync() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}
    assert_send::<IdGenerator>();
    assert_sync::<IdGenerator>();
}

// ---------------------------------------------------------------------------
// New: next_string format
// ---------------------------------------------------------------------------

#[test]
fn next_string_format_matches_id_prefix() {
    let gen = IdGenerator::new();
    let s = gen.next_string();
    // Expected format: "id_{n}"
    assert!(s.starts_with("id_"));
    let num_part = s.strip_prefix("id_").unwrap();
    let parsed: u64 = num_part.parse().expect("numeric suffix");
    assert_eq!(parsed, 0);
}

#[test]
fn next_with_prefix_format() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("tab");
    assert_eq!(id, "tab_0");
    let id2 = gen.next_with_prefix("tab");
    assert_eq!(id2, "tab_1");
}

// ---------------------------------------------------------------------------
// New: sequential uniqueness across mixed calls
// ---------------------------------------------------------------------------

#[test]
fn mixed_calls_unique_ids() {
    let gen = IdGenerator::new();
    let n1 = gen.next();
    let s1 = gen.next_string();
    let p1 = gen.next_with_prefix("msg");
    let n2 = gen.next();

    // All counters from same generator → incrementing
    assert_eq!(n1, 0);
    assert_eq!(s1, "id_1");
    assert_eq!(p1, "msg_2");
    assert_eq!(n2, 3);
}

// ---------------------------------------------------------------------------
// New: current before any next
// ---------------------------------------------------------------------------

#[test]
fn current_before_any_next() {
    let gen = IdGenerator::new();
    assert_eq!(gen.current(), 0);
}

// ---------------------------------------------------------------------------
// New: with_start string ID
// ---------------------------------------------------------------------------

#[test]
fn with_start_string_id() {
    let gen = IdGenerator::with_start(5);
    let s = gen.next_string();
    assert_eq!(s, "id_5");
}

// ---------------------------------------------------------------------------
// New: next_with_prefix unicode multiple calls
// ---------------------------------------------------------------------------

#[test]
fn next_with_prefix_unicode_multiple() {
    let gen = IdGenerator::new();
    let a = gen.next_with_prefix("イベント");
    let b = gen.next_with_prefix("イベント");
    assert_eq!(a, "イベント_0");
    assert_eq!(b, "イベント_1");
}

// ---------------------------------------------------------------------------
// New: string generation count matches current
// ---------------------------------------------------------------------------

#[test]
fn string_id_count_matches_current() {
    let gen = IdGenerator::new();
    for _ in 0..5 {
        gen.next_string();
    }
    assert_eq!(gen.current(), 5);
}

// ---------------------------------------------------------------------------
// New: prefix generation count matches current
// ---------------------------------------------------------------------------

#[test]
fn prefix_id_count_matches_current() {
    let gen = IdGenerator::new();
    for _ in 0..7 {
        gen.next_with_prefix("x");
    }
    assert_eq!(gen.current(), 7);
}

// ---------------------------------------------------------------------------
// Extended coverage: edge values and concurrency
// ---------------------------------------------------------------------------

#[test]
fn next_returns_zero_first() {
    let gen = IdGenerator::new();
    assert_eq!(gen.next(), 0, "first ID should always be 0");
}

#[test]
fn with_start_zero_same_as_new() {
    let gen = IdGenerator::with_start(0);
    assert_eq!(gen.next(), 0);
}

#[test]
fn with_start_large_value() {
    let gen = IdGenerator::with_start(u64::MAX - 1);
    assert_eq!(gen.next(), u64::MAX - 1);
    // wraps around
    let next = gen.next();
    // just check it doesn't panic
    let _ = next;
}

#[test]
fn next_string_sequential_numbers() {
    let gen = IdGenerator::new();
    let s0 = gen.next_string();
    let s1 = gen.next_string();
    let s2 = gen.next_string();
    assert_eq!(s0, "id_0");
    assert_eq!(s1, "id_1");
    assert_eq!(s2, "id_2");
}

#[test]
fn prefix_empty_string() {
    let gen = IdGenerator::new();
    let id = gen.next_with_prefix("");
    // Empty prefix should still produce "_0" or similar
    assert!(id.ends_with('0'), "should end with the counter value 0");
}

#[test]
fn id_uniqueness_with_reset() {
    let gen = IdGenerator::new();
    let ids: HashSet<u64> = (0..20).map(|_| gen.next()).collect();
    assert_eq!(ids.len(), 20, "all 20 IDs should be unique");
}

#[test]
fn string_ids_all_unique() {
    let gen = IdGenerator::new();
    let ids: HashSet<String> = (0..20).map(|_| gen.next_string()).collect();
    assert_eq!(ids.len(), 20);
}

#[test]
fn prefix_ids_all_unique() {
    let gen = IdGenerator::new();
    let ids: HashSet<String> = (0..20).map(|_| gen.next_with_prefix("evt")).collect();
    assert_eq!(ids.len(), 20);
}

#[test]
fn next_with_prefix_numeric_part_matches_counter() {
    let gen = IdGenerator::new();
    for i in 0u64..10 {
        let id = gen.next_with_prefix("n");
        let num: u64 = id.strip_prefix("n_").unwrap().parse().unwrap();
        assert_eq!(num, i);
    }
}

#[test]
fn concurrent_string_ids_no_duplicates() {
    let gen = Arc::new(IdGenerator::new());
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let g = gen.clone();
            thread::spawn(move || {
                (0..25).map(|_| g.next_string()).collect::<Vec<_>>()
            })
        })
        .collect();

    let mut all: HashSet<String> = HashSet::new();
    for h in handles {
        for s in h.join().unwrap() {
            assert!(all.insert(s.clone()), "duplicate string ID: {s}");
        }
    }
    assert_eq!(all.len(), 100);
}

#[test]
fn current_after_mixed_calls_equals_total() {
    let gen = IdGenerator::new();
    gen.next();             // +1
    gen.next_string();      // +1
    gen.next_with_prefix("p"); // +1
    gen.next();             // +1
    assert_eq!(gen.current(), 4);
}


