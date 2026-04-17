use rx_rs::prelude::*;

// ============================================================================
// COMBINED OPERATIONS MEMORY LEAK TESTS
// ============================================================================

#[test]
fn test_combined_operations_leak_potential() {
    let a = RxRef::new(1);
    let b = RxRef::new(2);

    // Create complex dependency graph
    let a_clone = a.clone();
    let ab_sum = b.val().flat_map(move |&b_val| {
        a_clone.val().map(move |&a_val| a_val + b_val)
    });

    let b_clone = b.clone();
    let ab_product = a.val().flat_map(move |&a_val| {
        b_clone.val().map(move |&b_val| a_val * b_val)
    });

    // Clone ab_product before moving it into zip_val
    let ab_product_clone = ab_product.clone();

    // Zip the two derived values
    let combined = ab_sum.zip_val(ab_product_clone);

    assert_eq!(combined.get(), (3, 2)); // (1+2, 1*2)

    a.set(3);
    assert_eq!(combined.get(), (5, 6)); // (3+2, 3*2)

    println!("A subscribers: {}", a.subscriber_count());
    println!("B subscribers: {}", b.subscriber_count());

    drop(combined);
    drop(ab_sum);
    drop(ab_product);

    println!("After drops - A subscribers: {}", a.subscriber_count());
    println!("After drops - B subscribers: {}", b.subscriber_count());
}
