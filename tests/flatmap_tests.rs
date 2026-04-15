use rx_rs::core::{DisposableTracker, RxRef, RxSubject};
use std::cell::RefCell;
use std::rc::Rc;

// Test 1: RxVal.flatMap() switches to new inner RxVal
#[test]
fn test_rx_val_flatmap() {
    let outer = RxRef::new(1);
    let inner1 = RxRef::new(10);
    let inner2 = RxRef::new(20);

    let flattened = outer.val().flatMap(move |&x| {
        if x == 1 {
            inner1.val()
        } else {
            inner2.val()
        }
    });

    assert_eq!(flattened.get(), 10);

    // Change outer - should switch to inner2
    outer.set(2);
    assert_eq!(flattened.get(), 20);

    // Change back to inner1
    outer.set(1);
    assert_eq!(flattened.get(), 10);
}

// Test 2: RxVal.flatMap() subscribes to inner changes
#[test]
fn test_rx_val_flatmap_inner_changes() {
    let outer = RxRef::new(1);
    let inner1 = RxRef::new(10);
    let inner2 = RxRef::new(20);

    let inner1_clone = inner1.clone();
    let inner2_clone = inner2.clone();

    let flattened = outer.val().flatMap(move |&x| {
        if x == 1 {
            inner1_clone.val()
        } else {
            inner2_clone.val()
        }
    });

    assert_eq!(flattened.get(), 10);

    // Change inner1 while it's active
    inner1.set(15);
    assert_eq!(flattened.get(), 15);

    // Switch to inner2
    outer.set(2);
    assert_eq!(flattened.get(), 20);

    // Change inner1 while inner2 is active - should not affect result
    inner1.set(99);
    assert_eq!(flattened.get(), 20);

    // Change inner2 while it's active
    inner2.set(25);
    assert_eq!(flattened.get(), 25);
}

// Test 3: RxRef.flatMap() works the same
#[test]
fn test_rx_ref_flatmap() {
    let selector = RxRef::new(0);
    let values = vec![RxRef::new(100), RxRef::new(200), RxRef::new(300)];

    let flattened = selector.flatMap(move |&idx| values[idx].val());

    assert_eq!(flattened.get(), 100);

    selector.set(1);
    assert_eq!(flattened.get(), 200);

    selector.set(2);
    assert_eq!(flattened.get(), 300);
}

// Test 4: RxObservable.flatMapVal() emits from inner RxVal
#[test]
fn test_rx_observable_flatmap_val() {
    let tracker = DisposableTracker::new();
    let subject = RxSubject::new();
    let inner = RxRef::new(100);

    let inner_clone = inner.clone();
    let flattened = subject.observable().flatMapVal(move |_| inner_clone.val());

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    flattened.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    // Emit from subject - should emit inner's current value
    subject.next(1);
    // Note: This emits twice because flatMapVal both emits the current value
    // AND subscribes to future changes
    assert_eq!(*values.borrow(), vec![100, 100]);

    // Change inner - should emit
    inner.set(200);
    assert_eq!(*values.borrow(), vec![100, 100, 200]);

    // Emit from subject again - should emit current inner value twice
    subject.next(2);
    assert_eq!(*values.borrow(), vec![100, 100, 200, 200, 200]);
}

// Test 5: RxVal.flatMapRef() works with RxRef
#[test]
fn test_rx_val_flatmap_ref() {
    let outer = RxRef::new(0);
    let inner1 = RxRef::new("first");
    let inner2 = RxRef::new("second");

    let flattened = outer.val().flatMapRef(move |&x| {
        if x == 0 {
            inner1.clone()
        } else {
            inner2.clone()
        }
    });

    assert_eq!(flattened.get(), "first");

    outer.set(1);
    assert_eq!(flattened.get(), "second");
}

// Test 6: RxVal.flatMapObservable() switches observables
#[test]
fn test_rx_val_flatmap_observable() {
    let tracker = DisposableTracker::new();
    let outer = RxRef::new(1);
    let subject1 = RxSubject::new();
    let subject2 = RxSubject::new();

    let sub1_clone = subject1.clone();
    let sub2_clone = subject2.clone();

    let flattened = outer.val().flatMapObservable(move |&x| {
        if x == 1 {
            sub1_clone.observable()
        } else {
            sub2_clone.observable()
        }
    });

    let values = Rc::new(RefCell::new(Vec::new()));
    let values_clone = values.clone();

    flattened.subscribe(tracker.tracker(), move |val| {
        values_clone.borrow_mut().push(*val);
    });

    // Emit from subject1
    subject1.next(10);
    assert_eq!(*values.borrow(), vec![10]);

    // Switch to subject2
    outer.set(2);

    // Emit from subject2
    subject2.next(20);
    assert_eq!(*values.borrow(), vec![10, 20]);

    // Emit from subject1 again - should not appear (not subscribed anymore)
    subject1.next(99);
    assert_eq!(*values.borrow(), vec![10, 20]);
}
