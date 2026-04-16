use rx_rs::prelude::*;

// RxObservable.to_val() creates val with initial value
#[test]
fn test_observable_to_val_initial() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.observable().to_val(0, tracker.tracker());

    assert_eq!(val.get(), 0);
}

// RxObservable.to_val() updates on emissions
#[test]
fn test_observable_to_val_updates() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.observable().to_val(0, tracker.tracker());

    subject.next(42);
    assert_eq!(val.get(), 42);

    subject.next(100);
    assert_eq!(val.get(), 100);
}

// RxSubject.to_val() works the same
#[test]
fn test_subject_to_val() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.to_val("initial", tracker.tracker());

    assert_eq!(val.get(), "initial");

    subject.next("changed");
    assert_eq!(val.get(), "changed");
}

// toVal() stops updating when tracker is disposed
#[test]
fn test_to_val_tracker_disposal() {
    let mut tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let val = subject.to_val(0, tracker.tracker());

    subject.next(1);
    assert_eq!(val.get(), 1);

    tracker.dispose();

    subject.next(2);
    assert_eq!(val.get(), 1); // Still 1, not 2
}
