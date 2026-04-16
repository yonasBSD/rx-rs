use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// RxVal.stream() does NOT emit current value immediately
#[test]
fn test_rx_val_stream_does_not_emit_immediately() {
    let tracker = DisposableTracker::new();
    let rx_ref = RxRef::new(42);
    let stream = rx_ref.val().stream();

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    stream.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    // Should NOT emit immediately - stream only emits on changes
    assert_eq!(*values.borrow(), vec![]);

    // But when we change the value, it should emit
    rx_ref.set(100);
    assert_eq!(*values.borrow(), vec![100]);
}

// RxVal.stream() emits only on changes
#[test]
fn test_rx_val_stream_emits_on_change() {
    let tracker = DisposableTracker::new();
    let rx_ref = RxRef::new(0);
    let stream = rx_ref.val().stream();

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    stream.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    rx_ref.set(1);
    rx_ref.set(2);
    rx_ref.set(3);

    // Note: Does not include initial 0, only the changes
    assert_eq!(*values.borrow(), vec![1, 2, 3]);
}

// RxRef.stream() works the same
#[test]
fn test_rx_ref_stream() {
    let tracker = DisposableTracker::new();
    let rx_ref = RxRef::new("initial");
    let stream = rx_ref.stream();

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    stream.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    rx_ref.set("changed");

    // Does not include "initial", only the change
    assert_eq!(*values.borrow(), vec!["changed"]);
}
