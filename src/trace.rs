//! Defines types for error tracing.

use std::fmt;
use std::panic;

/// A trait denoting "stack-like" types that can be used with [`Result<T, E, S>`].
pub trait Traced {
    fn trace(&mut self, location: &'static panic::Location);
}

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
    /// # use propagate::trace::*;
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
    /// # use propagate::trace::*;
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
    /// Constructs a new code location stack with the caller at the top.
    #[inline]
    #[track_caller]
    pub fn new() -> Self {
        let caller = CodeLocation::from(panic::Location::caller());
        Self(vec![caller])
    }

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
