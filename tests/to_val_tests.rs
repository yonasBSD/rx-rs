use rx_rs::core::{DisposableTracker, RxSubject};
use std::cell::RefCell;
use std::rc::Rc;

// Test 1: RxObservable.toVal() creates val with initial value
#[test]
fn test_observable_to_val_initial() {
    let mut tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.observable().toVal(0, tracker.tracker());

    assert_eq!(val.get(), 0);
}

// Test 2: RxObservable.toVal() updates on emissions
#[test]
fn test_observable_to_val_updates() {
    let mut tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.observable().toVal(0, tracker.tracker());

    subject.next(42);
    assert_eq!(val.get(), 42);

    subject.next(100);
    assert_eq!(val.get(), 100);
}

// Test 3: RxSubject.toVal() works the same
#[test]
fn test_subject_to_val() {
    let mut tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.toVal("initial", tracker.tracker());

    assert_eq!(val.get(), "initial");

    subject.next("changed");
    assert_eq!(val.get(), "changed");
}

// Test 4: toVal() stops updating when tracker is disposed
#[test]
fn test_to_val_tracker_disposal() {
    let mut tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.toVal(0, tracker.tracker());

    subject.next(1);
    assert_eq!(val.get(), 1);

    tracker.dispose();

    subject.next(2);
    assert_eq!(val.get(), 1); // Still 1, not 2
}
