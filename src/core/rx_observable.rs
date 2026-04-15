use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use super::rx_ref::RxRef;
use super::rx_subject::RxSubject;
use super::rx_val::RxVal;
use super::tracker::Tracker;

type Subscriber<T> = Rc<RefCell<Box<dyn FnMut(&T)>>>;

/// Internal storage for an observable stream.
pub(super) struct RxObservableInner<T> {
    subscribers: Vec<Subscriber<T>>,
    // Optional tracker to keep subscriptions alive
    // Used by .stream() to maintain the source subscription
    pub(super) _lifetime_tracker: Option<Rc<dyn Any>>,
}

/// A read-only stream of events.
///
/// Unlike RxVal, RxObservable does NOT have a current value. Subscribers are
/// only called when new events are emitted, not immediately upon subscription.
///
/// This is useful for representing discrete events like button clicks, network
/// messages, or user actions that don't have a "current state".
///
/// # Example
/// ```
/// use rx_rs::core::{RxSubject, DisposableTracker};
///
/// let mut tracker = DisposableTracker::new();
/// let rx_subject = RxSubject::new();
/// let rx_observable = rx_subject.observable();
///
/// rx_observable.subscribe(tracker.tracker(), |value| {
///     println!("Event: {}", value);
/// }); // Nothing printed yet
///
/// rx_subject.next(42); // Prints "Event: 42"
/// rx_subject.next(100); // Prints "Event: 100"
/// ```
pub struct RxObservable<T> {
    pub(super) inner: Rc<RefCell<RxObservableInner<T>>>,
}

impl<T> Clone for RxObservable<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: 'static> RxObservable<T> {
    /// Subscribes to events.
    ///
    /// The subscriber function is called each time a new event is emitted.
    /// Unlike RxVal, it is NOT called immediately upon subscription.
    ///
    /// The subscription is automatically cleaned up when the tracker is dropped.
    ///
    /// # Arguments
    /// * `tracker` - Tracker that will manage this subscription's lifetime
    /// * `f` - Function called with a reference to the event on each emission
    pub fn subscribe<F>(&self, tracker: &Tracker, f: F)
    where
        F: FnMut(&T) + 'static,
    {
        // Wrap the subscriber in Rc<RefCell<>> for shared ownership
        let subscriber = Rc::new(RefCell::new(Box::new(f) as Box<dyn FnMut(&T)>));

        // Store for future events
        let subscriber_clone = subscriber.clone();
        let inner_clone = self.inner.clone();

        self.inner.borrow_mut().subscribers.push(subscriber_clone);

        // Add cleanup to tracker
        tracker.add(move || {
            // Remove subscriber when tracker drops
            inner_clone
                .borrow_mut()
                .subscribers
                .retain(|s| !Rc::ptr_eq(s, &subscriber));
        });
    }

    /// Returns the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.borrow().subscribers.len()
    }
}

impl<T: 'static> RxObservable<T> {
    /// Creates a new RxObservable.
    ///
    /// This is primarily used internally by RxSubject. Users should typically
    /// create an RxSubject and get the RxObservable via `.observable()`.
    pub(crate) fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(RxObservableInner {
                subscribers: Vec::new(),
                _lifetime_tracker: None,
            })),
        }
    }

    /// Emits an event to all subscribers.
    ///
    /// This is an internal method used by RxSubject.
    pub(crate) fn emit(&self, value: &T) {
        let inner = self.inner.borrow();

        // Notify all subscribers
        for subscriber in &inner.subscribers {
            let mut sub = subscriber.borrow_mut();
            sub(value);
        }
    }

    /// Converts this RxObservable to an RxVal with an initial value.
    ///
    /// The RxVal is updated whenever the observable emits a new value.
    /// A tracker must be provided to manage the subscription lifetime.
    ///
    /// # Arguments
    /// * `initial` - The initial value for the RxVal
    /// * `tracker` - Tracker to manage the subscription lifetime
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxSubject, DisposableTracker};
    ///
    /// let mut tracker = DisposableTracker::new();
    /// let subject = RxSubject::new();
    /// let val = subject.observable().to_val(0, tracker.tracker());
    ///
    /// assert_eq!(val.get(), 0);
    ///
    /// subject.next(42);
    /// assert_eq!(val.get(), 42);
    /// ```
    pub fn to_val(&self, initial: T, tracker: &Tracker) -> RxVal<T>
    where
        T: Clone + PartialEq,
    {
        // Create a new RxRef with the initial value
        let rx_ref = RxRef::new(initial);

        // Subscribe to this observable and update the ref
        let rx_ref_clone = rx_ref.clone();
        self.subscribe(tracker, move |value| {
            rx_ref_clone.set(value.clone());
        });

        // Return the val
        rx_ref.val()
    }

    /// Maps the values of this RxObservable using a transformation function.
    ///
    /// Returns a new RxObservable that emits transformed values.
    /// When the source observable emits, the transformation is applied and
    /// the resulting observable emits the transformed value.
    ///
    /// # Arguments
    /// * `f` - Function to transform values from A to B
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxSubject, DisposableTracker};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// let tracker = DisposableTracker::new();
    /// let subject = RxSubject::new();
    /// let doubled = subject.observable().map(|x| x * 2);
    ///
    /// let result = Rc::new(RefCell::new(None));
    /// let result_clone = result.clone();
    ///
    /// doubled.subscribe(tracker.tracker(), move |value| {
    ///     *result_clone.borrow_mut() = Some(*value);
    /// });
    ///
    /// subject.next(5);
    /// assert_eq!(*result.borrow(), Some(10));
    /// ```
    pub fn map<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> B + 'static,
    {
        use super::rx_subject::RxSubject;

        // Create a subject to forward transformed values to
        let subject = RxSubject::new();

        // Create a tracker that will live as long as the returned observable
        let tracker = Rc::new(Tracker::new());

        // Subscribe to this observable and forward transformed values to the subject
        let subject_clone = subject.clone();
        self.subscribe(&tracker, move |value| {
            subject_clone.next(f(value));
        });

        // Get the observable from the subject
        let observable = subject.observable();

        // Attach the tracker to keep subscription alive
        observable.inner.borrow_mut()._lifetime_tracker = Some(tracker as Rc<dyn Any>);

        observable
    }

    /// Flat-maps the values of this RxObservable using a function that returns RxVal<B>.
    ///
    /// When the observable emits, the function is called to get an RxVal<B>,
    /// and the resulting observable emits the current value of that RxVal.
    ///
    /// # Arguments
    /// * `f` - Function to transform values from A to RxVal<B>
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxSubject, RxRef, DisposableTracker};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// let tracker = DisposableTracker::new();
    /// let subject = RxSubject::new();
    /// let inner = RxRef::new(100);
    ///
    /// let inner_clone = inner.clone();
    /// let flattened = subject.observable().flat_map_val(move |_| inner_clone.val());
    ///
    /// let result = Rc::new(RefCell::new(None));
    /// let result_clone = result.clone();
    ///
    /// flattened.subscribe(tracker.tracker(), move |value| {
    ///     *result_clone.borrow_mut() = Some(*value);
    /// });
    ///
    /// subject.next(1);
    /// // Emits twice: once for current value, once for subscription
    /// assert_eq!(*result.borrow(), Some(100));
    /// ```
    pub fn flat_map_val<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxVal<B> + 'static,
    {
        use super::rx_subject::RxSubject;

        // Create a subject to forward values to
        let subject = RxSubject::new();

        // Create trackers
        let outer_tracker = Rc::new(Tracker::new());
        let inner_tracker = Rc::new(RefCell::new(Tracker::new()));

        // Subscribe to this observable
        let subject_clone = subject.clone();
        let inner_tracker_clone = inner_tracker.clone();
        let f_rc = Rc::new(f);

        self.subscribe(&outer_tracker, move |outer_value| {
            // Get new inner RxVal
            let new_inner = f_rc(outer_value);

            // Cancel previous inner subscription
            *inner_tracker_clone.borrow_mut() = Tracker::new();

            // Emit the current value of the new inner
            subject_clone.next(new_inner.get());

            // Subscribe to the new inner
            let subject_clone2 = subject_clone.clone();
            new_inner.subscribe(&inner_tracker_clone.borrow(), move |inner_value| {
                subject_clone2.next(inner_value.clone());
            });
        });

        // Get observable with trackers attached
        let observable = subject.observable();
        let combined_tracker = Rc::new((outer_tracker, inner_tracker));
        observable.inner.borrow_mut()._lifetime_tracker = Some(combined_tracker as Rc<dyn Any>);

        observable
    }

    /// Flat-maps using a function that returns RxRef<B>.
    /// Delegates to flat_map_val by converting the RxRef to RxVal.
    pub fn flat_map_ref<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxRef<B> + 'static,
    {
        self.flat_map_val(move |x| f(x).val())
    }

    /// Flat-maps using a function that returns RxObservable<B>.
    /// Switches to the new observable each time the source emits.
    pub fn flat_map_observable<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> RxObservable<B> + 'static,
    {
        use super::rx_subject::RxSubject;

        let subject = RxSubject::new();
        let outer_tracker = Rc::new(Tracker::new());
        let inner_tracker = Rc::new(RefCell::new(Tracker::new()));

        let subject_clone = subject.clone();
        let inner_tracker_clone = inner_tracker.clone();
        let f_rc = Rc::new(f);

        self.subscribe(&outer_tracker, move |outer_value| {
            let new_inner = f_rc(outer_value);
            *inner_tracker_clone.borrow_mut() = Tracker::new();

            let subject_clone2 = subject_clone.clone();
            new_inner.subscribe(&inner_tracker_clone.borrow(), move |inner_value| {
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
        B: Clone + 'static,
        F: Fn(&T) -> RxSubject<B> + 'static,
    {
        self.flat_map_observable(move |x| f(x).observable())
    }

    /// Joins this RxObservable with another RxObservable.
    ///
    /// The resulting observable emits whenever either source emits.
    /// Both observables must have the same type.
    ///
    /// # Arguments
    /// * `other` - Another RxObservable to join with
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxSubject, DisposableTracker};
    /// use std::cell::RefCell;
    /// use std::rc::Rc;
    ///
    /// let tracker = DisposableTracker::new();
    /// let subject1 = RxSubject::new();
    /// let subject2 = RxSubject::new();
    ///
    /// let joined = subject1.observable().join_observable(subject2.observable());
    ///
    /// let results = Rc::new(RefCell::new(Vec::new()));
    /// let results_clone = results.clone();
    ///
    /// joined.subscribe(tracker.tracker(), move |value| {
    ///     results_clone.borrow_mut().push(*value);
    /// });
    ///
    /// subject1.next(1);
    /// subject2.next(2);
    /// subject1.next(3);
    ///
    /// assert_eq!(*results.borrow(), vec![1, 2, 3]);
    /// ```
    pub fn join_observable(&self, other: RxObservable<T>) -> RxObservable<T>
    where
        T: Clone,
    {
        use super::rx_subject::RxSubject;

        // Create a subject to forward values to
        let subject = RxSubject::new();

        // Create trackers for both subscriptions
        let tracker1 = Rc::new(Tracker::new());
        let tracker2 = Rc::new(Tracker::new());

        // Subscribe to self
        let subject_clone1 = subject.clone();
        self.subscribe(&tracker1, move |value| {
            subject_clone1.next(value.clone());
        });

        // Subscribe to other
        let subject_clone2 = subject.clone();
        other.subscribe(&tracker2, move |value| {
            subject_clone2.next(value.clone());
        });

        // Get observable with both trackers attached
        let observable = subject.observable();
        let combined_tracker = Rc::new((tracker1, tracker2));
        observable.inner.borrow_mut()._lifetime_tracker = Some(combined_tracker as Rc<dyn Any>);

        observable
    }

    /// Joins this RxObservable with an RxSubject.
    /// Delegates to join_observable by converting the RxSubject to RxObservable.
    pub fn join_subject(&self, other: RxSubject<T>) -> RxObservable<T>
    where
        T: Clone,
    {
        self.join_observable(other.observable())
    }
}
