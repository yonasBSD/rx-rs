use super::rx_observable::RxObservable;
use super::rx_subject::RxSubject;
use super::rx_val::RxVal;

/// A read-write reactive value that holds a current state.
///
/// RxRef provides both read and write access to a reactive value. It exposes
/// a read-only RxVal via the `.val()` method, and allows updating the value
/// via the `.set()` method.
///
/// When the value is set, all subscribers to the RxVal are notified.
///
/// # Example
/// ```
/// use rx_rs::core::{RxRef, DisposableTracker};
///
/// let mut tracker = DisposableTracker::new();
/// let client_id = RxRef::new(None);
///
/// // Subscribe to changes
/// client_id.val().subscribe(tracker.tracker(), |maybe_id| {
///     match maybe_id {
///         Some(id) => println!("Client ID assigned: {}", id),
///         None => println!("No client ID yet"),
///     }
/// }); // Prints "No client ID yet" immediately
///
/// // Update the value
/// client_id.set(Some(42)); // Prints "Client ID assigned: 42"
/// ```
#[derive(Clone)]
pub struct RxRef<T> {
    inner: RxVal<T>,
}

impl<T: 'static> RxRef<T>
where
    T: Clone + PartialEq,
{
    /// Creates a new RxRef with the given initial value.
    ///
    /// # Arguments
    /// * `value` - The initial value
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let connection_state = RxRef::new("Disconnected");
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            inner: RxVal::new(value),
        }
    }

    /// Sets a new value and notifies all subscribers.
    ///
    /// All subscribers to the RxVal obtained via `.val()` will be called
    /// with the new value.
    ///
    /// # Arguments
    /// * `value` - The new value to set
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let state = RxRef::new(0);
    /// state.set(42);
    /// assert_eq!(state.get(), 42);
    /// ```
    pub fn set(&self, value: T) {
        self.inner.update(value);
    }

    /// Gets the current value.
    ///
    /// This is a convenience method that delegates to the underlying RxVal.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let counter = RxRef::new(5);
    /// assert_eq!(counter.get(), 5);
    /// ```
    pub fn get(&self) -> T {
        self.inner.get()
    }

    /// Returns a read-only view of this reactive value.
    ///
    /// The returned RxVal can be cloned and passed around, allowing multiple
    /// parts of the code to subscribe to changes without having write access.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxRef, DisposableTracker};
    ///
    /// let mut tracker = DisposableTracker::new();
    /// let state = RxRef::new("initial");
    /// let read_only = state.val();
    ///
    /// read_only.subscribe(tracker.tracker(), |val| {
    ///     println!("State: {}", val);
    /// });
    /// ```
    pub fn val(&self) -> RxVal<T> {
        self.inner.clone()
    }

    /// Modifies the value using a closure and notifies subscribers.
    ///
    /// This is useful when you need to update the value based on its current
    /// state, without having to clone it first.
    ///
    /// # Arguments
    /// * `f` - Function that receives a mutable reference to the current value
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::RxRef;
    ///
    /// let counter = RxRef::new(0);
    /// counter.modify(|count| *count += 1);
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn modify<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut current = self.get();
        f(&mut current);
        self.set(current);
    }

    /// Returns the number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.inner.subscriber_count()
    }

    /// Converts this RxRef into a stream (RxObservable).
    ///
    /// The returned observable emits the current value immediately on subscription,
    /// and then emits all future changes.
    ///
    /// This is a convenience method that delegates to the underlying RxVal's `.stream()`.
    ///
    /// # Example
    /// ```
    /// use rx_rs::core::{RxRef, DisposableTracker};
    ///
    /// let mut tracker = DisposableTracker::new();
    /// let state = RxRef::new(0);
    /// let stream = state.stream();
    ///
    /// stream.subscribe(tracker.tracker(), |value| {
    ///     println!("State: {}", value);
    /// }); // Prints "State: 0" immediately
    ///
    /// state.set(42); // Prints "State: 42"
    /// ```
    pub fn stream(&self) -> RxObservable<T> {
        self.inner.stream()
    }

    /// Maps the values of this RxRef using a transformation function.
    ///
    /// Returns a new RxVal that always contains the transformed value.
    /// When the source RxRef changes, the transformation is applied and
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
    /// let doubled = number.map(|x| x * 2);
    ///
    /// assert_eq!(doubled.get(), 10);
    ///
    /// number.set(10);
    /// assert_eq!(doubled.get(), 20);
    /// ```
    pub fn map<B, F>(&self, f: F) -> RxVal<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> B + 'static,
    {
        self.inner.map(f)
    }

    /// Flat-maps the values of this RxRef using a function that returns RxVal<B>.
    ///
    /// When the source RxRef changes, the function is called to produce a new RxVal<B>,
    /// and the result RxVal is updated to reflect the current value of that inner RxVal.
    /// The result also updates when the inner RxVal changes.
    ///
    /// This is a convenience method that delegates to the underlying RxVal's `.flat_map()`.
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
    /// let flattened = outer.flat_map(move |&x| {
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
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxVal<B> + 'static,
    {
        self.inner.flat_map(f)
    }

    /// Flat-maps using a function that returns RxRef<B>.
    pub fn flat_map_ref<B, F>(&self, f: F) -> RxVal<B>
    where
        B: Clone + PartialEq + 'static,
        F: Fn(&T) -> RxRef<B> + 'static,
    {
        self.inner.flat_map_ref(f)
    }

    /// Flat-maps using a function that returns RxObservable<B>.
    pub fn flat_map_observable<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> RxObservable<B> + 'static,
    {
        self.inner.flat_map_observable(f)
    }

    /// Flat-maps using a function that returns RxSubject<B>.
    pub fn flat_map_subject<B, F>(&self, f: F) -> RxObservable<B>
    where
        B: Clone + 'static,
        F: Fn(&T) -> RxSubject<B> + 'static,
    {
        self.inner.flat_map_subject(f)
    }

    /// Combines this RxRef with another RxVal.
    pub fn zip_val<U>(&self, other: RxVal<U>) -> RxVal<(T, U)>
    where
        U: Clone + PartialEq + 'static,
    {
        self.inner.zip_val(other)
    }

    /// Combines this RxRef with another RxRef.
    pub fn zip_ref<U>(&self, other: RxRef<U>) -> RxVal<(T, U)>
    where
        U: Clone + PartialEq + 'static,
    {
        self.inner.zip_ref(other)
    }
}
