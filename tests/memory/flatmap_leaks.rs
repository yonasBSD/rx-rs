use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// FLATMAP MEMORY LEAK TESTS
// ============================================================================

#[test]
fn test_self_referencing_flatmap_creates_cycle() {
    let counter = RxRef::new(1);

    // Use subscriber count as a proxy for detecting leaks
    let initial_subs = counter.subscriber_count();
    assert_eq!(initial_subs, 0);

    // Create a self-referencing flat_map
    let counter_clone = counter.clone();
    let flattened = counter.val().flat_map(move |_| {
        // LEAK: Returning the same RxVal we're operating on
        counter_clone.val()
    });

    // Use the value to ensure it's not optimized away
    assert_eq!(flattened.get(), 1);

    // Check subscriber count after flat_map
    let subs_during = counter.subscriber_count();
    println!("Subscribers during flat_map: {}", subs_during);

    // The count should have increased due to subscriptions
    assert!(
        subs_during > initial_subs,
        "Expected subscribers to be created"
    );

    // Drop the flattened value
    drop(flattened);

    // Check if subscribers are cleaned up
    let subs_after = counter.subscriber_count();
    println!("Subscribers after drop: {}", subs_after);

    // THIS IS THE LEAK DETECTION:
    // If there's a cycle, subscribers won't be cleaned up
    if subs_after > initial_subs {
        println!("POTENTIAL LEAK DETECTED!");
        println!(
            "Initial: {}, During: {}, After drop: {}",
            initial_subs, subs_during, subs_after
        );
        println!("Subscribers not cleaned up properly.");
        println!("This indicates a reference cycle.");
    }

    // Note: This test demonstrates the leak but may not fail in all scenarios
    // due to how Rc counting works with complex subscription patterns
}

#[test]
fn test_self_flatmap_leak_with_subscriber_count() {
    let counter = RxRef::new(1);

    // Initial subscriber count should be 0
    assert_eq!(counter.subscriber_count(), 0);

    let counter_clone = counter.clone();

    {
        let flattened = counter.val().flat_map(move |_| counter_clone.val());
        let _ = flattened.get();

        // The flat_map creates subscriptions
        let sub_count = counter.subscriber_count();
        println!("Subscribers during flat_map: {}", sub_count);
        assert!(sub_count > 0, "Expected subscribers to be created");
    }

    // After dropping flattened, check if subscribers are cleaned up
    let final_count = counter.subscriber_count();
    println!("Subscribers after drop: {}", final_count);

    // In a leak scenario, subscribers may not be fully cleaned up
    // This is a diagnostic test to observe the behavior
}

#[test]
fn test_circular_flatmap_chain() {
    let a = RxRef::new(1);
    let b = RxRef::new(2);

    let b_clone = b.clone();
    let a_clone = a.clone();

    // A's flat_map depends on B
    let a_flat = a.val().flat_map(move |_| b_clone.val());

    // B's flat_map depends on A - CIRCULAR!
    let b_flat = b.val().flat_map(move |_| a_clone.val());

    assert_eq!(a_flat.get(), 2);
    assert_eq!(b_flat.get(), 1);

    // Check subscriber counts
    let a_subs_before = a.subscriber_count();
    let b_subs_before = b.subscriber_count();

    println!(
        "A subscribers: {}, B subscribers: {}",
        a_subs_before, b_subs_before
    );

    drop(a_flat);
    drop(b_flat);

    // Check if subscribers were cleaned up
    let a_subs_after = a.subscriber_count();
    let b_subs_after = b.subscriber_count();

    println!(
        "After drop - A subscribers: {}, B subscribers: {}",
        a_subs_after, b_subs_after
    );

    if a_subs_after > 0 || b_subs_after > 0 {
        println!("POTENTIAL CIRCULAR REFERENCE LEAK DETECTED!");
        println!("Subscribers not fully cleaned up after dropping flat_mapped values");
    }
}

#[test]
fn test_flatmap_observable_self_subscribe() {
    let tracker = DisposableTracker::new();
    let source = RxRef::new(1);

    let source_clone = source.clone();

    // flat_map_observable that returns source's stream
    let flattened = source.val().flat_map_observable(move |_| {
        // Returns a stream derived from the same source
        source_clone.stream()
    });

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    flattened.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    // Initial subscription triggers the flat_map
    // This subscribes to source.stream()

    let subs = source.subscriber_count();
    println!("Source subscribers after flat_map_observable: {}", subs);

    source.set(2);
    source.set(3);

    println!("Values emitted: {:?}", values.borrow());
    println!("Final subscriber count: {}", source.subscriber_count());

    // This test demonstrates the subscription pattern
    // In a leak scenario, subscribers would accumulate
}
