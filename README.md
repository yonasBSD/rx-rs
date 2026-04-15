# rx-rs

A lightweight single-threaded push-based reactive programming library for Rust.

## Quick Start

```rust
use rx_rs::prelude::*;

fn main() {
    let tracker = DisposableTracker::new();

    // Reactive value with current state
    let counter = RxRef::new(0);

    counter.val().subscribe(tracker.tracker(), |value| {
        println!("Counter: {}", value);
    });

    counter.set(1); // Prints: Counter: 1
    counter.set(2); // Prints: Counter: 2
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rx-rs = "0.0.1"
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
