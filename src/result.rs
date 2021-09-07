//! Defines a new result type.

use crate::error::ErrorStack;
use crate::CodeLocationStack;

use std::convert::Infallible;
use std::fmt;
use std::ops::{ControlFlow, FromResidual, Try};
use std::panic::Location;
use std::process::Termination;

pub use self::Result::Err;
pub use self::Result::Ok;

/// A trait denoting "stack-like" types that can be used with [`Result<T, E, S>`].
pub trait Traced {
    fn trace(&mut self, location: &'static Location);
}

/*  ____                 _ _    _______   _______
 * |  _ \ ___  ___ _   _| | |_ / /_   _| | ____\ \
 * | |_) / _ \/ __| | | | | __/ /  | |   |  _|  \ \
 * |  _ <  __/\__ \ |_| | | |_\ \  | |_  | |___ / /
 * |_| \_\___||___/\__,_|_|\__|\_\ |_( ) |_____/_/
 *                                   |/
 *  FIGLET: Result<T, E>
 */

/// A replacement for [`std::result::Result<T, E>`] that supports chaining via the `?` operator.
///
/// See [`propagate`][crate] for more information.
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
#[derive(Debug)]
pub enum Result<T, E, S: Traced = CodeLocationStack> {
    /// Contains the success value.
    Ok(T),
    /// Contains the error value wrapped in a [`ErrorStack`].
    Err(ErrorStack<E, S>),
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
/// Invoking the `?` operator invokes [`Self::branch()`] under the hood. This function returns a
/// [`ControlFlow`] enum which dictates whether the execution will continue forward (i.e., `Ok()`),
/// or break early (i.e., `Err()`). The value produced when continuing is the `Output`, and the
/// value produced when breaking early is called the `Residual`.
///
/// Coercion between residual types is achieved by implementing the [`FromResidual`] trait.
/// `Result` allows coercion from standard library results ([`std::result::Result`]) as well as
/// from other `Result` instances whose inner error types are convertible from one to another.
impl<T, E> Try for Result<T, E> {
    type Output = T;
    type Residual = Result<Infallible, E>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self {
            Ok(ok) => ControlFlow::Continue(ok),
            Err(err) => ControlFlow::Break(Err(err)),
        }
    }
}

/// Pushes an entry to the stack when one [`Result`] is coerced to another using the `?` operator.
impl<T, E, S: Traced, F: From<E>> FromResidual<Result<Infallible, E, S>> for Result<T, F, S> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E, S>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(mut e) => {
                e.stack.trace(Location::caller());
                Err(e.convert_inner())
            }
        }
    }
}

/// Starts a new stack when a [`std::result::Result`] is coerced to a [`Result`] using `?`.
impl<T, E, F: From<E>> FromResidual<std::result::Result<Infallible, E>> for Result<T, F> {
    #[inline]
    #[track_caller]
    fn from_residual(residual: std::result::Result<Infallible, E>) -> Self {
        match residual {
            std::result::Result::Ok(_) => unreachable!(),
            std::result::Result::Err(e) => Err(ErrorStack::new(From::from(e))),
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

impl<T, E: std::error::Error> Termination for Result<T, E> {
    fn report(self) -> i32 {
        match self {
            Ok(_) => 0,
            Err(err) => {
                println!(
                    "Error: {}",
                    trial_and_error::Report::new(&*err).pretty(true)
                );

                let stack = err.stack();
                if !stack.0.is_empty() {
                    println!("\nReturn Trace: {}", err.stack());
                }

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
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// ```
    #[inline]
    #[track_caller]
    pub fn new_err<D>(error_value: D) -> Self
    where
        E: From<D>,
    {
        Err(ErrorStack::new(E::from(error_value)))
    }
}

impl<T, E, S: Traced> Result<T, E, S> {
    /// Converts from `Result<T, E>` to [`std::result::Result<T, E>`].
    ///
    /// Converts `self` into a [`std::result::Result<T, E>`], consuming `self`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.to_std(), std::result::Result::Ok(2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.to_std(), std::result::Result::Err("Nothing here"));
    /// ```
    #[inline]
    pub fn to_std(self) -> std::result::Result<T, E> {
        match self {
            Ok(t) => std::result::Result::Ok(t),
            Err(e) => std::result::Result::Err(e.error),
        }
    }

    #[inline]
    pub fn as_std_ref(&self) -> std::result::Result<&T, &E> {
        match self {
            Ok(ref t) => std::result::Result::Ok(t),
            Err(ref e) => std::result::Result::Err(&e.error),
        }
    }

    /// Converts from `Result<T, E>` to [`Option<ErrorStack<E>>`].
    ///
    /// Converts `self` into an [`Option<ErrorStack<E>>`], consuming `self`,
    /// and discarding the success value, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.err_stack(), None);
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// match x.err_stack() {
    ///     Some(e) => assert_eq!(*e, "Nothing here"),
    ///     None => unreachable!(),
    /// }
    /// ```
    #[inline]
    pub fn err_stack(self) -> Option<ErrorStack<E, S>> {
        match self {
            Ok(_) => None,
            Err(x) => Some(x),
        }
    }
}

impl<T, E> Result<T, E> {
    /////////////////////////////////////////////////////////////////////////
    // Querying the contained values
    /////////////////////////////////////////////////////////////////////////

    /// Returns `true` if the result is [`Ok`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<i32, &str> = Ok(-3);
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

    /// Returns `true` if the result is [`Err`].
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<i32, &str> = Ok(-3);
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

    /// Converts from `Result<T, E>` to [`Option<T>`].
    ///
    /// Converts `self` into an [`Option<T>`], consuming `self`,
    /// and discarding the error, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.ok(), Some(2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.ok(), None);
    /// ```
    #[inline]
    pub fn ok(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(_) => None,
        }
    }

    /// Converts from `Result<T, E>` to [`Option<E>`].
    ///
    /// Converts `self` into an [`Option<E>`], consuming `self`,
    /// and discarding the success value, if any.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.err(), None);
    ///
    /// let x: Result<u32, &str> = Result::new_err("Nothing here");
    /// assert_eq!(x.err(), Some("Nothing here"));
    /// ```
    #[inline]
    pub fn err(self) -> Option<E> {
        match self {
            Ok(_) => None,
            Err(x) => Some(x.error),
        }
    }

    /////////////////////////////////////////////////////////////////////////
    // Adapter for working with references
    /////////////////////////////////////////////////////////////////////////

    // TODO: how to do this? I think the returned result should have a `&T` or a `&ErrorStack<E>`,
    // but idk how to make that happen.
    /*
    /// Converts from `&Result<T, E>` to `Result<&T, &E>`.
    ///
    /// Produces a new `Result`, containing a reference
    /// into the original, leaving the original in place.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.as_ref(), Ok(&2));
    ///
    /// let x: Result<u32, &str> = Result::new_err("Error");
    /// assert_eq!(x.as_ref(), Err(&"Error"));
    /// ```
    #[inline]
    pub const fn as_ref(&self) -> Result<&T, &E> {
        match *self {
            Ok(ref x) => Ok(x),
            Err(ref x) => Err(x),
        }
    }
    */

    // TODO: how to do this? I think the returned result should have a `&mut T` or a
    // `&mut ErrorStack<E>`, but idk how to make that happen.
    /*
    /// Converts from `&mut Result<T, E>` to `Result<&mut T, &mut E>`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// fn mutate(r: &mut Result<i32, i32>) {
    ///     match r.as_mut() {
    ///         Ok(v) => *v = 42,
    ///         Err(e) => *e = 0,
    ///     }
    /// }
    ///
    /// let mut x: Result<i32, i32> = Ok(2);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap(), 42);
    ///
    /// let mut x: Result<i32, i32> = Result::new_err(13);
    /// mutate(&mut x);
    /// assert_eq!(x.unwrap_err(), 0);
    /// ```
    #[inline]
    pub fn as_mut(&mut self) -> Result<&mut T, &mut E> {
        match *self {
            Ok(ref mut x) => Ok(x),
            Err(ref mut x) => Err(x),
        }
    }
    */

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
    /// fn stringify(x: u32) -> String { format!("error code: {}", x) }
    ///
    /// let x: Result<u32, u32> = Ok(2);
    /// assert_eq!(x.map_err(stringify), Ok(2));
    ///
    /// let x: Result<u32, u32> = Result::new_err(13);
    /// assert_eq!(x.map_err(stringify), Err("error code: 13".to_string()));
    /// ```
    #[inline]
    pub fn map_err<F, O: FnOnce(E) -> F>(self, op: O) -> Result<T, F> {
        // XXX: should this push_caller? I think probably not, as users will just use
        // `?` with whatever comes out of this.
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(ErrorStack {
                error: op(e.error),
                stack: e.stack,
            }),
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
    /// let default = 2;
    /// let x: Result<u32, &str> = Ok(9);
    /// assert_eq!(x.unwrap_or(default), 9);
    ///
    /// let x: Result<u32, &str> = Result::new_err("error");
    /// assert_eq!(x.unwrap_or(default), default);
    /// ```
    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        match self {
            Ok(t) => t,
            Err(_) => default,
        }
    }

    /// Returns the contained [`Ok`] value or computes it from a closure.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// fn count(x: &str) -> usize { x.len() }
    ///
    /// assert_eq!(Ok(2).unwrap_or_else(count), 2);
    /// assert_eq!(Err("foo").unwrap_or_else(count), 3);
    /// ```
    #[inline]
    pub fn unwrap_or_else<F: FnOnce(E) -> T>(self, op: F) -> T {
        match self {
            Ok(t) => t,
            Err(e) => op(e.error),
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
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// x.expect("Testing expect"); // panics with `Testing expect: emergency failure`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed(msg, &e),
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
    /// let x: Result<u32, &str> = Ok(2);
    /// assert_eq!(x.unwrap(), 2);
    /// ```
    ///
    /// ```should_panic
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// x.unwrap(); // panics with `emergency failure`
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap(self) -> T {
        match self {
            Ok(t) => t,
            Err(e) => unwrap_failed("called `Result::unwrap()` on an `Err` value", &e),
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
    /// let x: Result<u32, &str> = Ok(10);
    /// x.expect_err("Testing expect_err"); // panics with `Testing expect_err: 10`
    /// ```
    #[inline]
    #[track_caller]
    pub fn expect_err(self, msg: &str) -> E {
        match self {
            Ok(t) => unwrap_failed(msg, &t),
            Err(e) => e.error,
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
    /// let x: Result<u32, &str> = Ok(2);
    /// x.unwrap_err(); // panics with `2`
    /// ```
    ///
    /// ```
    /// let x: Result<u32, &str> = Result::new_err("emergency failure");
    /// assert_eq!(x.unwrap_err(), "emergency failure");
    /// ```
    #[inline]
    #[track_caller]
    pub fn unwrap_err(self) -> E {
        match self {
            Ok(t) => unwrap_failed("called `Result::unwrap_err()` on an `Ok` value", &t),
            Err(e) => e.error,
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
    /// [`FromStr`]: crate::str::FromStr
    #[inline]
    pub fn unwrap_or_default(self) -> T {
        match self {
            Ok(x) => x,
            Err(_) => Default::default(),
        }
    }
}

impl<T, E> Result<Option<T>, E> {
    /// Transposes a `Result` of an `Option` into an `Option` of a `Result`.
    ///
    /// `Ok(None)` will be mapped to `None`.
    /// `Ok(Some(_))` and `Err(_)` will be mapped to `Some(Ok(_))` and `Some(Err(_))`.
    ///
    /// # Examples
    ///
    /// ```
    /// #[derive(Debug, Eq, PartialEq)]
    /// struct SomeErr;
    ///
    /// let x: Result<Option<i32>, SomeErr> = Ok(Some(5));
    /// let y: Option<Result<i32, SomeErr>> = Some(Ok(5));
    /// assert_eq!(x.transpose(), y);
    /// ```
    #[inline]
    pub fn transpose(self) -> Option<Result<T, E>> {
        match self {
            Ok(Some(x)) => Some(Ok(x)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

// This is a separate function to reduce the code size of the methods
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
    use super::*;
    use crate::test::Fixture;
    use crate::CodeLocation;
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
