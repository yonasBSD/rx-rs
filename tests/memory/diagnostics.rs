use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// ============================================================================
// DIAGNOSTIC AND BASELINE TESTS
// ============================================================================

#[test]
fn test_proper_cleanup_no_leak() {
    let x = RxRef::new(10);

    assert_eq!(x.subscriber_count(), 0);

    {
        let tracker = DisposableTracker::new();

        x.val().subscribe(tracker.tracker(), |val| {
            println!("Value: {}", val);
        });

        assert_eq!(x.subscriber_count(), 1);

        // Tracker drops here
    }

    // After tracker drops, subscription should be cleaned up
    assert_eq!(x.subscriber_count(), 0, "Subscription should be cleaned up");
}

#[test]
fn test_subscriber_count_analysis() {
    let x = RxRef::new(1);

    let counts = Rc::new(RefCell::new(Vec::new()));

    // Track subscriber count through various operations
    let initial = x.subscriber_count();
    counts.borrow_mut().push(("initial", initial));

    let val = x.val();
    counts
        .borrow_mut()
        .push(("after val()", x.subscriber_count()));

    let mapped = val.map(|v| v * 2);
    counts
        .borrow_mut()
        .push(("after map()", x.subscriber_count()));

    drop(mapped);
    counts
        .borrow_mut()
        .push(("after drop map", x.subscriber_count()));

    drop(val);
    counts
        .borrow_mut()
        .push(("after drop val", x.subscriber_count()));

    println!("Subscriber count analysis:");
    for (label, count) in counts.borrow().iter() {
        println!("  {}: {}", label, count);
    }

    // Verify cleanup
    assert_eq!(
        x.subscriber_count(),
        0,
        "All subscribers should be cleaned up"
    );
}
