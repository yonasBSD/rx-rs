use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// STREAM MEMORY LEAK TESTS
// ============================================================================

#[test]
fn test_stream_circular_update_pattern() {
    let tracker = DisposableTracker::new();
    let counter = RxRef::new(0);

    let update_count = Rc::new(RefCell::new(0));
    let update_count_clone = update_count.clone();

    let counter_clone = counter.clone();
    let stream = counter.stream();

    // Subscribe and potentially update counter within subscription
    stream.subscribe(tracker.tracker(), move |val| {
        *update_count_clone.borrow_mut() += 1;

        // This creates a feedback pattern (but with guard to prevent infinite loop)
        if *val < 3 {
            counter_clone.set(val + 1);
        }
    });

    // Trigger the update chain
    counter.set(1);

    println!("Update count: {}", update_count.borrow());
    println!("Final counter value: {}", counter.get());
    println!("Counter subscribers: {}", counter.subscriber_count());

    // This test demonstrates feedback loops
    // The subscription closure holds counter_clone
}
