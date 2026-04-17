# Memory Leak Tests

This directory contains tests that validate memory safety and detect potential memory leaks in rx-rs.

## Test Categories

### 1. `flatmap_leaks.rs` - FlatMap Memory Leak Tests
Tests related to `flat_map`, `flat_map_ref`, and `flat_map_observable` operations:
- Self-referencing flat_map (returns same RxVal)
- Circular flat_map chains (A depends on B, B depends on A)
- FlatMap observable with self-subscription

**Key Issues Tested:**
- Reference cycles when flat_map returns the source
- RefCell borrow violations in self-referential patterns
- Subscriber cleanup failures

### 2. `map_leaks.rs` - Map Memory Leak Tests
Tests related to `map` operation:
- Map with closure captures
- Complex map chains
- Independent map operations (baseline)

**Key Issues Tested:**
- Memory leaks when map closures capture the source RxRef
- RefCell panics when closures call `.get()` on captured sources
- Proper cleanup verification

### 3. `zip_leaks.rs` - Zip Memory Leak Tests
Tests related to `zip_val` and `zip_ref` operations:
- Zipping RxVal with itself
- Transitive self-references (zip with derived values)

**Key Issues Tested:**
- Double borrow panics when updating self-zipped values
- Circular subscriptions in zip operations
- Cleanup of dual subscriptions

### 4. `stream_leaks.rs` - Stream Memory Leak Tests
Tests related to `stream()` conversion and stream subscriptions:
- Circular update patterns (stream subscribes and updates source)
- Feedback loops

**Key Issues Tested:**
- Re-entrant calls causing RefCell panics
- Infinite update loops (with guards)
- Subscription lifecycle in streams

### 5. `combined_leaks.rs` - Combined Operations Tests
Tests that combine multiple operations to detect complex leak scenarios:
- Map + Zip combinations
- Complex dependency graphs

**Key Issues Tested:**
- Interaction between multiple operations
- Compound reference cycles
- Multi-source subscriber management

### 6. `diagnostics.rs` - Diagnostic and Baseline Tests
Utility tests that verify proper behavior and provide debugging information:
- Proper cleanup (no leak baseline)
- Subscriber count analysis through operation lifecycle

**Purpose:**
- Establish baseline for proper cleanup behavior
- Track subscriber counts at each stage
- Verify that simple cases work correctly

## Running Memory Tests

```bash
# Run all memory tests
cargo test --test tests memory::

# Run specific category
cargo test --test tests memory::flatmap_leaks::
cargo test --test tests memory::map_leaks::
cargo test --test tests memory::zip_leaks::
cargo test --test tests memory::stream_leaks::
cargo test --test tests memory::combined_leaks::
cargo test --test tests memory::diagnostics::

# Run with backtrace for panic details
RUST_BACKTRACE=1 cargo test --test tests memory::
```

## Current Test Results

**Total Memory Tests:** 13
- **Passing:** 3 (23%)
- **Failing:** 10 (77%)
  - 7 with RefCell panics
  - 2 with memory leak assertions
  - 1 with logic errors

## Known Issues

All documented memory leak and panic scenarios are reproducible. See:
- `/CRITICAL_FINDINGS.md` - Detailed technical analysis
- `/MEMORY_LEAK_ANALYSIS.md` - In-depth leak patterns
- `/LEAK_TEST_CASES.md` - Concrete code examples

## Test Organization Rationale

Tests are split by operation type to:
1. **Improve maintainability** - Easy to locate tests for specific operations
2. **Enable targeted testing** - Run only relevant tests during development
3. **Clarify failure patterns** - Group related failures by category
4. **Facilitate documentation** - Each category maps to specific leak scenarios
5. **Support incremental fixes** - Fix one category at a time

## Adding New Tests

When adding memory leak tests:
1. Choose the appropriate category file based on the primary operation
2. Use descriptive test names: `test_<operation>_<scenario>_<expected_behavior>`
3. Include diagnostic output (println! for subscriber counts)
4. Document the expected behavior in comments
5. Mark known failures with comments explaining the issue

Example:
```rust
#[test]
fn test_map_with_external_state_no_leak() {
    // Create external state (not captured in closure)
    let external = 10;

    let x = RxRef::new(5);
    let mapped = x.val().map(move |v| v + external);

    assert_eq!(mapped.get(), 15);

    drop(mapped);

    // Should clean up since no cycle
    assert_eq!(x.subscriber_count(), 0, "No leak expected");
}
```
