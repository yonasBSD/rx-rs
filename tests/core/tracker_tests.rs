use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_tracker_cleanup() {
    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));

    {
        let tracker = DisposableTracker::new();
        let call_count_clone = call_count.clone();

        rx.val().subscribe(tracker.tracker(), move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        // Called immediately
        assert_eq!(*call_count.borrow(), 1);

        rx.set(1);
        assert_eq!(*call_count.borrow(), 2);

        // Tracker drops here
    }

    // After tracker dropped, subscriber should not be called
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // Still 2, not 3
}

#[test]
fn test_disposable_tracker_dispose() {
    let mut tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.val().subscribe(tracker.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1); // Immediate call

    rx.set(1);
    assert_eq!(*call_count.borrow(), 2);

    // Manually dispose
    tracker.dispose();

    // After disposal, no more calls
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // Still 2
}

#[test]
fn test_multiple_trackers() {
    let rx = RxRef::new(0);

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    {
        let tracker1 = DisposableTracker::new();
        let count1_clone = count1.clone();

        rx.val().subscribe(tracker1.tracker(), move |_| {
            *count1_clone.borrow_mut() += 1;
        });

        {
            let tracker2 = DisposableTracker::new();
            let count2_clone = count2.clone();

            rx.val().subscribe(tracker2.tracker(), move |_| {
                *count2_clone.borrow_mut() += 1;
            });

            rx.set(1);

            // Both subscriptions active
            assert_eq!(*count1.borrow(), 2); // immediate + update
            assert_eq!(*count2.borrow(), 2);

            // tracker2 drops here
        }

        rx.set(2);

        // Only tracker1's subscription active
        assert_eq!(*count1.borrow(), 3);
        assert_eq!(*count2.borrow(), 2); // Still 2

        // tracker1 drops here
    }

    rx.set(3);

    // No subscriptions active
    assert_eq!(*count1.borrow(), 3);
    assert_eq!(*count2.borrow(), 2);
}

#[test]
fn test_resubscribe_after_dispose() {
    let mut tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    let count = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    rx.val().subscribe(tracker.tracker(), move |_| {
        *count_clone.borrow_mut() += 1;
    });

    rx.set(1);
    assert_eq!(*count.borrow(), 2);

    tracker.dispose();

    rx.set(2);
    assert_eq!(*count.borrow(), 2); // No change

    // Resubscribe with same tracker
    let count_clone2 = count.clone();
    rx.val().subscribe(tracker.tracker(), move |_| {
        *count_clone2.borrow_mut() += 1;
    });

    // Called immediately with current value
    assert_eq!(*count.borrow(), 3);

    rx.set(3);
    assert_eq!(*count.borrow(), 4);
}
