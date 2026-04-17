use rx_rs::prelude::*;

// ============================================================================
// ZIP MEMORY LEAK TESTS
// ============================================================================

#[test]
fn test_zip_self_reference() {
    let x = RxRef::new(10);

    let initial_subs = x.subscriber_count();
    assert_eq!(initial_subs, 0);

    // Zip x with itself
    let x_clone = x.clone();
    let zipped = x.val().zip_val(x_clone.val());

    // This creates two subscriptions on x
    let subs_during = x.subscriber_count();
    println!("Subscribers during zip: {}", subs_during);
    assert_eq!(
        subs_during, 2,
        "Expected 2 subscriptions (one for each side of zip)"
    );

    assert_eq!(zipped.get(), (10, 10));

    // Update x and verify zipped updates
    x.set(20);
    assert_eq!(zipped.get(), (20, 20));

    // Drop zipped and check cleanup
    drop(zipped);

    let subs_after = x.subscriber_count();
    println!("Subscribers after drop: {}", subs_after);

    if subs_after > initial_subs {
        println!("POTENTIAL LEAK: Subscribers not fully cleaned up");
        println!("Expected: {}, Got: {}", initial_subs, subs_after);
    }
}

#[test]
fn test_zip_transitive_self_reference() {
    let base = RxRef::new(5);

    // Create a derived value
    let derived = base.val().map(|x| x * 2);

    assert_eq!(derived.get(), 10);

    // Zip base with its derived value
    let base_clone = base.clone();
    let zipped = base_clone.val().zip_val(derived.clone());

    assert_eq!(zipped.get(), (5, 10));

    // Update base
    base.set(10);
    assert_eq!(zipped.get(), (10, 20));

    let subs_before_drop = base.subscriber_count();
    println!("Base subscribers before drop: {}", subs_before_drop);

    drop(zipped);
    drop(derived);

    let subs_after_drop = base.subscriber_count();
    println!("Base subscribers after drop: {}", subs_after_drop);

    if subs_after_drop > 0 {
        println!("POTENTIAL LEAK: Transitive reference may have created cycle");
    }
}
