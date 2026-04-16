use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Test that dropping a Tracker does NOT clean up subscriptions
#[test]
fn test_tracker_drop_does_not_cleanup() {
    let dt = DisposableTracker::new();
    let rx = RxRef::new(0);

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    // Create a tracker and subscribe
    {
        let tracker = dt.tracker();
        rx.val().subscribe(tracker, move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        // Tracker gets dropped here
    }

    // Subscription should still be active after tracker drop
    assert_eq!(*call_count.borrow(), 1); // Initial call
    rx.set(1);
    assert_eq!(*call_count.borrow(), 2); // Should still receive updates

    rx.set(2);
    assert_eq!(*call_count.borrow(), 3); // Should still receive updates
}

// Test that cloning Tracker does not affect subscription lifetime
#[test]
fn test_tracker_clone_does_not_affect_lifetime() {
    let dt = DisposableTracker::new();
    let rx = RxRef::new(0);

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    // Create a tracker, clone it, and subscribe with the clone
    {
        let tracker1 = dt.tracker();
        let tracker2 = tracker1.clone();

        rx.val().subscribe(&tracker2, move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        // Both trackers get dropped here
    }

    // Subscription should still be active
    assert_eq!(*call_count.borrow(), 1); // Initial call
    rx.set(1);
    assert_eq!(*call_count.borrow(), 2); // Should still receive updates
}

// Test that only DisposableTracker controls subscription lifetime
#[test]
fn test_only_disposable_tracker_controls_lifetime() {
    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));

    {
        let dt = DisposableTracker::new();
        let tracker = dt.tracker();

        let call_count_clone = call_count.clone();
        rx.val().subscribe(tracker, move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        assert_eq!(*call_count.borrow(), 1); // Initial call

        rx.set(1);
        assert_eq!(*call_count.borrow(), 2);

        // DisposableTracker gets dropped here
    }

    // After DisposableTracker drop, subscription should be cleaned up
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // Should NOT receive update
}

// Test that multiple Trackers from same DisposableTracker don't interfere
#[test]
fn test_multiple_trackers_from_same_disposable() {
    let dt = DisposableTracker::new();
    let rx1 = RxRef::new(0);
    let rx2 = RxRef::new(0);

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    {
        let tracker1 = dt.tracker();
        let count1_clone = count1.clone();

        rx1.val().subscribe(tracker1, move |_| {
            *count1_clone.borrow_mut() += 1;
        });

        // Drop tracker1, create tracker2
    }

    {
        let tracker2 = dt.tracker();
        let count2_clone = count2.clone();

        rx2.val().subscribe(tracker2, move |_| {
            *count2_clone.borrow_mut() += 1;
        });

        // Drop tracker2
    }

    // Both subscriptions should still be active
    rx1.set(1);
    rx2.set(1);

    assert_eq!(*count1.borrow(), 2); // initial + update
    assert_eq!(*count2.borrow(), 2); // initial + update
}

// Test that Tracker can be stored and used later without affecting lifetime
#[test]
fn test_tracker_can_be_stored() {
    let dt = DisposableTracker::new();
    let rx = RxRef::new(0);

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    // Store tracker in a variable
    let stored_tracker = dt.tracker();

    {
        let tracker_clone = stored_tracker.clone();
        rx.val().subscribe(&tracker_clone, move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        // tracker_clone gets dropped here
    }

    // Subscription should still be active
    rx.set(1);
    assert_eq!(*call_count.borrow(), 2);

    // stored_tracker is still alive here but shouldn't affect anything
    let _ = stored_tracker;

    // Subscription should still be active
    rx.set(2);
    assert_eq!(*call_count.borrow(), 3);
}

// Test that Tracker reference doesn't extend DisposableTracker lifetime
#[test]
fn test_tracker_does_not_extend_disposable_lifetime() {
    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    let tracker = {
        let dt = DisposableTracker::new();
        let t = dt.tracker();

        rx.val().subscribe(t, move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        assert_eq!(*call_count.borrow(), 1); // Initial call

        t.clone() // Return a clone of the tracker
                  // dt gets dropped here
    };

    // DisposableTracker was dropped, so subscription should be cleaned up
    rx.set(1);
    assert_eq!(*call_count.borrow(), 1); // Should NOT receive update

    // Even though we still have the tracker
    drop(tracker);

    rx.set(2);
    assert_eq!(*call_count.borrow(), 1); // Still no updates
}
