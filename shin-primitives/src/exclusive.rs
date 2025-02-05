//! A copy of [`std::sync::Exclusive`] on stable (tracked at https://github.com/rust-lang/rust/issues/98407)

use std::{
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

/// `Exclusive` provides only _mutable_ access, also referred to as _exclusive_
/// access to the underlying value. It provides no _immutable_, or _shared_
/// access to the underlying value.
///
/// While this may seem not very useful, it allows `Exclusive` to _unconditionally_
/// implement [`Sync`]. Indeed, the safety requirements of `Sync` state that for `Exclusive`
/// to be `Sync`, it must be sound to _share_ across threads, that is, it must be sound
/// for `&Exclusive` to cross thread boundaries. By design, a `&Exclusive` has no API
/// whatsoever, making it useless, thus harmless, thus memory safe.
///
/// Certain constructs like [`Future`]s can only be used with _exclusive_ access,
/// and are often `Send` but not `Sync`, so `Exclusive` can be used as hint to the
/// Rust compiler that something is `Sync` in practice.
///
/// ## Examples
/// Using a non-`Sync` future prevents the wrapping struct from being `Sync`
/// ```compile_fail
/// use core::cell::Cell;
///
/// async fn other() {}
/// fn assert_sync<T: Sync>(t: T) {}
/// struct State<F> {
///     future: F
/// }
///
/// assert_sync(State {
///     future: async {
///         let cell = Cell::new(1);
///         let cell_ref = &cell;
///         other().await;
///         let value = cell_ref.get();
///     }
/// });
/// ```
///
/// `Exclusive` ensures the struct is `Sync` without stripping the future of its
/// functionality.
/// ```
/// use core::cell::Cell;
/// use shin_primitives::exclusive::Exclusive;
///
/// async fn other() {}
/// fn assert_sync<T: Sync>(t: T) {}
/// struct State<F> {
///     future: Exclusive<F>
/// }
///
/// assert_sync(State {
///     future: Exclusive::new(async {
///         let cell = Cell::new(1);
///         let cell_ref = &cell;
///         other().await;
///         let value = cell_ref.get();
///     })
/// });
/// ```
///
/// ## Parallels with a mutex
/// In some sense, `Exclusive` can be thought of as a _compile-time_ version of
/// a mutex, as the borrow-checker guarantees that only one `&mut` can exist
/// for any value. This is a parallel with the fact that
/// `&` and `&mut` references together can be thought of as a _compile-time_
/// version of a read-write lock.

#[doc(alias = "SyncWrapper")]
#[doc(alias = "SyncCell")]
#[doc(alias = "Unique")]
// `Exclusive` can't have `PartialOrd`, `Clone`, etc. impls as they would
// use `&` access to the inner value, violating the `Sync` impl's safety
// requirements.
#[derive(Default)]
#[repr(transparent)]
pub struct Exclusive<T: ?Sized> {
    inner: T,
}

// See `Exclusive`'s docs for justification.
unsafe impl<T: ?Sized> Sync for Exclusive<T> {}

impl<T: ?Sized> fmt::Debug for Exclusive<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("Exclusive").finish_non_exhaustive()
    }
}

impl<T: Sized> Exclusive<T> {
    /// Wrap a value in an `Exclusive`
    #[must_use]
    #[inline]
    pub const fn new(t: T) -> Self {
        Self { inner: t }
    }

    /// Unwrap the value contained in the `Exclusive`
    #[must_use]
    #[inline]
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T: ?Sized> Exclusive<T> {
    /// Gets exclusive access to the underlying value.
    #[must_use]
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Gets pinned exclusive access to the underlying value.
    ///
    /// `Exclusive` is considered to _structurally pin_ the underlying
    /// value, which means _unpinned_ `Exclusive`s can produce _unpinned_
    /// access to the underlying value, but _pinned_ `Exclusive`s only
    /// produce _pinned_ access to the underlying value.
    #[must_use]
    #[inline]
    pub const fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
        // SAFETY: `Exclusive` can only produce `&mut T` if itself is unpinned
        // `Pin::map_unchecked_mut` is not const, so we do this conversion manually
        unsafe { Pin::new_unchecked(&mut self.get_unchecked_mut().inner) }
    }

    /// Build a _mutable_ reference to an `Exclusive<T>` from
    /// a _mutable_ reference to a `T`. This allows you to skip
    /// building an `Exclusive` with [`Exclusive::new`].
    #[must_use]
    #[inline]
    pub const fn from_mut(r: &'_ mut T) -> &'_ mut Exclusive<T> {
        // SAFETY: repr is ≥ C, so refs have the same layout; and `Exclusive` properties are `&mut`-agnostic
        unsafe { &mut *(r as *mut T as *mut Exclusive<T>) }
    }

    /// Build a _pinned mutable_ reference to an `Exclusive<T>` from
    /// a _pinned mutable_ reference to a `T`. This allows you to skip
    /// building an `Exclusive` with [`Exclusive::new`].
    #[must_use]
    #[inline]
    pub const fn from_pin_mut(r: Pin<&'_ mut T>) -> Pin<&'_ mut Exclusive<T>> {
        // SAFETY: `Exclusive` can only produce `&mut T` if itself is unpinned
        // `Pin::map_unchecked_mut` is not const, so we do this conversion manually
        unsafe { Pin::new_unchecked(Self::from_mut(r.get_unchecked_mut())) }
    }
}

impl<T> From<T> for Exclusive<T> {
    #[inline]
    fn from(t: T) -> Self {
        Self::new(t)
    }
}

impl<T> Future for Exclusive<T>
where
    T: Future + ?Sized,
{
    type Output = T::Output;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_pin_mut().poll(cx)
    }
}
