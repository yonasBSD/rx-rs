use rx_rs::core::{DisposableTracker, RxSubject};
use std::cell::RefCell;
use std::rc::Rc;

// Test 1: RxObservable.joinObservable() merges two observables
#[test]
fn test_rx_observable_join_observable() {
    let tracker = DisposableTracker::new();
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();

    let joined = subject1.observable().joinObservable(subject2.observable());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    joined.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject1.next(1);
    subject2.next(2);
    subject1.next(3);
    subject2.next(4);

    assert_eq!(*values.borrow(), vec![1, 2, 3, 4]);
}

// Test 2: RxObservable.joinSubject() works
#[test]
fn test_rx_observable_join_subject() {
    let tracker = DisposableTracker::new();
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();

    let joined = subject1.observable().joinSubject(subject2.clone());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    joined.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject1.next(10);
    subject2.next(20);
    subject1.next(30);

    assert_eq!(*values.borrow(), vec![10, 20, 30]);
}

// Test 3: RxSubject.joinObservable() works
#[test]
fn test_rx_subject_join_observable() {
    let tracker = DisposableTracker::new();
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();

    let joined = subject1.joinObservable(subject2.observable());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    joined.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject1.next("a");
    subject2.next("b");
    subject1.next("c");

    assert_eq!(*values.borrow(), vec!["a", "b", "c"]);
}

// Test 4: RxSubject.joinSubject() works
#[test]
fn test_rx_subject_join_subject() {
    let tracker = DisposableTracker::new();
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();

    let joined = subject1.joinSubject(subject2.clone());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    joined.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject1.next(100);
    subject2.next(200);
    subject2.next(300);
    subject1.next(400);

    assert_eq!(*values.borrow(), vec![100, 200, 300, 400]);
}

// Test 5: Join multiple observables
#[test]
fn test_join_multiple() {
    let tracker = DisposableTracker::new();
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();
    let subject3 = RxSubject::new();

    let joined12 = subject1.joinSubject(subject2.clone());
    let joined_all = joined12.joinSubject(subject3.clone());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    joined_all.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject1.next(1);
    subject2.next(2);
    subject3.next(3);
    subject1.next(4);

    assert_eq!(*values.borrow(), vec![1, 2, 3, 4]);
}
