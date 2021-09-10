//! Defines a new error type.

use std::fmt;
use std::panic;

use crate::result::Traced;

/*   ____          _      _                    _   _
 *  / ___|___   __| | ___| |    ___   ___ __ _| |_(_) ___  _ __
 * | |   / _ \ / _` |/ _ \ |   / _ \ / __/ _` | __| |/ _ \| '_ \
 * | |__| (_) | (_| |  __/ |__| (_) | (_| (_| | |_| | (_) | | | |
 *  \____\___/ \__,_|\___|_____\___/ \___\__,_|\__|_|\___/|_| |_|
 *  FIGLET: CodeLocation
 */

/// Represents a location (filename, line number) in the source code.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CodeLocation {
    file: &'static str,
    line: u32,
}

impl CodeLocation {
    pub fn new(file: &'static str, line: u32) -> Self {
        Self { file, line }
    }

    /// Returns the code location at the site of the caller.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use propagate::error::*;
    /// // begin file: foo.rs
    /// let loc = CodeLocation::here();
    /// assert_eq!(format!("{}", &loc), "foo.rs:1");
    /// ```
    #[inline]
    #[track_caller]
    pub fn here() -> Self {
        Self::from(panic::Location::caller())
    }

    /// Returns the `CodeLocation` that is `lines` lines below `self`,
    /// consuming `self`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use propagate::error::*;
    /// // begin file: foo.rs
    /// let loc = CodeLocation::here().down_by(1);
    /// assert_eq!(format!("{}", &loc), "foo.rs:2");
    /// ```
    pub fn down_by(self, lines: u32) -> Self {
        Self {
            file: self.file,
            line: self.line + lines,
        }
    }
}

impl From<&'static panic::Location<'static>> for CodeLocation {
    fn from(loc: &'static panic::Location<'static>) -> Self {
        CodeLocation {
            file: loc.file(),
            line: loc.line(),
        }
    }
}

impl fmt::Display for CodeLocation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.file, self.line)
    }
}

/*   ____          _      _                    _   _             ____  _             _
 *  / ___|___   __| | ___| |    ___   ___ __ _| |_(_) ___  _ __ / ___|| |_ __ _  ___| | __
 * | |   / _ \ / _` |/ _ \ |   / _ \ / __/ _` | __| |/ _ \| '_ \\___ \| __/ _` |/ __| |/ /
 * | |__| (_) | (_| |  __/ |__| (_) | (_| (_| | |_| | (_) | | | |___) | || (_| | (__|   <
 *  \____\___/ \__,_|\___|_____\___/ \___\__,_|\__|_|\___/|_| |_|____/ \__\__,_|\___|_|\_\
 *  FIGLET: CodeLocationStack
 */

/// A stack of code locations.
#[derive(PartialEq, Eq, Default, Debug)]
pub struct CodeLocationStack(pub Vec<CodeLocation>);

impl Traced for CodeLocationStack {
    fn trace(&mut self, location: &'static panic::Location) {
        self.0.push(location.into());
    }
}

impl CodeLocationStack {
    pub fn to_strings(&self) -> Vec<String> {
        self.0.iter().map(|loc| format!("{}", loc)).collect()
    }
}

impl fmt::Display for CodeLocationStack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, location) in self.0.iter().enumerate() {
            write!(f, "\n   {}: {}", index, location)?;
        }

        Ok(())
    }
}

/*
  _____                       _ _____
 |_   _| __ __ _  ___ ___  __| | ____|_ __ _ __ ___  _ __
   | || '__/ _` |/ __/ _ \/ _` |  _| | '__| '__/ _ \| '__|
   | || | | (_| | (_|  __/ (_| | |___| |  | | | (_) | |
   |_||_|  \__,_|\___\___|\__,_|_____|_|  |_|  \___/|_|

    FIGLET: TracedError
*/

/// A wrapper around a generic error type that stores an error trace.
///
/// # Custom Trace Type
///
/// The trace type `S` can be customized if you would like to customize how code
/// locations are stored or processed.
///
/// # Examples
///
/// Typically you would not work with `TracedError` manually, but the following
/// example illustrates the tracing behavior of `TracedError`:
///
/// ```
/// # use propagate::error::*;
///
/// fn foo() -> TracedError<&'static str> {
///     // Create new error with foo() at the start of the error trace.
///     TracedError::new("Nothing here")
/// }
///
/// fn bar() -> TracedError<&'static str> {
///     let mut error = foo();
///     // Add bar() to the error trace.
///     error.push_caller();
///     error
/// }
///
/// let traced_error = bar();
/// let error: &str = traced_error.error();
/// let stack: &CodeLocationStack = traced_error.stack();
/// assert_eq!(error, "Nothing here");
/// assert_eq!(stack.0.len(), 2);
/// ```
///
/// And here's an example of using a custom trace type:
///
#[derive(PartialEq, Eq, Debug, Hash)]
pub struct TracedError<E, S = CodeLocationStack> {
    pub(crate) error: E,
    pub(crate) stack: S,
}

impl<E, S> TracedError<E, S> {
    /// Returns the wrapped error.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Returns the traced stack.
    pub fn stack(&self) -> &S {
        &self.stack
    }

    /// Converts the wrapped error from type `E` to type `F`.
    ///
    /// The error trace is not modified.
    pub(crate) fn convert_inner<F: From<E>>(self) -> TracedError<F, S> {
        // N.B. I would implement this as `From<TracedError<E>> for
        // TracedError<F>`, but that conflicts with the blanket trait
        // `From<T> for T` when `E` == `F`.
        TracedError {
            error: From::from(self.error),
            stack: self.stack,
        }
    }
}

impl<E, S: Default + Traced> TracedError<E, S> {
    /// Constructs a new [`TracedError`] from the given error.
    ///
    /// The stack will contain the source location of the caller of this
    /// function. If that function's caller is also annotated with
    /// `#[track_caller]`, then its location will be used instead, and so on up
    /// the stack to the first call within a non-tracked function.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::error::*;
    /// let e: TracedError<&str> = TracedError::new("Nothing here");
    /// assert_eq!(e.stack().0.len(), 1);
    /// ```
    #[inline]
    #[track_caller]
    pub fn new(error: E) -> Self {
        let mut this = Self {
            error,
            stack: Default::default(),
        };
        this.stack.trace(panic::Location::caller());
        this
    }
}

impl<E, S: Traced> TracedError<E, S> {
    /// Pushes the source location of the caller of this function onto the
    /// stack.
    ///
    /// If that function's caller is also annotated with `#[track_caller]`, then
    /// its location will be used instead, and so on up the stack to the first
    /// call within a non-tracked function.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```
    /// # use propagate::error::*;
    /// let loc0 = CodeLocation::here().down_by(1);
    /// let mut e: TracedError<&str> = TracedError::new("Nothing here");
    ///
    /// let loc1 = CodeLocation::here().down_by(1);
    /// e.push_caller();
    ///
    /// let loc2 = CodeLocation::here().down_by(1);
    /// e.push_caller();
    ///
    /// assert_eq!(e.stack().0, vec![loc0, loc1, loc2]);
    /// ```
    #[inline]
    #[track_caller]
    pub fn push_caller(&mut self) {
        self.stack.trace(panic::Location::caller());
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

    #[test]
    fn error_stack_new_and_push_both_append_to_stack() {
        let mut fix = Fixture::default();

        fix.tag_location("new", CodeLocation::here().down_by(1));
        let mut err_stack = TracedError::new("oops");

        fix.assert_error_has_stack(&err_stack, &["new"]);

        fix.tag_location("push", CodeLocation::here().down_by(1));
        err_stack.push_caller();

        fix.assert_error_has_stack(&err_stack, &["new", "push"]);
    }
}
