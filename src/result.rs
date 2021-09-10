//! Defines a new result type.

use crate::error::TracedError;
use crate::CodeLocationStack;

use std::convert::Infallible;
use std::fmt;
use std::ops::{ControlFlow, FromResidual, Try};
use std::panic::Location;
use std::process::Termination;

/// A trait denoting "stack-like" types that can be used with [`Result<T, E, S>`].
pub trait Traced {
    fn trace(&mut self, location: &'static Location);
}

/// Construct a new [`propagate::Result`][crate::Result] with the given success value.
#[inline]
pub fn ok<T, E, S>(ok_value: T) -> Result<T, E, S> {
    self::Result(Ok(ok_value))
}

/// Construct a new [`propagate::Result`][crate::Result] with the given error value.
#[inline]
#[track_caller]
pub fn err<T, E, S: Traced + Default, F: From<E>>(err_value: E) -> Result<T, F, S> {
    let wrapped = TracedError::new(F::from(err_value));
    self::Result(Err(wrapped))
}

/*  ____                 _ _    _______   _______
 * |  _ \ ___  ___ _   _| | |_ / /_   _| | ____\ \
 * | |_) / _ \/ __| | | | | __/ /  | |   |  _|  \ \
 * |  _ <  __/\__ \ |_| | | |_\ \  | |_  | |___ / /
 * |_| \_\___||___/\__,_|_|\__|\_\ |_( ) |_____/_/
 *                                   |/
 *  FIGLET: Result<T, E>
 */

/// A wrapper around [`std::result::Result<T, E>`] that supports chaining via the `?` operator.
///
/// TODO: document custom stack types.
///
/// # Propagation Using `?`
///
/// After a [`propagate::Result`] has been constructed, it will keep a running
/// "stack trace" of the code locations where the `?` operator is invoked on it.
///
/// ## Coercion Using `From`
///
/// Any `propagate::Result<T, E>` can be coerced to a `propagate::Result<T, F>`
/// using the `?` operator if there is a [`From<E>`] defined for type `F`:
///
/// ```
/// use propagate::Result;
/// fn f() -> Result<(), String> {
///     let result: Result<(), &str> = Result::new_err("str slice");
///     propagate::Ok(result?)
/// }
/// ```
///
/// ## Coercion from `std::result::Result`
///
/// To provide easy interoperability with standard library modules and other
/// crates that return results, any [`std::result::Result`] can be coerced to a
/// `propagate::Result` using the `?` operator:
///
/// ```
/// use propagate::Result;
/// use std::fs::File;
/// fn f() -> Result<File, std::io::Error> {
///     let result: std::result::Result<File, std::io::Error> =
///         File::open("foo.txt");
///     propagate::Ok(result?)
/// }
/// ```
///
/// You can also coerce `std::result::Result<T, E>` to `propagate::Result<T, F>`
/// if there is a [`From<E>`] defined for type `F`.
///
/// ```
/// use propagate::Result;
/// fn f() -> Result<(), String> {
///     let result: std::result::Result<(), &str> = Err("string slice");
///     propagate::Ok(result?)
/// }
/// ```
///
///
/// # Working with `propagate::Result`
///
/// There are a few caveats when using [`propagate::Result`] as a replacement
/// for the std library result.
///
/// ## Contained Value
///
/// `propagate::Result` is defined as such;
///
/// ```
/// # use propagate::error::TracedError;
/// enum Result<T, E> {
///     Ok(T),
///     Err(TracedError<E>),
/// }
/// ```
///
/// [`TracedError`] is a wrapper around an arbitrary error value, and it stores
/// a stack trace alongside the wrapped error value.
///
/// Thus, when a `propagate::Result` is equal to `Err(e)`, the value `e` is not
/// of type `E`, but rather it is of type `TracedError<E>`.
///
/// Because of this, if you want to pattern match a `Result<T, E>` and get a
/// value of `E`, you must call [`error()`][crate::TracedError::error()] on the
/// the `Err(e)` value first:
///
/// ```
/// # fn function_that_returns_result() -> propagate::Result<(), String> {
/// #     propagate::Result::new_err("a")
/// # }
/// let result: propagate::Result<(), String> = function_that_returns_result();
/// match result {
///     propagate::Ok(_) => {}
///     propagate::Err(e) => {
///         println!("stack: {}", e.stack());
///         let inner: &String = e.error();
///         println!("inner: {}", inner);
///     }
/// }
/// ```
///
/// ## Creating Errors
///
/// Because `Result<T, E>` is technically a `Result<T, TracedError<E>>`, you
/// cannot construct a new error result by simply doing `Err(error_value)`.
///
/// You can turn your error value into a `TracedError` in one of the
/// following ways:
///
/// ```
/// use propagate::{Result, TracedError};
///
/// // Directly
/// let result: Result<(), i32> = propagate::Err(TracedError::new(42));
///
/// // Using Result::new_err()
/// let result: Result<(), i32> = Result::new_err(42);
/// ```
///
/// ## **IMPORTANT**: Forwarding Errors
///
/// When not using `try` blocks, you must remember to surround result values
/// with `Ok(..?)` when returning them in a function. The compiler will not
/// force you to do this if the result value's type is identical to the
/// function's return type.
///
/// ```
/// use propagate::Result;
///
/// fn gives_error() -> Result<(), &'static str> {
///     Result::new_err("Nothing here")
/// }
///
/// // YES: Result surrounded by Ok(..?), so the stack trace will include foo()
/// fn foo() -> Result<(), &'static str> {
///     let result = gives_error();
///     propagate::Ok(result?)
/// }
///
/// // NO: Result returned directly, so the stack trace will not include bar()
/// fn bar() -> Result<(), &'static str> {
///     let result = gives_error();
///     result
/// }
/// ```
///
/// When you do use `try` blocks, you do not need the `Ok`, and the compiler
/// will force you to use `?`:
///
/// ```
/// #![feature(try_blocks)]
/// # use propagate::Result;
/// # fn gives_error() -> Result<(), &'static str> {
/// #     Result::new_err("Nothing here")
/// # }
/// // YES
/// fn foo() -> Result<(), &'static str> {
///     try {
///         let result = gives_error();
///         result?
///     }
/// }
/// ```
///
/// ```compile_fail
/// #![feature(try_blocks)]
/// # use propagate::Result;
/// # fn gives_error() -> Result<(), &'static str> {
/// #     Result::new_err("Nothing here")
/// # }
/// // NO: will not compile
/// fn bar() -> Result<(), &'static str> {
///     try {
///         let result = gives_error();
///         result
///     }
/// }
/// // NO: will not compile
/// fn baz() -> Result<(), &'static str> {
///     try {
///         let result = gives_error();
///         propagate::Ok(result?)
///     }
/// }
/// ```
///
/// [`propagate::Result`]: crate::Result
#[must_use = "this `Result` may be an `Err` variant, which should be handled"]
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct Result<T, E, S = CodeLocationStack>(std::result::Result<T, TracedError<E, S>>);

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
        Self(Ok(output))
    }

    #[inline]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        match self.0 {
            Ok(ok) => ControlFlow::Continue(ok),
            Err(err) => ControlFlow::Break(Result(Err(err))),
        }
    }
}

/// Pushes an entry to the stack when one [`Result`] is coerced to another using the `?` operator.
impl<T, E, S, F> FromResidual<Result<Infallible, E, S>> for Result<T, F, S>
where
    S: Traced,
    F: From<E>,
{
    #[inline]
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E, S>) -> Self {
        match residual.0 {
            Ok(_) => unreachable!(),
            Err(mut e) => {
                e.push_caller();
                Self(Err(e.convert_inner()))
            }
        }
    }
}

/// Starts a new stack when a [`std::result::Result`] is coerced to a [`Result`] using `?`.
impl<T, E, S, F> FromResidual<std::result::Result<Infallible, E>> for Result<T, F, S>
where
    S: Traced + Default,
    F: From<E>,
{
    #[inline]
    #[track_caller]
    fn from_residual(residual: std::result::Result<Infallible, E>) -> Self {
        match residual {
            Ok(_) => unreachable!(),
            Err(e) => {
                let wrapped = TracedError::new(F::from(e));
                Self(Err(wrapped))
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
        match self.0 {
            Ok(_) => 0,
            Err(err) => {
                println!(
                    "Error: {}",
                    trial_and_error::Report::new(err.error()).pretty(true)
                );

                println!("\nReturn Trace: {}", err.stack());

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

impl<T, E, S> Result<T, E, S> {
    /// Converts from `Result<T, E>` to [`std::result::Result<T, E>`]
    /// and returns the traced stack if it is `Err`.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::result::Result;
    /// let x: Result<(), &str> = propagate::err("Nothing here");
    /// let (result, stack) = x.unpack();
    /// assert!(matches!(result, Err("Nothing here")));
    /// assert_eq!(stack.unwrap().0.len(), 1);
    /// ```
    pub fn unpack(self) -> (std::result::Result<T, E>, Option<S>) {
        match self.0 {
            Ok(ok) => (Ok(ok), None),
            Err(err) => {
                let inner = err.error;
                let stack = err.stack;
                (Err(inner), Some(stack))
            }
        }
    }
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
        fn func() -> Result<u32, String> {
            let x: Result<u32, String> = err("string slice");
            x
        }
        let (result, _stack) = func().unpack();
        assert_eq!(result, Err(String::from("string slice")));
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
        ok(())
    }

    #[test]
    fn test_success() {
        let mut fix = Fixture::default();

        let result = maybe_io_error(&mut fix, false);
        assert!(matches!(result.0, Ok(())));
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
            ok(maybe_io_error(&mut fix, true)?)
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
            ok(())
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error", "bottom"]);
    }

    #[test]
    fn propagate_coerces_to_custom_error_type() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            ok(maybe_io_error(&mut fix, true)?)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["io_error", "bottom"]);
    }

    #[test]
    fn new_err_coerces_to_custom_error_type_from_inner() {
        let mut fix = Fixture::default();

        let mut bottom = || -> Result<(), MyError> {
            fix.tag_location("bottom", CodeLocation::here().down_by(1));
            err("oops".to_string())
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
            err(my_error)
        };

        let result = bottom();
        fix.assert_result_has_stack(result, &["bottom"]);
    }
}
