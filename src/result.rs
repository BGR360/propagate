//! Defines a new result type.

use crate::trace::{ErrorTrace, Traced};

use std::convert::Infallible;
use std::fmt;
use std::ops::{ControlFlow, FromResidual, Try};
use std::panic;
use std::process::Termination;

pub use self::Result::Err;
pub use self::Result::Ok;

/*  ____                 _ _    _______   _______
 * |  _ \ ___  ___ _   _| | |_ / /_   _| | ____\ \
 * | |_) / _ \/ __| | | | | __/ /  | |   |  _|  \ \
 * |  _ <  __/\__ \ |_| | | |_\ \  | |_  | |___ / /
 * |_| \_\___||___/\__,_|_|\__|\_\ |_( ) |_____/_/
 *                                   |/
 *  FIGLET: Result<T, E>
 */

/// A replacement for [`std::result::Result`] that supports tracing via the `?`
/// operator.
///
/// After a [`propagate::Result`] has been constructed, it will keep a running
/// "return trace" of the code locations where the `?` operator is invoked on it.
///
/// # Working with `propagate::Result`
///
/// There are a few things to remember when using [`propagate::Result`] as a
/// replacement for the std library result.
///
/// ## Creating Errors
///
/// Construct an [`Err`] variant by providing an error value and a new trace
/// (e.g., using [`ErrorTrace::new()`]):
///
/// ```
/// use propagate::ErrorTrace;
///
/// fn gives_error() -> propagate::Result<(), &'static str> {
///     propagate::Err("Nothing here", ErrorTrace::new())
/// }
/// ```
///
/// ## Pattern Matching
///
/// The [`Err`] variant contains two values: the error and its associated trace.
/// Pattern matching should look like this:
///
/// ```
/// # use propagate::ErrorTrace;
/// # fn gives_error() -> propagate::Result<(), &'static str> {
/// #     propagate::Err("Nothing here", ErrorTrace::new())
/// # }
/// let result: propagate::Result<_, _> = gives_error();
/// match result {
///     propagate::Ok(_) => {}
///     propagate::Err(err, trace) => {
///         println!("error: {}", err);
///         println!("trace: {}", trace);
///     }
/// }
/// ```
///
/// ## **IMPORTANT**: Forwarding Errors
///
/// When not using [`try` blocks], you must remember to surround result values
/// with `Ok(..?)` when returning them in a function. The compiler will not
/// force you to do this if the result value's type is identical to the
/// function's return type.
///
/// ```
/// # use propagate::ErrorTrace;
/// # fn gives_error() -> propagate::Result<(), &'static str> {
/// #     propagate::Err("Nothing here", ErrorTrace::new())
/// # }
/// // YES: Result surrounded by Ok(..?), so the error trace will include foo()
/// fn foo() -> propagate::Result<(), &'static str> {
///     let result = gives_error();
///     propagate::Ok(result?)
/// }
///
/// // NO: Result returned directly, so the error trace will not include bar()
/// fn bar() -> propagate::Result<(), &'static str> {
///     let result = gives_error();
///     result
/// }
/// ```
///
/// When you use [`try` blocks], you do not need the `Ok`:
///
/// ```
/// #![feature(try_blocks)]
/// # use propagate::ErrorTrace;
/// # fn gives_error() -> propagate::Result<(), &'static str> {
/// #     propagate::Err("Nothing here", ErrorTrace::new())
/// # }
/// fn foo() -> propagate::Result<(), &'static str> {
///     try {
///         let result = gives_error();
///         result?
///     }
/// }
/// ```
///
/// And the compiler will force you to use `?`:
///
/// ```compile_fail
/// #![feature(try_blocks)]
/// # use propagate::ErrorTrace;
/// # fn gives_error() -> propagate::Result<(), &'static str> {
/// #     propagate::Err("Nothing here", ErrorTrace::new())
/// # }
/// fn bar() -> propagate::Result<(), &'static str> {
///     try {
///         let result = gives_error();
///         result
///     }
/// }
/// ```
///
///
/// # Coercion Using `?`
///
/// The `?` operator can be used to coerce between result types.
///
/// ## From `propagate` to `propagate`
///
/// One `propagate::Result` can be coerced to another if there is an
/// implementation of [`From`] between the two error types:
///
/// ```
/// #![feature(try_blocks)]
/// # use propagate::ErrorTrace;
///
/// let result: propagate::Result<(), &str> =
///     propagate::Err("str slice", ErrorTrace::new());
///
/// // Coerces string slice to String object
/// let result: propagate::Result<(), String> = try { result? };
/// ```
///
/// ## From `std` to `propagate`
///
/// To provide easy interoperability with standard library modules and other
/// crates that return results, any [`std::result::Result`] can be coerced to a
/// `propagate::Result`:
///
/// ```
/// #![feature(try_blocks)]
///
/// let result: Result<(), u32> = Err(42);
///
/// // Coerces std::result::Result to propagate::Result
/// let result: propagate::Result<(), u32> = try { result? };
/// ```
///
/// And if there is an implementation of [`From`] between the two error types,
/// you can do that coercion at the same time:
///
/// ```
/// #![feature(try_blocks)]
///
/// let result: Result<(), &str> = Err("string slice");
///
/// // Coerces std::result::Result to propagate::Result
/// // AND
/// // Coerces string slice to String object
/// let result: propagate::Result<(), String> = try { result? };
/// ```
///
/// [`propagate::Result`]: crate::Result
/// [`try` blocks]: https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
#[derive(PartialEq, Eq, Debug, Hash)]
pub enum Result<T, E, S = ErrorTrace> {
    /// Contains the success value.
    Ok(T),
    /// Contains the error value and associated error trace.
    Err(E, S),
}

/*  _                 _   _____
 * (_)_ __ ___  _ __ | | |_   _| __ _   _
 * | | '_ ` _ \| '_ \| |   | || '__| | | |
 * | | | | | | | |_) | |   | || |  | |_| |
 * |_|_| |_| |_| .__/|_|   |_||_|   \__, |
 *             |_|                  |___/
 *  FIGLET: impl Try
 */

/// Overriding the try operator `?` for [`Result`].
///
/// Invoking the `?` operator invokes [`Self::branch()`] under the hood. This
/// function returns a [`ControlFlow`] enum which dictates whether the execution
/// will continue forward (i.e., `Ok()`), or break early (i.e., `Err()`). The
/// value produced when continuing is the `Output`, and the value produced when
/// breaking early is called the `Residual`.
///
/// Coercion between residual types is achieved by implementing the
/// [`FromResidual`] trait. `Result` allows coercion from standard library
/// results ([`std::result::Result`]) as well as from other `Result` instances
/// whose inner error types are convertible from one to another.
impl<T, E, S: Traced> Try for Result<T, E, S> {
    type Output = T;
    type Residual = Result<Infallible, E, S>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(ok) => ControlFlow::Continue(ok),
            Err(err, trace) => ControlFlow::Break(Err(err, trace)),
        }
    }
}

/// Pushes an entry to the trace when one [`Result`] is coerced to another using the `?` operator.
impl<T, E, S, F> FromResidual<Result<Infallible, E, S>> for Result<T, F, S>
where
    S: Traced,
    F: From<E>,
{
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E, S>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(err, mut trace) => {
                trace.trace(panic::Location::caller());
                Err(From::from(err), trace)
            }
        }
    }
}

/// Starts a new trace when a [`std::result::Result`] is coerced to a [`Result`] using `?`.
impl<T, E, S, F> FromResidual<std::result::Result<Infallible, E>> for Result<T, F, S>
where
    S: Traced + Default,
    F: From<E>,
{
    #[inline]
    #[track_caller]
    fn from_residual(residual: std::result::Result<Infallible, E>) -> Self {
        match residual {
            std::result::Result::Ok(_) => unreachable!(),
            std::result::Result::Err(err) => {
                let mut trace = S::default();
                trace.trace(panic::Location::caller());
                Err(From::from(err), trace)
            }
        }
    }
}

/*
  _                 _   _____                   _             _   _
 (_)_ __ ___  _ __ | | |_   _|__ _ __ _ __ ___ (_)_ __   __ _| |_(_) ___  _ __
 | | '_ ` _ \| '_ \| |   | |/ _ \ '__| '_ ` _ \| | '_ \ / _` | __| |/ _ \| '_ \
 | | | | | | | |_) | |   | |  __/ |  | | | | | | | | | | (_| | |_| | (_) | | | |
 |_|_| |_| |_| .__/|_|   |_|\___|_|  |_| |_| |_|_|_| |_|\__,_|\__|_|\___/|_| |_|
             |_|
 FIGLET: impl Termination
*/

impl<T, E: std::error::Error, S: fmt::Display> Termination for Result<T, E, S> {
    fn report(self) -> i32 {
        match self {
            Ok(_) => 0,
            Err(err, trace) => {
                println!("Error: {}", trial_and_error::Report::new(err).pretty(true));

                println!("\nReturn Trace: {}", trace);

                1
            }
        }
    }
}

/*  _                 _   ____                 _ _
 * (_)_ __ ___  _ __ | | |  _ \ ___  ___ _   _| | |_
 * | | '_ ` _ \| '_ \| | | |_) / _ \/ __| | | | | __|
 * | | | | | | | |_) | | |  _ <  __/\__ \ |_| | | |_
 * |_|_| |_| |_| .__/|_| |_| \_\___||___/\__,_|_|\__|
 *             |_|
 *  FIGLET: impl Result
 */

/// Stuff not from the standard library.
impl<T, E, S: Traced + Default> Result<T, E, S> {
    /// Constructs a new error result from the provided error value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// ```
    #[inline]
    #[track_caller]
    pub fn new_err<D>(error_value: D) -> Self
    where
        E: From<D>,
    {
        let mut trace = S::default();
        trace.trace(panic::Location::caller());
        Err(E::from(error_value), trace)
    }
}

impl<T, E, S> Result<T, E, S> {
    /// Converts from `Result<T, E, S>` to [`std::result::Result<T, E>`].
    ///
    /// Converts `self` into a [`std::result::Result<T, E>`], consuming `self`,
    /// and discarding the error trace, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert_eq!(x.to_std(), std::result::Result::Ok(2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.to_std(), std::result::Result::Err("Nothing here"));
    /// ```
    #[inline]
    pub fn to_std(self) -> std::result::Result<T, E> {
        match self {
            Ok(t) => std::result::Result::Ok(t),
            Err(err, _) => std::result::Result::Err(err),
        }
    }

    /// Converts from `Result<T, E, S>` to [`Option<(E, S)>`][Option].
    ///
    /// Converts `self` into an [`Option<(E, S)>`][Option], consuming `self`,
    /// and discarding the success value, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert!(x.err_trace().is_none());
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// match x.err_trace() {
    ///     Some((err, trace)) => {
    ///         assert_eq!(err, "Nothing here");
    ///         assert_eq!(trace.0.len(), 1);
    ///     }
    ///     None => unreachable!(),
    /// }
    /// ```
    #[inline]
    pub fn err_trace(self) -> Option<(E, S)> {
        match self {
            Ok(_) => None,
            Err(err, trace) => Some((err, trace)),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is `Ok`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<i32, &str> = propagate::Ok(-3);
    /// assert_eq!(x.is_ok(), true);
    ///
    /// let x: Result<i32, &str> = Result::new_err("Some error message");
    /// assert_eq!(x.is_ok(), false);
    /// ```
    #[must_use = "if you intended to assert that this is ok, consider `.unwrap()` instead"]
    #[inline]
    pub const fn is_ok(&self) -> bool {
        matches!(*self, Ok(_))
    }

    /// Returns `true` if the result is `Err`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<i32, &str> = propagate::Ok(-3);
    /// assert_eq!(x.is_err(), false);
    ///
    /// let x: Result<i32, &str> = Result::new_err("Some error message");
    /// assert_eq!(x.is_err(), true);
    /// ```
    #[must_use = "if you intended to assert that this is err, consider `.unwrap_err()` instead"]
    #[inline]
    pub const fn is_err(&self) -> bool {
        !self.is_ok()
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for each variant
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `Result<T, E, S>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_, _) => None,
        }
    }

    /// Converts from `Result<T, E, S>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value and error trace, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert_eq!(x.err(), None);
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.err(), Some("Nothing here"));
    /// ```
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            Ok(_) => None,
            Err(err, _) => Some(err),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////

    /// Converts from `&Result<T, E, S>` to `Result<&T, &E, &S>`.
    ///
    /// Produces a new `Result`, containing a reference
    /// into the original, leaving the original in place.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert_eq!(x.as_ref(), propagate::Ok(&2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Error");
    /// assert!(matches!(x.as_ref(), propagate::Err(&"Error", _)));
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Result<&T, &E, &S> {
        match *self {
            Ok(ref t) => Ok(t),
            Err(ref err, ref trace) => Err(err, trace),
        }
    }

    /// Converts from `&mut Result<T, E, S>` to `Result<&mut T, &mut E, &mut S>`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// fn mutate(r: &mut Result<i32, i32>) {
    ///     match r.as_mut() {
    ///         propagate::Ok(v) => *v = 42,
    ///         propagate::Err(e, _) => *e = 0,
    ///     }
    /// }
    ///
    /// let mut x: Result<i32, i32> = propagate::Ok(2);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap(), 42);
    ///
    /// let mut x: Result<i32, i32> = Result::new_err(13);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap_err(), 0);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> Result<&mut T, &mut E, &mut S> {
        match *self {
            Ok(ref mut t) => Ok(t),
            Err(ref mut err, ref mut trace) => Err(err, trace),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Transforming contained values
    /////////////////////////////////////////////////////////////////////////

    // TODO: map
    // TODO: map_or
    // TODO: map_or_else

    /// Maps a `Result<T, E>` to `Result<T, F>` by applying a function to a
    /// contained [`Err`] value, leaving an [`Ok`] value untouched.
    ///
    /// This function can be used to pass through a successful result while handling
    /// an error.
    ///
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// fn stringify(x: i32) -> String { format!("error code: {}", x) }
    ///
    /// let x: Result<i32, i32> = propagate::Ok(2);
    /// assert_eq!(x.map_err(stringify), propagate::Ok(2));
    ///
    /// let x: Result<i32, i32> = Result::new_err(13);
    /// let y: Result<i32, String> = x.map_err(stringify);
    /// assert_eq!(y.err().unwrap(), "error code: 13".to_string());
    /// ```
    #[inline]
    pub fn map_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, F, S> {
        // XXX: should this push_caller? I think probably not, as users will just use
        // `?` with whatever comes out of this.
        match self {
            Ok(t) => Ok(t),
            Err(err, trace) => Err(op(err), trace),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Boolean operations on the values, eager and lazy
    /////////////////////////////////////////////////////////////////////////

    /// Returns the contained [`Ok`] value or a provided default.
    ///
    /// Arguments passed to `unwrap_or` are eagerly evaluated; if you are passing
    /// the result of a function call, it is recommended to use [`unwrap_or_else`],
    /// which is lazily evaluated.
    ///
    /// [`unwrap_or_else`]: Result::unwrap_or_else
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let default = 2;
    /// let x: Result<u32, &str> = propagate::Ok(9);
    /// assert_eq!(x.unwrap_or(default), 9);
    ///
    /// let x: Result<u32, &str> = Result::new_err("error");
    /// assert_eq!(x.unwrap_or(default), default);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(t) => t,
            Err(_, _) => default,
        }
    }

    /// Returns the contained [`Ok`] value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::{ErrorTrace, Result};
    /// fn count(x: &str) -> usize { x.len() }
    ///
    /// let x: Result<usize, &str> = propagate::Ok(2);
    /// assert_eq!(x.unwrap_or_else(count), 2);
    ///
    /// let x: Result<usize, &str> = propagate::Err("foo", ErrorTrace::new());
    /// assert_eq!(x.unwrap_or_else(count), 3);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, op: F) -> T {
        match self {
            Ok(t) => t,
            Err(err, _) => op(err),
        }
    }
}

impl<T, E: fmt::Debug> Result<T, E> {
    /// Returns the contained [`Ok`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Err`], with a panic message including the
    /// passed message, and the content of the [`Err`].
    ///
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```should_panic
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// x.expect("Testing expect"); // panics with `Testing expect: emergency failure`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(err, _) => unwrap_failed(msg, &err),
        }
    }

    /// Returns the contained [`Ok`] value, consuming the `self` value.
    ///
    /// Because this function may panic, its use is generally discouraged.
    /// Instead, prefer to use pattern matching and handle the [`Err`]
    /// case explicitly, or call [`unwrap_or`], [`unwrap_or_else`], or
    /// [`unwrap_or_default`].
    ///
    /// [`unwrap_or`]: Result::unwrap_or
    /// [`unwrap_or_else`]: Result::unwrap_or_else
    /// [`unwrap_or_default`]: Result::unwrap_or_default
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Err`], with a panic message provided by the
    /// [`Err`]'s value.
    ///
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// assert_eq!(x.unwrap(), 2);
    /// ```
    ///
    /// ```should_panic
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// x.unwrap(); // panics with `emergency failure`
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(err, _) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &err),
        }
    }
}

impl<T: fmt::Debug, E> Result<T, E> {
    /// Returns the contained [`Err`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Ok`], with a panic message including the
    /// passed message, and the content of the [`Ok`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```should_panic
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(10);
    /// x.expect_err("Testing expect_err"); // panics with `Testing expect_err: 10`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E {
        match self {
            Ok(t) => unwrap_failed(msg, &t),
            Err(err, _) => err,
        }
    }

    /// Returns the contained [`Err`] value, consuming the `self` value.
    ///
    /// # Panics
    ///
    /// Panics if the value is an [`Ok`], with a custom panic message provided
    /// by the [`Ok`]'s value.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = propagate::Ok(2);
    /// x.unwrap_err(); // panics with `2`
    /// ```
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// assert_eq!(x.unwrap_err(), "emergency failure");
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap_err(self) -> E {
        match self {
            Ok(t) => unwrap_failed("called `Result::unwrap_err()` on an `Ok` value", &t),
            Err(err, _) => err,
        }
    }
}

impl<T: Default, E> Result<T, E> {
    /// Returns the contained [`Ok`] value or a default
    ///
    /// Consumes the `self` argument then, if [`Ok`], returns the contained
    /// value, otherwise if [`Err`], returns the default value for that
    /// type.
    ///
    /// # Examples
    ///
    /// Converts a string to an integer, turning poorly-formed strings
    /// into 0 (the default value for integers). [`parse`] converts
    /// a string to any other type that implements [`FromStr`], returning an
    /// [`Err`] on error.
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let good_year_from_input = "1909";
    /// let bad_year_from_input = "190blarg";
    /// let good_year = good_year_from_input.parse().unwrap_or_default();
    /// let bad_year = bad_year_from_input.parse().unwrap_or_default();
    ///
    /// assert_eq!(1909, good_year);
    /// assert_eq!(0, bad_year);
    /// ```
    ///
    /// [`parse`]: str::parse
    /// [`FromStr`]: std::str::FromStr
    #[inline]
    pub fn unwrap_or_default(self) -> T {
        match self {
            Ok(x) => x,
            Err(_, _) => Default::default(),
        }
    }
}

impl<T, E, S> Result<Option<T>, E, S> {
    /// Transposes a `Result` of an `Option` into an `Option` of a `Result`.
    ///
    /// `Ok(None)` will be mapped to `None`.
    /// `Ok(Some(_))` and `Err(_, _)` will be mapped to `Some(Ok(_))` and
    /// `Some(Err(_, _))`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use propagate::result::Result;
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Result<Option<i32>, SomeErr> = propagate::Ok(Some(5));
    /// let y: Option<Result<i32, SomeErr>> = Some(propagate::Ok(5));
    /// assert_eq!(x.transpose(), y);
    /// ```
    #[inline]
    pub fn transpose(self) -> Option<Result<T, E, S>> {
        match self {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(err, trace) => Some(Err(err, trace)),
        }
    }
}

// This is a separate function to reduce the code size of the methods
// TODO: Include the error trace in the panic message.
#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{}: {:?}", msg, error)
}

/*  _            _
 * | |_ ___  ___| |_
 * | __/ _ \/ __| __|
 * | ||  __/\__ \ |_
 *  \__\___||___/\__|
 *  FIGLET: test
 */

#[cfg(test)]
mod test {
    use crate::test::Fixture;
    use crate::CodeLocation;
    use crate::{Ok, Result};
    use std::fs;
    use std::io;

    /*  ____            _         __                  _   _
     * | __ )  __ _ ___(_) ___   / _|_   _ _ __   ___| |_(_) ___  _ __  ___
     * |  _ \ / _` / __| |/ __| | |_| | | | '_ \ / __| __| |/ _ \| '_ \/ __|
     * | |_) | (_| \__ \ | (__  |  _| |_| | | | | (__| |_| | (_) | | | \__ \
     * |____/ \__,_|___/_|\___| |_|  \__,_|_| |_|\___|\__|_|\___/|_| |_|___/
     *  FIGLET: Basic functions
     */

    #[test]
    fn new_err_coerce() {
        fn inner() -> Result<u32, String> {
            let x: Result<u32, String> = Result::new_err("string slice");
            x
        }
        assert_eq!(inner().err().unwrap(), String::from("string slice"));
    }

    #[test]
    fn can_convert_to_std_result() {
        let x: Result<u32, &str> = Ok(2);
        assert_eq!(x.to_std(), std::result::Result::Ok(2));

        let x: Result<u32, &str> = Result::new_err("Nothing here");
        assert_eq!(x.to_std(), std::result::Result::Err("Nothing here"));
    }

    /*   ____ _           _       _
     *  / ___| |__   __ _(_)_ __ (_)_ __   __ _
     * | |   | '_ \ / _` | | '_ \| | '_ \ / _` |
     * | |___| | | | (_| | | | | | | | | | (_| |
     *  \____|_| |_|\__,_|_|_| |_|_|_| |_|\__, |
     *                                    |___/
     *  FIGLET: Chaining
     */

    fn maybe_io_error(fix: &mut Fixture, fail: bool) -> Result<(), io::Error> {
        fix.tag_location("io_error", CodeLocation::here().down_by(2));
        if fail {
            let _ = fs::File::open("/nonexistent/file")?;
        }
        Ok(())
    }

    #[test]
    fn test_success() {
        let mut fix = Fixture::default();

        let result = maybe_io_error(&mut fix, false);
        assert!(matches!(result, Ok(())));
    }

    #[test]
    fn question_mark_operator_coerces_from_std_result() {
        let mut fix = Fixture::default();

        let result = maybe_io_error(&mut fix, true);
        fix.assert_result_has_stack(result, &["io_error"])
    }

    #[test]
    fn return_with_propagate_appends_to_stack() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), io::Error> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            Ok(maybe_io_error(&mut fix, true)?)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error", "bottom"]);
    }

    #[test]
    fn return_without_propagate_does_not_append_to_stack() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), io::Error> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            maybe_io_error(&mut fix, true)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error"]);
    }

    #[derive(Debug)]
    enum MyError {
        Io(io::Error),
        Other(String),
    }

    impl From<io::Error> for MyError {
        fn from(e: io::Error) -> Self {
            Self::Io(e)
        }
    }

    impl From<String> for MyError {
        fn from(s: String) -> Self {
            Self::Other(s)
        }
    }

    #[test]
    fn question_mark_operator_coerces_to_custom_error_type() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            maybe_io_error(&mut fix, true)?;
            Ok(())
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error", "bottom"]);
    }

    #[test]
    fn propagate_coerces_to_custom_error_type() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            Ok(maybe_io_error(&mut fix, true)?)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error", "bottom"]);
    }

    #[test]
    fn new_err_coerces_to_custom_error_type_from_inner() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            Result::new_err("oops".to_string())
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["bottom"]);
    }

    #[test]
    fn new_err_coerces_to_result_from_custom_error_type() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            let my_error = MyError::Other("oops".to_string());
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            Result::new_err(my_error)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["bottom"]);
    }
}
