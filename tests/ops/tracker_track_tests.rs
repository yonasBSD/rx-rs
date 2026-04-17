use rx_rs::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

// Test that tracking a DisposableTracker disposes it when parent disposes
#[test]
fn test_track_disposes_child_on_parent_dispose() {
    let mut parent = DisposableTracker::new();
    let child = DisposableTracker::new();

    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    // Subscribe with child tracker
    rx.val().subscribe(child.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1); // Initial call

    // Track child with parent tracker
    parent.tracker().track(child);

    rx.set(1);
    assert_eq!(*call_count.borrow(), 2); // Child subscription still active

    // Dispose parent
    parent.dispose();

    // Child should also be disposed
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // No new call
}

// Test that tracking works when parent is dropped
#[test]
fn test_track_disposes_child_on_parent_drop() {
    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));

    {
        let parent = DisposableTracker::new();
        let child = DisposableTracker::new();

        let call_count_clone = call_count.clone();
        rx.val().subscribe(child.tracker(), move |_| {
            *call_count_clone.borrow_mut() += 1;
        });

        assert_eq!(*call_count.borrow(), 1); // Initial call

        parent.tracker().track(child);

        rx.set(1);
        assert_eq!(*call_count.borrow(), 2);

        // Parent drops here
    }

    // Child should be disposed when parent dropped
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // No new call
}

// Test tracking multiple children
#[test]
fn test_track_multiple_children() {
    let mut parent = DisposableTracker::new();
    let child1 = DisposableTracker::new();
    let child2 = DisposableTracker::new();

    let rx1 = RxRef::new(0);
    let rx2 = RxRef::new(0);

    let count1 = Rc::new(RefCell::new(0));
    let count2 = Rc::new(RefCell::new(0));

    let count1_clone = count1.clone();
    let count2_clone = count2.clone();

    rx1.val().subscribe(child1.tracker(), move |_| {
        *count1_clone.borrow_mut() += 1;
    });

    rx2.val().subscribe(child2.tracker(), move |_| {
        *count2_clone.borrow_mut() += 1;
    });

    assert_eq!(*count1.borrow(), 1);
    assert_eq!(*count2.borrow(), 1);

    parent.tracker().track(child1);
    parent.tracker().track(child2);

    rx1.set(1);
    rx2.set(1);
    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 2);

    // Dispose parent
    parent.dispose();

    // Both children should be disposed
    rx1.set(2);
    rx2.set(2);
    assert_eq!(*count1.borrow(), 2);
    assert_eq!(*count2.borrow(), 2);
}

// Test hierarchical tracking (grandparent -> parent -> child)
#[test]
fn test_hierarchical_tracking() {
    let mut grandparent = DisposableTracker::new();
    let parent = DisposableTracker::new();
    let child = DisposableTracker::new();

    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.val().subscribe(child.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1);

    // Create hierarchy: grandparent -> parent -> child
    parent.tracker().track(child);
    grandparent.tracker().track(parent);

    rx.set(1);
    assert_eq!(*call_count.borrow(), 2);

    // Dispose grandparent
    grandparent.dispose();

    // Should cascade down and dispose child
    rx.set(2);
    assert_eq!(*call_count.borrow(), 2);
}

// Test that dispose is idempotent (disposing twice is safe)
#[test]
fn test_double_dispose_is_safe() {
    let mut parent = DisposableTracker::new();
    let child = DisposableTracker::new();

    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.val().subscribe(child.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1);

    parent.tracker().track(child);

    rx.set(1);
    assert_eq!(*call_count.borrow(), 2);

    // Dispose parent (which disposes child)
    parent.dispose();

    rx.set(2);
    assert_eq!(*call_count.borrow(), 2); // Child disposed

    // Disposing parent again should be safe
    parent.dispose();

    rx.set(3);
    assert_eq!(*call_count.borrow(), 2);
}

// Test tracking with resubscription
#[test]
fn test_track_with_resubscription() {
    let mut parent = DisposableTracker::new();
    let child = DisposableTracker::new();

    let rx = RxRef::new(0);
    let call_count = Rc::new(RefCell::new(0));
    let call_count_clone = call_count.clone();

    rx.val().subscribe(child.tracker(), move |_| {
        *call_count_clone.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 1);

    parent.tracker().track(child);

    rx.set(1);
    assert_eq!(*call_count.borrow(), 2);

    // Dispose parent (and child)
    parent.dispose();

    // Create new child and resubscribe
    let new_child = DisposableTracker::new();
    let call_count_clone2 = call_count.clone();

    rx.val().subscribe(new_child.tracker(), move |_| {
        *call_count_clone2.borrow_mut() += 1;
    });

    assert_eq!(*call_count.borrow(), 3); // Immediate call from new subscription

    parent.tracker().track(new_child);

    rx.set(2);
    assert_eq!(*call_count.borrow(), 4);
}

// Test that tracking doesn't affect parent's own subscriptions
#[test]
fn test_track_independent_of_parent_subscriptions() {
    let mut parent = DisposableTracker::new();
    let child = DisposableTracker::new();

    let rx1 = RxRef::new(0);
    let rx2 = RxRef::new(0);

    let count_parent = Rc::new(RefCell::new(0));
    let count_child = Rc::new(RefCell::new(0));

    let count_parent_clone = count_parent.clone();
    let count_child_clone = count_child.clone();

    // Parent has its own subscription
    rx1.val().subscribe(parent.tracker(), move |_| {
        *count_parent_clone.borrow_mut() += 1;
    });

    // Child has its own subscription
    rx2.val().subscribe(child.tracker(), move |_| {
        *count_child_clone.borrow_mut() += 1;
    });

    assert_eq!(*count_parent.borrow(), 1);
    assert_eq!(*count_child.borrow(), 1);

    parent.tracker().track(child);

    rx1.set(1);
    rx2.set(1);
    assert_eq!(*count_parent.borrow(), 2);
    assert_eq!(*count_child.borrow(), 2);

    // Dispose parent
    parent.dispose();

    // Both parent and child subscriptions should be disposed
    rx1.set(2);
    rx2.set(2);
    assert_eq!(*count_parent.borrow(), 2);
    assert_eq!(*count_child.borrow(), 2);
}
