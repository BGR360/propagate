//! Defines a new error type.

use std::fmt;
use std::panic::{self, Location};

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
#[derive(Debug, Eq, PartialEq, Default)]
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

/*  _____                     ____  _             _
 * | ____|_ __ _ __ ___  _ __/ ___|| |_ __ _  ___| | __
 * |  _| | '__| '__/ _ \| '__\___ \| __/ _` |/ __| |/ /
 * | |___| |  | | | (_) | |   ___) | || (_| | (__|   <
 * |_____|_|  |_|  \___/|_|  |____/ \__\__,_|\___|_|\_\
 *  FIGLET: ErrorStack
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
pub struct ErrorStack<E, S = CodeLocationStack> {
    pub(crate) error: E,
    pub(crate) stack: S,
}

impl<E, S> ErrorStack<E, S> {
    /// Returns the wrapped error.
    pub fn error(&self) -> &E {
        &self.error
    }

    /// Returns the traced stack.
    pub fn stack(&self) -> &S {
        &self.stack
    }

    /// Converts the wrapped error from type `E` to type `F`.
    pub(crate) fn convert_inner<F: From<E>>(self) -> ErrorStack<F, S> {
        // N.B. I would implement this as `From<ErrorStack<E>> for ErrorStack<F>`,
        // but that conflicts with the blanket trait `From<T> for T` when `E` == `F`.
        ErrorStack {
            error: From::from(self.error),
            stack: self.stack,
        }
    }
}

impl<E, S: Default + Traced> ErrorStack<E, S> {
    /// Constructs a new [`ErrorStack`] from the given error.
    ///
    /// The stack will contain the source location of the caller of this function. If that
    /// function's caller is also annotated with `#[track_caller]`, then its location will be used
    /// instead, and so on up the stack to the first call within a non-tracked function.
    #[inline]
    #[track_caller]
    pub fn new(error: E) -> Self {
        let mut this = Self {
            error,
            stack: Default::default(),
        };
        this.stack.trace(Location::caller());
        this
    }
}

impl<E, S: Traced> ErrorStack<E, S> {
    /// Pushes the source location of the caller of this function onto the stack.
    ///
    /// If that function's caller is also annotated with `#[track_caller]`, then its location will
    /// be used instead, and so on up the stack to the first call within a non-tracked function.
    #[inline]
    #[track_caller]
    pub fn push_caller(&mut self) {
        self.stack.trace(Location::caller());
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
        let mut err_stack = ErrorStack::new("oops");

        fix.assert_error_has_stack(&err_stack, &["new"]);

        fix.tag_location("push", CodeLocation::here().down_by(1));
        err_stack.push_caller();

        fix.assert_error_has_stack(&err_stack, &["new", "push"]);
    }
}
