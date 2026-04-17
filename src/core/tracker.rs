use std::cell::RefCell;
use std::rc::Rc;

type Cleanups = Rc<RefCell<Vec<Box<dyn FnOnce()>>>>;

/// Tracks reactive subscriptions and automatically cleans them up when dropped.
///
/// Subscriptions registered with a Tracker will be automatically disposed when
/// the Tracker is dropped, preventing memory leaks and ensuring proper cleanup.
///
/// Note: Tracker cannot be created directly. Use `DisposableTracker::new()` and
/// get the Tracker via `.tracker()`.
pub struct Tracker {
    cleanups: Cleanups,
}

impl Tracker {
    /// Creates a new empty Tracker.
    ///
    /// This is private - only DisposableTracker can create Trackers.
    pub(super) fn new() -> Self {
        Self {
            cleanups: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Adds a cleanup function to be called when the tracker is dropped.
    ///
    /// This is an internal method used by reactive primitives to register
    /// their cleanup logic.
    pub(crate) fn add<F: FnOnce() + 'static>(&self, cleanup: F) {
        self.cleanups.borrow_mut().push(Box::new(cleanup));
    }

    /// Returns the number of active subscriptions tracked.
    pub fn subscription_count(&self) -> usize {
        self.cleanups.borrow().len()
    }

    /// Tracks another DisposableTracker's lifetime.
    ///
    /// When the Tracker's parent DisposableTracker is disposed (either manually via `dispose()` or
    /// automatically when dropped), the tracked DisposableTracker will also be disposed.
    ///
    /// This is useful for creating hierarchical cleanup relationships.
    ///
    /// # Arguments
    /// * `child` - The DisposableTracker to track
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::DisposableTracker;
    ///
    /// let parent = DisposableTracker::new();
    /// let mut child = DisposableTracker::new();
    ///
    /// parent.tracker().track(child);
    ///
    /// // When parent is disposed, child is also disposed
    /// ```
    pub fn track(&self, mut child: DisposableTracker) {
        self.add(move || {
            child.dispose();
        });
    }
}

impl Clone for Tracker {
    fn clone(&self) -> Self {
        Self {
            cleanups: self.cleanups.clone(),
        }
    }
}

/// A tracker that can be manually disposed before it's dropped.
///
/// Unlike Tracker, DisposableTracker provides a `dispose()` method to
/// explicitly clean up all subscriptions. This is useful for long-lived
/// objects that need to clear subscriptions mid-lifecycle.
pub struct DisposableTracker {
    tracker: Tracker,
}

impl DisposableTracker {
    /// Creates a new empty DisposableTracker.
    pub fn new() -> Self {
        Self {
            tracker: Tracker::new(),
        }
    }

    /// Returns the underlying Tracker for use with subscribe methods.
    pub fn tracker(&self) -> &Tracker {
        &self.tracker
    }

    /// Manually disposes all tracked subscriptions.
    ///
    /// After calling this, the tracker is still valid and can track new
    /// subscriptions, but all previous subscriptions are cleaned up.
    pub fn dispose(&mut self) {
        if let Ok(mut cleanups) = self.tracker.cleanups.try_borrow_mut() {
            for cleanup in cleanups.drain(..) {
                cleanup();
            }
        }
    }

    /// Returns the number of active subscriptions tracked.
    pub fn subscription_count(&self) -> usize {
        self.tracker.subscription_count()
    }
}

impl Default for DisposableTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for DisposableTracker {
    fn drop(&mut self) {
        // Clean up all subscriptions when DisposableTracker is dropped
        if let Ok(mut cleanups) = self.tracker.cleanups.try_borrow_mut() {
            for cleanup in cleanups.drain(..) {
                cleanup();
            }
        }
    }
}
