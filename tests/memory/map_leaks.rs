use rx_rs::prelude::*;

// ============================================================================
// MAP MEMORY LEAK TESTS
// ============================================================================

#[test]
fn test_map_chain_with_closure_capture() {
    let base = RxRef::new(10);

    let initial_subs = base.subscriber_count();

    // Create a map that captures base in its closure
    let base_clone = base.clone();
    let mapped = base.val().map(move |x| {
        // The closure captures base_clone and uses it
        x + base_clone.get()
    });

    assert_eq!(mapped.get(), 20); // 10 + 10

    base.set(5);
    assert_eq!(mapped.get(), 10); // 5 + 5

    let subs_during = base.subscriber_count();
    println!(
        "Subscribers - initial: {}, during map: {}",
        initial_subs, subs_during
    );

    drop(mapped);

    let subs_after = base.subscriber_count();
    println!("Subscribers after drop: {}", subs_after);

    if subs_after > initial_subs {
        println!("POTENTIAL LEAK: Subscribers not cleaned up");
        println!("This indicates the closure is holding a reference");
    }
}

#[test]
fn test_complex_map_chain_leak() {
    let source = RxRef::new(1);

    // Create multiple map levels, each capturing source
    let s1 = source.clone();
    let map1 = source.val().map(move |x| x + s1.get());

    let s2 = source.clone();
    let map2 = map1.map(move |x| x * s2.get());

    let s3 = source.clone();
    let map3 = map2.map(move |x| x - s3.get());

    assert_eq!(map3.get(), 1); // (1 + 1) * 1 - 1 = 1

    let subs = source.subscriber_count();
    println!("Source subscribers during chain: {}", subs);

    drop(map3);
    drop(map2);
    drop(map1);

    let subs_after = source.subscriber_count();
    println!("Source subscribers after drops: {}", subs_after);

    if subs_after > 0 {
        println!("POTENTIAL LEAK: Complex map chain may have created cycles");
    }
}

#[test]
fn test_independent_operations_no_leak() {
    let x = RxRef::new(5);

    // Create independent mapped value (no cycle)
    let doubled = x.val().map(|v| v * 2);

    assert_eq!(doubled.get(), 10);

    let _subs_before = x.subscriber_count();

    drop(doubled);

    let subs_after = x.subscriber_count();

    // Should clean up properly since no cycles
    assert_eq!(subs_after, 0, "Independent map should not leak");
}
