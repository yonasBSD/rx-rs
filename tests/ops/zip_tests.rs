use rx_rs::core::RxRef;

// Test 1: RxVal.zip_val() combines two values
#[test]
fn test_rx_val_zip_val() {
    let name = RxRef::new("Alice");
    let age = RxRef::new(30);

    let combined = name.val().zip_val(age.val());

    assert_eq!(combined.get(), ("Alice", 30));

    name.set("Bob");
    assert_eq!(combined.get(), ("Bob", 30));

    age.set(25);
    assert_eq!(combined.get(), ("Bob", 25));
}

// Test 2: RxVal.zip_ref() works with RxRef
#[test]
fn test_rx_val_zip_ref() {
    let x = RxRef::new(10);
    let y = RxRef::new(20);

    let combined = x.val().zip_ref(y.clone());

    assert_eq!(combined.get(), (10, 20));

    x.set(15);
    assert_eq!(combined.get(), (15, 20));

    y.set(25);
    assert_eq!(combined.get(), (15, 25));
}

// Test 3: RxRef.zip_val() works
#[test]
fn test_rx_ref_zip_val() {
    let a = RxRef::new(true);
    let b = RxRef::new(false);

    let combined = a.zip_val(b.val());

    assert_eq!(combined.get(), (true, false));

    a.set(false);
    assert_eq!(combined.get(), (false, false));
}

// Test 4: RxRef.zip_ref() works
#[test]
fn test_rx_ref_zip_ref() {
    let first = RxRef::new("hello");
    let second = RxRef::new("world");

    let combined = first.zip_ref(second.clone());

    assert_eq!(combined.get(), ("hello", "world"));

    first.set("goodbye");
    assert_eq!(combined.get(), ("goodbye", "world"));

    second.set("moon");
    assert_eq!(combined.get(), ("goodbye", "moon"));
}

// Test 5: Zip with different types
#[test]
fn test_zip_different_types() {
    let number = RxRef::new(42);
    let text = RxRef::new("answer");

    let combined = number.zip_ref(text);

    assert_eq!(combined.get(), (42, "answer"));

    number.set(100);
    assert_eq!(combined.get(), (100, "answer"));
}

// Test 6: Multiple zips
#[test]
fn test_multiple_zips() {
    let a = RxRef::new(1);
    let b = RxRef::new(2);
    let c = RxRef::new(3);

    let ab = a.zip_val(b.val());
    let c_clone = c.clone();
    let abc = ab.map(move |(a, b)| (*a, *b, c_clone.get()));

    assert_eq!(abc.get(), (1, 2, 3));

    a.set(10);
    assert_eq!(abc.get(), (10, 2, 3));

    b.set(20);
    assert_eq!(abc.get(), (10, 20, 3));
}
