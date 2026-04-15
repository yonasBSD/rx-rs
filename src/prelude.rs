//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types from rx-rs.
//! You can import everything with:
//!
//! ```
//! use rx_rs::prelude::*;
//! ```

pub use crate::core::{DisposableTracker, RxObservable, RxRef, RxSubject, RxVal, Tracker};
