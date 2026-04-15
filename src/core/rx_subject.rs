use super::rx_observable::RxObservable;
use super::rx_val::RxVal;
use super::rx_ref::RxRef;
use super::tracker::Tracker;

/// A read-write stream of events.
///
/// RxSubject provides both read and write access to an event stream. It exposes
/// a read-only RxObservable via the `.observable()` method, and allows emitting
/// events via the `.next()` method.
///
/// Unlike RxRef, RxSubject does NOT hold a current value. It only emits discrete
/// events to subscribers.
///
/// # Example
/// ```
/// use rx_rs::core::{RxSubject, DisposableTracker};
///
/// let mut tracker = DisposableTracker::new();
/// let button_clicks = RxSubject::new();
///
/// // Subscribe to button click events
/// button_clicks.observable().subscribe(tracker.tracker(), |click_count| {
///     println!("Button clicked {} times", click_count);
/// }); // Nothing printed yet (no current value)
///
/// // Emit events
/// button_clicks.next(1); // Prints "Button clicked 1 times"
/// button_clicks.next(2); // Prints "Button clicked 2 times"
/// ```
#[derive(Clone)]
pub struct RxSubject<T> {
    inner: RxObservable<T>,
}

impl<T: 'static> RxSubject<T> {
    /// Creates a new RxSubject.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxSubject;
    ///
    /// let messages = RxSubject::<String>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            inner: RxObservable::new(),
        }
    }

    /// Emits a new event to all subscribers.
    ///
    /// All subscribers to the RxObservable obtained via `.observable()` will be
    /// called with the event.
    ///
    /// # Arguments
    /// * `value` - The event to emit
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxSubject;
    ///
    /// let events = RxSubject::new();
    /// events.next("click");
    /// events.next("hover");
    /// ```
    pub fn next(&self, value: T) {
        self.inner.emit(&value);
    }

    /// Returns a read-only view of this event stream.
    ///
    /// The returned RxObservable can be cloned and passed around, allowing multiple
    /// parts of the code to subscribe to events without having write access.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxSubject, DisposableTracker};
    ///
    /// let mut tracker = DisposableTracker::new();
    /// let events = RxSubject::new();
    /// let read_only = events.observable();
    ///
    /// read_only.subscribe(tracker.tracker(), |event| {
    ///     println!("Event: {}", event);
    /// });
    ///
    /// events.next(42);
    /// ```
    pub fn observable(&self) -> RxObservable<T> {
        self.inner.clone()
    }

    /// Returns the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.subscriber_count()
    }

    /// Converts this RxSubject to an RxVal with an initial value.
    ///
    /// The RxVal is updated whenever the subject emits a new value.
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
    /// let val = subject.toVal(0, tracker.tracker());
    ///
    /// assert_eq!(val.get(), 0);
    ///
    /// subject.next(42);
    /// assert_eq!(val.get(), 42);
    /// ```
    pub fn toVal(&self, initial: T, tracker: &Tracker) -> RxVal<T>
    where
        T: Clone + PartialEq,
    {
        self.inner.toVal(initial, tracker)
    }

    /// Maps the values of this RxSubject using a transformation function.
    ///
    /// Returns a new RxObservable that emits transformed values.
    /// When the source subject emits, the transformation is applied and
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
    /// let doubled = subject.map(|x| x * 2);
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
        self.inner.map(f)
    }

    /// Flat-maps the values of this RxSubject using a function that returns RxVal<B>.
    ///
    /// When the subject emits, the function is called to get an RxVal<B>,
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
    /// let flattened = subject.flatMapVal(move |_| inner_clone.val());
    ///
    /// let result = Rc::new(RefCell::new(None));
    /// let result_clone = result.clone();
    ///
    /// flattened.subscribe(tracker.tracker(), move |value| {
    ///     *result_clone.borrow_mut() = Some(*value);
    /// });
    ///
    /// subject.next(1);
    /// assert_eq!(*result.borrow(), Some(100));
    /// ```
    pub fn flatMapVal<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxVal<B> + 'static,
    {
        self.inner.flatMapVal(f)
    }

    /// Flat-maps using a function that returns RxRef<B>.
    pub fn flatMapRef<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxRef<B> + 'static,
    {
        self.inner.flatMapRef(f)
    }

    /// Flat-maps using a function that returns RxObservable<B>.
    pub fn flatMapObservable<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> RxObservable<B> + 'static,
    {
        self.inner.flatMapObservable(f)
    }

    /// Flat-maps using a function that returns RxSubject<B>.
    pub fn flatMapSubject<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> RxSubject<B> + 'static,
    {
        self.inner.flatMapSubject(f)
    }

    /// Joins this RxSubject with an RxObservable.
    pub fn joinObservable(&self, other: RxObservable<T>) -> RxObservable<T>
    where
        T: Clone,
    {
        self.inner.joinObservable(other)
    }

    /// Joins this RxSubject with another RxSubject.
    pub fn joinSubject(&self, other: RxSubject<T>) -> RxObservable<T>
    where
        T: Clone,
    {
        self.inner.joinSubject(other)
    }
}

impl<T: 'static> Default for RxSubject<T> {
    fn default() -> Self {
        Self::new()
    }
}
