use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Test 1: RxVal.map() creates mapped value
#[test]
fn test_rx_val_map() {
    let number = RxRef::new(5);
    let doubled = number.val().map(|x| x * 2);

    assert_eq!(doubled.get(), 10);

    number.set(10);
    assert_eq!(doubled.get(), 20);
}

// Test 2: RxRef.map() works the same
#[test]
fn test_rx_ref_map() {
    let name = RxRef::new("alice");
    let upper = name.map(|s| s.to_uppercase());

    assert_eq!(upper.get(), "ALICE");

    name.set("bob");
    assert_eq!(upper.get(), "BOB");
}

// Test 3: RxObservable.map() transforms emissions
#[test]
fn test_rx_observable_map() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let doubled = subject.observable().map(|x| x * 2);

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    doubled.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject.next(1);
    subject.next(2);
    subject.next(3);

    assert_eq!(*values.borrow(), vec![2, 4, 6]);
}

// Test 4: RxSubject.map() works the same
#[test]
fn test_rx_subject_map() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let doubled = subject.map(|x| x * 2);

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    doubled.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    subject.next(5);
    subject.next(10);

    assert_eq!(*values.borrow(), vec![10, 20]);
}

// Test 5: Multiple maps can be chained
#[test]
fn test_map_chaining() {
    let number = RxRef::new(2);
    let doubled = number.val().map(|x| x * 2);
    let squared = doubled.map(|x| x * x);

    assert_eq!(squared.get(), 16); // (2 * 2) ^ 2 = 16

    number.set(3);
    assert_eq!(squared.get(), 36); // (3 * 2) ^ 2 = 36
}
