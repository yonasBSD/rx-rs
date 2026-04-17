use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Basic get/set
#[test]
fn test_basic_get_set() {
    let rx = RxRef::new(42);
    assert_eq!(rx.get(), 42);

    rx.set(100);
    assert_eq!(rx.get(), 100);
}

// Immediate subscription call
#[test]
fn test_immediate_subscription_call() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(42);

    let called = Rc::new(RefCell::new(false));
    let called_clone = called.clone();

    rx.val().subscribe(tracker.tracker(), move |_| {
        *called_clone.borrow_mut() = true;
    });

    // Should be called immediately
    assert!(*called.borrow());
}

// Multiple updates
#[test]
fn test_subscriber_receives_updates() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    rx.val().subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    rx.set(1);
    rx.set(2);
    rx.set(3);

    assert_eq!(*values.borrow(), vec![0, 1, 2, 3]);
}

// Multiple subscribers all notified
#[test]
fn test_multiple_subscribers() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    rx.val().subscribe(tracker.tracker(), move |_| {
        *count1_clone.borrow_mut() += 1;
    });

    rx.val().subscribe(tracker.tracker(), move |_| {
        *count2_clone.borrow_mut() += 1;
    });

    // Both called immediately
    assert_eq!(*count1.borrow(), 1);
    assert_eq!(*count2.borrow(), 1);

    rx.set(42);

    // Both called on update
    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 2);
}

// Same value does NOT trigger (deduplication)
#[test]
fn test_same_value_does_not_trigger() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(42);

    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.val().subscribe(tracker.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1); // Immediate call

    // Setting same value should NOT trigger
    rx.set(42);
    assert_eq!(*call_count.borrow(), 1); // Still 1

    rx.set(42);
    assert_eq!(*call_count.borrow(), 1); // Still 1

    // Setting different value SHOULD trigger
    rx.set(100);
    assert_eq!(*call_count.borrow(), 2);

    // Back to same value, no trigger
    rx.set(100);
    assert_eq!(*call_count.borrow(), 2);
}

// Option types
#[test]
fn test_option_type() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(None::<i32>);

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    rx.val().subscribe(tracker.tracker(), move |opt| {
        values_clone.borrow_mut().push(*opt);
    });

    rx.set(Some(42));
    rx.set(None);
    rx.set(Some(100));

    assert_eq!(*values.borrow(), vec![None, Some(42), None, Some(100)]);
}

// Modify triggers subscribers
#[test]
fn test_modify_triggers_subscribers() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    rx.val().subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    rx.modify(|val| *val += 10);
    rx.modify(|val| *val *= 2);

    assert_eq!(*values.borrow(), vec![0, 10, 20]);
}

#[test]
fn test_dropped_mapped_rx_to_drop_sub() {
    let tracker = DisposableTracker::new();
    let rx = RxRef::new(0);

    {
        let mapped = rx.map(|a| a * a);
        mapped.subscribe(tracker.tracker(), |a| println!("{a}"));
        assert_eq!(rx.subscriber_count(), 1);
    }

    assert_eq!(rx.subscriber_count(), 0);
    rx.val().subscribe(tracker.tracker(), |a| println!("{a}"));
}
