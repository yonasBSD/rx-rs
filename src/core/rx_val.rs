use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use super::rx_observable::RxObservable;
use super::rx_ref::RxRef;
use super::rx_subject::RxSubject;
use super::tracker::{DisposableTracker, Tracker};

type Subscriber<T> = Rc<RefCell<Box<dyn FnMut(&T)>>>;

/// Internal storage for a reactive value.
#[derive(Clone)]
struct RxValInner<T> {
    value: T,
    subscribers: Vec<Subscriber<T>>,
    // Optional tracker to keep subscriptions alive
    // Used by .map() and other operators to maintain source subscriptions
    // Can store any type that needs to be kept alive
    _lifetime_tracker: Option<Rc<dyn Any>>,
}

/// A read-only reactive value that holds a current state.
///
/// When subscribed, the subscriber is called immediately with the current value,
/// and then called again whenever the value changes.
///
/// This is useful for representing state that always has a current value, like
/// connection status, client ID, or configuration values.
///
/// # Example
/// ```
/// use rx_rs::core::{RxRef, DisposableTracker};
///
/// let mut tracker = DisposableTracker::new();
/// let rx_ref = RxRef::new(42);
/// let rx_val = rx_ref.val();
///
/// rx_val.subscribe(tracker.tracker(), |value| {
///     println!("Value: {}", value);
/// }); // Prints "Value: 42" immediately
///
/// rx_ref.set(100); // Prints "Value: 100"
/// ```
#[derive(Clone)]
pub struct RxVal<T> {
    inner: Rc<RefCell<RxValInner<T>>>,
}

impl<T: 'static> RxVal<T> {
    /// Gets the current value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        self.inner.borrow().value.clone()
    }

    /// Subscribes to value changes.
    ///
    /// The subscriber function is called immediately with the current value,
    /// and then called again whenever the value changes.
    ///
    /// The subscription is automatically cleaned up when the tracker is dropped.
    ///
    /// # Arguments
    /// * `tracker` - Tracker that will manage this subscription's lifetime
    /// * `f` - Function called with a reference to the value on each update
    pub fn subscribe<F>(&self, tracker: &Tracker, mut f: F)
    where
        F: FnMut(&T) + 'static,
        T: Clone,
    {
        // Clone current value and release borrow before calling callback
        let current_value = self.inner.borrow().value.clone();

        // Call immediately with cloned value (no borrow held)
        f(&current_value);

        // Wrap the subscriber in Rc<RefCell<>> for shared ownership
        let subscriber = Rc::new(RefCell::new(Box::new(f) as Box<dyn FnMut(&T)>));

        // Store for future updates
        let subscriber_clone = subscriber.clone();
        let inner_weak = Rc::downgrade(&self.inner);

        // Add subscriber - now this will succeed even if called from within a notification
        self.inner.borrow_mut().subscribers.push(subscriber_clone);

        #[cfg(feature = "debug")]
        {
            let debug_ptr = std::rc::Rc::as_ptr(&self.inner) as usize;
            let sub_count = self.inner.borrow().subscribers.len();
            tracing::debug!(
                ptr = format!("0x{:x}", debug_ptr),
                subscriber_count = sub_count,
                "subscription added to RxVal"
            );
        }

        // Add cleanup to tracker
        #[cfg(feature = "debug")]
        let debug_ptr = std::rc::Rc::as_ptr(&self.inner) as usize;

        tracker.add(move || {
            // Remove subscriber when tracker drops
            // Use weak reference to avoid cycle
            #[cfg(feature = "debug")]
            tracing::debug!(
                ptr = format!("0x{:x}", debug_ptr),
                "subscription cleanup called"
            );

            if let Some(inner_rc) = inner_weak.upgrade() {
                if let Ok(mut inner) = inner_rc.try_borrow_mut() {
                    #[cfg(feature = "debug")]
                    let before_count = inner.subscribers.len();

                    inner.subscribers.retain(|s| !Rc::ptr_eq(s, &subscriber));

                    #[cfg(feature = "debug")]
                    {
                        let after_count = inner.subscribers.len();
                        tracing::debug!(
                            ptr = format!("0x{:x}", debug_ptr),
                            before = before_count,
                            after = after_count,
                            "removed subscriber from RxVal"
                        );
                    }
                }
            } else {
                #[cfg(feature = "debug")]
                tracing::debug!(ptr = format!("0x{:x}", debug_ptr), "RxVal already dropped");
            }
        });
    }

    /// Returns the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.borrow().subscribers.len()
    }

    /// Returns a pointer address for debugging identity.
    /// Use this to check if two RxVal instances share the same underlying data.
    pub fn debug_ptr(&self) -> usize {
        std::rc::Rc::as_ptr(&self.inner) as usize
    }

    /// Converts this RxVal into a stream (RxObservable).
    ///
    /// The returned observable does NOT emit the current value immediately on subscription.
    /// It only emits when the RxVal changes to a new value.
    ///
    /// Note: If you subscribe directly to the RxVal, it WILL emit immediately.
    /// But when converted to a stream, it behaves like a pure observable.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxRef, DisposableTracker};
    ///
    /// let mut tracker = DisposableTracker::new();
    /// let rx_ref = RxRef::new(42);
    /// let stream = rx_ref.val().stream();
    ///
    /// stream.subscribe(tracker.tracker(), |value| {
    ///     println!("Value: {}", value);
    /// }); // Does NOT print immediately
    ///
    /// rx_ref.set(100); // Prints "Value: 100"
    /// ```
    pub fn stream(&self) -> RxObservable<T>
    where
        T: Clone,
    {
        // Create a subject to forward values to
        let subject = RxSubject::new();

        // Create a tracker that will keep the subscription alive
        let tracker = Rc::new(DisposableTracker::new());

        // Create a flag to skip the first emission (current value)
        let first = Rc::new(RefCell::new(true));

        // Subscribe to this RxVal and forward all values to the subject
        // BUT skip the immediate emission of the current value
        let subject_clone = subject.clone();
        self.subscribe(tracker.tracker(), move |value| {
            if *first.borrow() {
                *first.borrow_mut() = false;
                return; // Skip the first emission (current value)
            }
            subject_clone.next(value.clone());
        });

        // Get the inner from the subject's observable
        let subject_observable = subject.observable();

        // Attach the tracker to keep subscription alive
        subject_observable.inner.borrow_mut()._lifetime_tracker = Some(tracker as Rc<dyn Any>);

        subject_observable
    }

    /// Maps the values of this RxVal using a transformation function.
    ///
    /// Returns a new RxVal that always contains the transformed value.
    /// When the source RxVal changes, the transformation is applied and
    /// the resulting RxVal is updated.
    ///
    /// # Arguments
    /// * `f` - Function to transform values from A to B
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let number = RxRef::new(5);
    /// let doubled = number.val().map(|x| x * 2);
    ///
    /// assert_eq!(doubled.get(), 10);
    ///
    /// number.set(10);
    /// assert_eq!(doubled.get(), 20);
    /// ```
    pub fn map<B, F>(&self, f: F) -> RxVal<B>
    where
        T: Clone,
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> B + 'static,
    {
        // Create the initial mapped value
        let initial = f(&self.get());

        // Create a tracker that will live as long as the returned RxVal
        let tracker = Rc::new(DisposableTracker::new());

        // Create a new RxRef to hold the mapped values
        let rx_ref = RxRef::new(initial);

        // Get the result val first
        let mapped_val = rx_ref.val();

        // Subscribe to this RxVal and update the mapped ref
        // Use weak reference to result's inner to avoid cycle
        let result_weak = Rc::downgrade(&mapped_val.inner);
        self.subscribe(tracker.tracker(), move |value| {
            if let Some(result_inner) = result_weak.upgrade() {
                let new_value = f(value);
                let mut inner = result_inner.borrow_mut();
                if inner.value != new_value {
                    inner.value = new_value.clone();
                    // Notify subscribers
                    for subscriber in &inner.subscribers {
                        let mut sub = subscriber.borrow_mut();
                        sub(&new_value);
                    }
                }
            }
        });

        // Attach the tracker to keep subscription alive
        mapped_val.inner.borrow_mut()._lifetime_tracker = Some(tracker as Rc<dyn Any>);

        mapped_val
    }

    /// Flat-maps the values of this RxVal using a function that returns RxVal<B>.
    ///
    /// When the source RxVal changes, the function is called to produce a new RxVal<B>,
    /// and the result RxVal is updated to reflect the current value of that inner RxVal.
    /// The result also updates when the inner RxVal changes.
    ///
    /// # Arguments
    /// * `f` - Function to transform values from A to RxVal<B>
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let outer = RxRef::new(1);
    /// let inner1 = RxRef::new(10);
    /// let inner2 = RxRef::new(20);
    ///
    /// let inner1_clone = inner1.clone();
    /// let inner2_clone = inner2.clone();
    ///
    /// let flattened = outer.val().flat_map(move |&x| {
    ///     if x == 1 { inner1_clone.val() } else { inner2_clone.val() }
    /// });
    ///
    /// assert_eq!(flattened.get(), 10);
    ///
    /// outer.set(2);
    /// assert_eq!(flattened.get(), 20);
    /// ```
    pub fn flat_map<B, F>(&self, f: F) -> RxVal<B>
    where
        T: Clone + PartialEq,
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxVal<B> + 'static,
    {
        use super::tracker::DisposableTracker;

        // Get initial inner RxVal
        let initial_inner = f(&self.get());
        let initial_value = initial_inner.get();

        // Create RxRef to hold the result
        let result_ref = RxRef::new(initial_value);

        // Create tracker for the outer subscription
        let outer_tracker = Rc::new(DisposableTracker::new());

        // Track the current inner subscription
        let inner_tracker = Rc::new(RefCell::new(DisposableTracker::new()));

        // Store the current inner RxVal to keep it alive
        let current_inner = Rc::new(RefCell::new(Some(initial_inner.clone())));

        // Get result val first
        let result_val = result_ref.val();

        // Subscribe to the initial inner RxVal
        let result_weak_init = Rc::downgrade(&result_val.inner);
        initial_inner.subscribe(inner_tracker.borrow().tracker(), move |inner_value| {
            if let Some(result_inner) = result_weak_init.upgrade() {
                let mut inner = result_inner.borrow_mut();
                if inner.value != *inner_value {
                    inner.value = inner_value.clone();
                    // Notify subscribers
                    for subscriber in &inner.subscribers {
                        let mut sub = subscriber.borrow_mut();
                        sub(inner_value);
                    }
                }
            }
        });

        // Track the last outer value to avoid re-subscribing on duplicate updates
        let last_outer_value = Rc::new(RefCell::new(self.get()));

        // Subscribe to the outer RxVal using weak reference to result
        let result_weak = Rc::downgrade(&result_val.inner);
        let inner_tracker_clone = inner_tracker.clone();
        let current_inner_clone = current_inner.clone();
        let f_clone = Rc::new(f);

        self.subscribe(outer_tracker.tracker(), move |outer_value| {
            if let Some(result_inner) = result_weak.upgrade() {
                // Only recreate inner subscription if outer value changed
                let should_update = {
                    let mut last_val = last_outer_value.borrow_mut();
                    if *last_val != *outer_value {
                        *last_val = outer_value.clone();
                        true
                    } else {
                        false
                    }
                };

                if !should_update {
                    return; // Same value, don't recreate subscription
                }

                // Get new inner RxVal
                let new_inner = f_clone(outer_value);

                // Cancel previous inner subscription
                inner_tracker_clone.borrow_mut().dispose();
                *inner_tracker_clone.borrow_mut() = DisposableTracker::new();

                // Set to the new inner's current value
                let new_value = new_inner.get();
                {
                    let mut inner = result_inner.borrow_mut();
                    if inner.value != new_value {
                        inner.value = new_value.clone();
                        // Notify subscribers
                        for subscriber in &inner.subscribers {
                            let mut sub = subscriber.borrow_mut();
                            sub(&new_value);
                        }
                    }
                }

                // Subscribe to the new inner using weak reference
                let result_weak2 = Rc::downgrade(&result_inner);
                new_inner.subscribe(inner_tracker_clone.borrow().tracker(), move |inner_value| {
                    if let Some(result_inner2) = result_weak2.upgrade() {
                        let mut inner = result_inner2.borrow_mut();
                        if inner.value != *inner_value {
                            inner.value = inner_value.clone();
                            // Notify subscribers
                            for subscriber in &inner.subscribers {
                                let mut sub = subscriber.borrow_mut();
                                sub(inner_value);
                            }
                        }
                    }
                });

                // Update current_inner to keep the new one alive
                *current_inner_clone.borrow_mut() = Some(new_inner);
            }
        });

        // We need to keep both trackers and current_inner alive
        // Store them in a combined structure
        let combined_tracker = Rc::new((outer_tracker, inner_tracker, current_inner));
        result_val.inner.borrow_mut()._lifetime_tracker =
            Some(combined_tracker as Rc<dyn std::any::Any>);

        result_val
    }

    /// Flat-maps using a function that returns RxRef<B>.
    /// Delegates to flat_map by converting the RxRef to RxVal.
    pub fn flat_map_ref<B, F>(&self, f: F) -> RxVal<B>
    where
        T: Clone + PartialEq,
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxRef<B> + 'static,
    {
        self.flat_map(move |x| f(x).val())
    }

    /// Flat-maps using a function that returns RxObservable<B>.
    /// Returns an RxObservable that switches to the new observable on each change.
    pub fn flat_map_observable<B, F>(&self, f: F) -> RxObservable<B>
    where
        T: Clone,
        B: Clone + 'static,
        F: Fn(&T) -> RxObservable<B> + 'static,
    {
        use super::rx_subject::RxSubject;
        use super::tracker::DisposableTracker;

        let subject = RxSubject::new();
        let outer_tracker = Rc::new(DisposableTracker::new());
        let inner_tracker = Rc::new(RefCell::new(DisposableTracker::new()));

        let subject_clone = subject.clone();
        let inner_tracker_clone = inner_tracker.clone();
        let f_rc = Rc::new(f);

        self.subscribe(outer_tracker.tracker(), move |outer_value| {
            let new_inner = f_rc(outer_value);
            inner_tracker_clone.borrow_mut().dispose();
            *inner_tracker_clone.borrow_mut() = DisposableTracker::new();

            let subject_clone2 = subject_clone.clone();
            new_inner.subscribe(inner_tracker_clone.borrow().tracker(), move |inner_value| {
                subject_clone2.next(inner_value.clone());
            });
        });

        let observable = subject.observable();
        let combined_tracker = Rc::new((outer_tracker, inner_tracker));
        observable.inner.borrow_mut()._lifetime_tracker = Some(combined_tracker as Rc<dyn Any>);

        observable
    }

    /// Flat-maps using a function that returns RxSubject<B>.
    /// Delegates to flat_map_observable by converting the RxSubject to RxObservable.
    pub fn flat_map_subject<B, F>(&self, f: F) -> RxObservable<B>
    where
        T: Clone,
        B: Clone + 'static,
        F: Fn(&T) -> RxSubject<B> + 'static,
    {
        self.flat_map_observable(move |x| f(x).observable())
    }

    /// Combines this RxVal with another RxVal, producing a new RxVal containing a tuple.
    ///
    /// The resulting RxVal updates whenever either source changes.
    ///
    /// # Arguments
    /// * `other` - Another RxVal to combine with
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let name = RxRef::new("Alice");
    /// let age = RxRef::new(30);
    ///
    /// let combined = name.val().zip_val(age.val());
    ///
    /// assert_eq!(combined.get(), ("Alice", 30));
    ///
    /// name.set("Bob");
    /// assert_eq!(combined.get(), ("Bob", 30));
    /// ```
    pub fn zip_val<U>(&self, other: RxVal<U>) -> RxVal<(T, U)>
    where
        T: Clone + PartialEq,
        U: Clone + PartialEq + 'static,
    {
        // Create initial combined value
        let initial = (self.get(), other.get());

        // Create RxRef to hold the zipped values
        let result_ref = RxRef::new(initial);

        // Create trackers
        let tracker1 = Rc::new(DisposableTracker::new());
        let tracker2 = Rc::new(DisposableTracker::new());

        // Get result val first
        let result_val = result_ref.val();

        // Subscribe to self using weak references
        let result_weak1 = Rc::downgrade(&result_val.inner);
        let other_clone1 = other.clone();
        self.subscribe(tracker1.tracker(), move |self_val| {
            if let Some(result_inner) = result_weak1.upgrade() {
                let new_value = (self_val.clone(), other_clone1.get());
                let mut inner = result_inner.borrow_mut();
                if inner.value != new_value {
                    inner.value = new_value.clone();
                    // Notify subscribers
                    for subscriber in &inner.subscribers {
                        let mut sub = subscriber.borrow_mut();
                        sub(&new_value);
                    }
                }
            }
        });

        // Subscribe to other using weak references
        let result_weak2 = Rc::downgrade(&result_val.inner);
        let self_clone = self.clone();
        other.subscribe(tracker2.tracker(), move |other_val| {
            if let Some(result_inner) = result_weak2.upgrade() {
                let new_value = (self_clone.get(), other_val.clone());
                let mut inner = result_inner.borrow_mut();
                if inner.value != new_value {
                    inner.value = new_value.clone();
                    // Notify subscribers
                    for subscriber in &inner.subscribers {
                        let mut sub = subscriber.borrow_mut();
                        sub(&new_value);
                    }
                }
            }
        });
        let combined_tracker = Rc::new((tracker1, tracker2));
        result_val.inner.borrow_mut()._lifetime_tracker = Some(combined_tracker as Rc<dyn Any>);

        result_val
    }

    /// Combines this RxVal with an RxRef.
    /// Delegates to zip_val by converting the RxRef to RxVal.
    pub fn zip_ref<U>(&self, other: RxRef<U>) -> RxVal<(T, U)>
    where
        T: Clone + PartialEq,
        U: Clone + PartialEq + 'static,
    {
        self.zip_val(other.val())
    }
}

impl<T: 'static> RxVal<T>
where
    T: Clone,
{
    /// Creates a new RxVal with the given initial value.
    ///
    /// This is primarily used internally by RxRef. Users should typically
    /// create an RxRef and get the RxVal via `.val()`.
    pub(crate) fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(RxValInner {
                value,
                subscribers: Vec::new(),
                _lifetime_tracker: None,
            })),
        }
    }

    /// Updates the value and notifies all subscribers.
    ///
    /// This is an internal method used by RxRef.
    pub(crate) fn update(&self, value: T)
    where
        T: PartialEq,
    {
        // Clone subscribers list and value to avoid holding borrow during notification
        let (subscribers, new_value) = {
            let mut inner = self.inner.borrow_mut();

            // Only update and notify if the value actually changed
            if inner.value != value {
                inner.value = value.clone();
                (inner.subscribers.clone(), value)
            } else {
                return; // No change, no notification
            }
        };

        // Notify all subscribers without holding the borrow
        for subscriber in &subscribers {
            let mut sub = subscriber.borrow_mut();
            sub(&new_value);
        }
    }
}
