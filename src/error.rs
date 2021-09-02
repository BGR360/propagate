//! Defines a new error type.

use std::fmt;
use std::ops::Deref;
use std::panic;

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
    /// ```
    /// // begin file: foo.rs
    /// let loc = CodeLocation::here();
    /// assert_eq!(format!("{}", &loc), "foo.rs:1");
    /// ```
    #[inline]
    #[track_caller]
    pub fn here() -> Self {
        Self::from(panic::Location::caller())
    }

    /// Returns the `CodeLocation` that is `lines` lines below `self`, consuming `self`.
    ///
    /// # Example
    ///
    /// ```
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
#[derive(Debug, Eq, PartialEq)]
pub struct CodeLocationStack(pub Vec<CodeLocation>);

impl CodeLocationStack {
    pub fn to_strings(&self) -> Vec<String> {
        self.0.iter().map(|loc| format!("{}", loc)).collect()
    }
}

/*  ____  _             _    _____
 * / ___|| |_ __ _  ___| | _| ____|_ __ _ __ ___  _ __
 * \___ \| __/ _` |/ __| |/ /  _| | '__| '__/ _ \| '__|
 *  ___) | || (_| | (__|   <| |___| |  | | | (_) | |
 * |____/ \__\__,_|\___|_|\_\_____|_|  |_|  \___/|_|
 *  FIGLET: StackError
 */

/// A wrapper around a generic error type. Keeps track of a stack of code locations.
///
/// # Example
///
/// ```
/// use propagate::result::*;
/// use std::{fs, io};
///
/// fn file_size(path: &str) -> Result<u64, io::Error> {
///     let size = fs::File::open(path)?.metadata()?.len();
///     Ok(size)
/// }
///
/// let result = file_size("foo.txt");
/// match result {
///     Ok(size) => println!("File size: {} bytes", size),
///     Err(err) => {
///         println!("Call stack: {}", err.stack());
///         println!("I/O Error: {:?}", *err);
///     }
/// }
/// ```
#[derive(Debug)]
pub struct StackError<E> {
    pub(crate) error: E,
    pub(crate) stack: CodeLocationStack,
}

impl<E> StackError<E> {
    /// Constructs a new [`StackError`] from the given error.
    ///
    /// The stack will contain the source location of the caller of this function. If that
    /// function's caller is also annotated with `#[track_caller]`, then its location will be used
    /// instead, and so on up the stack to the first call within a non-tracked function.
    #[inline]
    #[track_caller]
    pub fn new(error: E) -> Self {
        let loc = CodeLocation::from(panic::Location::caller());
        Self {
            error,
            stack: CodeLocationStack(vec![loc]),
        }
    }

    /// Pushes the source location of the caller of this function onto the stack.
    ///
    /// If that function's caller is also annotated with `#[track_caller]`, then its location will
    /// be used instead, and so on up the stack to the first call within a non-tracked function.
    #[inline]
    #[track_caller]
    pub fn push_caller(&mut self) {
        let loc = CodeLocation::from(panic::Location::caller());
        self.stack.0.push(loc);
    }

    /// Returns the stack.
    pub fn stack(&self) -> &CodeLocationStack {
        &self.stack
    }

    /// Converts the wrapped error from type `E` to type `F`.
    pub(crate) fn convert_inner<F: From<E>>(self) -> StackError<F> {
        // N.B. I would implement this as `From<StackError<E>> for StackError<F>`,
        // but that conflicts with the blanket trait `From<T> for T` when `E` == `F`.
        StackError {
            error: From::from(self.error),
            stack: self.stack,
        }
    }
}

impl<E> Deref for StackError<E> {
    type Target = E;

    /// Returns a reference to the wrapped error.
    fn deref(&self) -> &Self::Target {
        &self.error
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
    fn stack_error_new_and_push_both_append_to_stack() {
        let mut fix = Fixture::default();

        fix.tag_location("new", CodeLocation::here().down_by(1));
        let mut stack_err = StackError::new("oops");

        fix.assert_error_has_stack(&stack_err, &["new"]);

        fix.tag_location("push", CodeLocation::here().down_by(1));
        stack_err.push_caller();

        fix.assert_error_has_stack(&stack_err, &["new", "push"]);
    }
}
