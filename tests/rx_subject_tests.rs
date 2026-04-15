use rx_rs::core::{DisposableTracker, RxSubject};
use std::cell::RefCell;
use std::rc::Rc;

// Test 1: No immediate call on subscribe
#[test]
fn test_no_immediate_call() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let called = Rc::new(RefCell::new(false));
    let called_clone = called.clone();

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *called_clone.borrow_mut() = true;
    });

    // Should NOT be called immediately (unlike RxVal)
    assert!(!*called.borrow());
}

// Test 2: Next emits to subscriber
#[test]
fn test_next_emits_to_subscriber() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let received = Rc::new(RefCell::new(None));
    let received_clone = received.clone();

    rx.observable().subscribe(tracker.tracker(), move |val: &i32| {
        *received_clone.borrow_mut() = Some(*val);
    });

    // Nothing received yet
    assert_eq!(*received.borrow(), None);

    // Emit event
    rx.next(42);
    assert_eq!(*received.borrow(), Some(42));
}

// Test 3: Multiple events
#[test]
fn test_multiple_events() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let events = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();

    rx.observable().subscribe(tracker.tracker(), move |val: &i32| {
        events_clone.borrow_mut().push(*val);
    });

    rx.next(1);
    rx.next(2);
    rx.next(3);

    assert_eq!(*events.borrow(), vec![1, 2, 3]);
}

// Test 4: Clone RxSubject shares state
#[test]
fn test_clone_subject_shares_state() {
    let tracker = DisposableTracker::new();
    let rx1 = RxSubject::new();
    let rx2 = rx1.clone();

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    rx1.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count1_clone.borrow_mut() += 1;
    });

    rx2.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count2_clone.borrow_mut() += 1;
    });

    // Emit on first clone
    rx1.next(42);

    // Both subscribers should receive
    assert_eq!(*count1.borrow(), 1);
    assert_eq!(*count2.borrow(), 1);

    // Emit on second clone
    rx2.next(100);

    // Both subscribers should receive again
    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 2);
}

// Test 5: Multiple subscribers all notified
#[test]
fn test_multiple_subscribers() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count1_clone.borrow_mut() += 1;
    });

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count2_clone.borrow_mut() += 1;
    });

    // Neither called yet
    assert_eq!(*count1.borrow(), 0);
    assert_eq!(*count2.borrow(), 0);

    rx.next(42);

    // Both called on event
    assert_eq!(*count1.borrow(), 1);
    assert_eq!(*count2.borrow(), 1);
}

// Test 7: Tracker drop cleans up
#[test]
fn test_tracker_cleanup() {
    let rx = RxSubject::new();
    let call_count = Rc::new(RefCell::new(0));

    {
        let tracker = DisposableTracker::new();
        let call_count_clone = call_count.clone();

        rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
            *call_count_clone.borrow_mut() += 1;
        });

        // Not called yet
        assert_eq!(*call_count.borrow(), 0);

        rx.next(1);
        assert_eq!(*call_count.borrow(), 1);

        // Tracker drops here
    }

    // After tracker dropped, subscriber should not be called
    rx.next(2);
    assert_eq!(*call_count.borrow(), 1); // Still 1, not 2
}

// Test 8: DisposableTracker.dispose()
#[test]
fn test_disposable_tracker_dispose() {
    let mut tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *call_count_clone.borrow_mut() += 1;
    });

    rx.next(1);
    assert_eq!(*call_count.borrow(), 1);

    // Manually dispose
    tracker.dispose();

    // After disposal, no more calls
    rx.next(2);
    assert_eq!(*call_count.borrow(), 1); // Still 1
}

// Test 9: Multiple trackers
#[test]
fn test_multiple_trackers() {
    let rx = RxSubject::new();

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    {
        let tracker1 = DisposableTracker::new();
        let count1_clone = count1.clone();

        rx.observable().subscribe(tracker1.tracker(), move |_: &i32| {
            *count1_clone.borrow_mut() += 1;
        });

        {
            let tracker2 = DisposableTracker::new();
            let count2_clone = count2.clone();

            rx.observable().subscribe(tracker2.tracker(), move |_: &i32| {
                *count2_clone.borrow_mut() += 1;
            });

            rx.next(1);

            // Both subscriptions active
            assert_eq!(*count1.borrow(), 1);
            assert_eq!(*count2.borrow(), 1);

            // tracker2 drops here
        }

        rx.next(2);

        // Only tracker1's subscription active
        assert_eq!(*count1.borrow(), 2);
        assert_eq!(*count2.borrow(), 1); // Still 1

        // tracker1 drops here
    }

    rx.next(3);

    // No subscriptions active
    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 1);
}

// Test 10: Resubscribe after dispose
#[test]
fn test_resubscribe_after_dispose() {
    let mut tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let count = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count_clone.borrow_mut() += 1;
    });

    rx.next(1);
    assert_eq!(*count.borrow(), 1);

    tracker.dispose();

    rx.next(2);
    assert_eq!(*count.borrow(), 1); // No change

    // Resubscribe with same tracker
    let count_clone2 = count.clone();
    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *count_clone2.borrow_mut() += 1;
    });

    // NOT called immediately (no current value)
    assert_eq!(*count.borrow(), 1);

    rx.next(3);
    assert_eq!(*count.borrow(), 2);
}

// Test 11: Same value emits every time (no deduplication)
#[test]
fn test_same_value_emits_every_time() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.observable().subscribe(tracker.tracker(), move |_: &i32| {
        *call_count_clone.borrow_mut() += 1;
    });

    // Emit same value multiple times
    rx.next(42);
    assert_eq!(*call_count.borrow(), 1);

    rx.next(42);
    assert_eq!(*call_count.borrow(), 2);

    rx.next(42);
    assert_eq!(*call_count.borrow(), 3);

    // Unlike RxRef, there's NO deduplication - every emit fires
}

// Test 14: Clone observable shares state
#[test]
fn test_clone_observable_shares_state() {
    let tracker = DisposableTracker::new();
    let rx = RxSubject::new();

    let obs1 = rx.observable();
    let obs2 = obs1.clone();

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    obs1.subscribe(tracker.tracker(), move |_: &i32| {
        *count1_clone.borrow_mut() += 1;
    });

    obs2.subscribe(tracker.tracker(), move |_: &i32| {
        *count2_clone.borrow_mut() += 1;
    });

    // Emit from subject
    rx.next(42);

    // Both cloned observables receive the event
    assert_eq!(*count1.borrow(), 1);
    assert_eq!(*count2.borrow(), 1);
}
