//! Error propagation tracing in Rust.
//!
//! This crate provides [`propagate::Result<T, E>`][crate::Result], a
//! replacement for the standard library result type that automatically tracks
//! the propagation of error results using the `?` operator.
//!
//!
//! # Propagation Tracing vs. Backtracing
//!
//! Being able to trace the cause of an error is critical for many types of
//! software written in Rust. For easy diagnosis, errors should provide some
//! sort of **trace** denoting source code locations that contributed to the
//! error.
//!
//! Crates such as [`anyhow`][anyhow] provide easy access to backtraces when
//! creating errors. The `propagate` crate provides **propagation tracing**:
//! every time the `?` operator is applied to an error result, the code location
//! of that `?` invocation is appended to a running "stack trace" stored in the
//! result.
//!
//! Propagation tracing differs from runtime backtracing in a few important
//! ways. You should evaluate which approach is appropriate for your
//! application.
//!
//! [anyhow]: https://docs.rs/anyhow/latest/anyhow/
//!
//!
//! ## Advantages of Propagation Tracing
//!
//! **Multithreaded tracing**
//!
//! A backtrace provides a single point-in-time capture of a call stack on a
//! single thread. In complex software, error results may pass between multiple
//! threads on their way up to their final consumers.
//!
//! Propagation tracing provides a true view into the path that an error takes
//! through your code, even if it passes between multiple threads.
//!
//! **Low performance overhead**
//!
//! Runtime backtracing requires unwinding stacks and mapping addresses to
//! source code locations symbols at runtime. With `propagate`, the information
//! for each code location is compiled statically into your application's
//! binary, and the stack trace is built up in real time as the error propagates
//! from function to function.
//!
//!
//! ## Disadvantages of Propagation Tracing
//!
//! **Code size**
//!
//! `propagate` stores code locations of `?` invocations in your application or
//! library's binary.
//!
//! **Boilerplate**
//!
//! `propagate` results require a bit more attention to work with than std
//! library results. Namely, you must use [`Result::new_err`] to construct new
//! errors and must remember to use `Ok(..?)` when forwarding errors to other
//! functions.
//!
//!
//! # Example
//!
//! ```
//! // Overrides the std library's `Result` type.
//! use propagate::prelude::*;
//!
//! use std::io;
//! use std::fs::File;
//!
//! enum MyError {
//!     Unlucky,
//!     Io(io::Error),
//!     TooSmall(u64),
//! }
//!
//! impl From<io::Error> for MyError {
//!     fn from(e: io::Error) -> Self {
//!         Self::Io(e)
//!     }
//! }
//!
//! fn file_size(path: &str) -> Result<u64, MyError> {
//!     // The `?` operator coerces `std::result::Result<_, io::Error>` into `Result<_, MyError>`.
//!     let size = File::open(path)?.metadata()?.len();
//!
//!     if size >= 1024 {
//!         Ok(size)
//!     } else {
//!         // Use `Result::new_err()` to start a new error chain.
//!         Result::new_err(MyError::TooSmall(size))
//!     }
//! }
//!
//! fn maybe_file_size(path: &str) -> Result<u64, MyError> {
//!     let lucky = (path.len() % 2) == 0;
//!     if !lucky {
//!         Result::new_err(MyError::Unlucky)
//!     } else {
//!         // Always use `Ok(..?)` when directly returning a `Result<T, E>`.
//!         Ok(file_size(path)?)
//!     }
//! }
//!
//! fn main() {
//!     let result = maybe_file_size("foo.txt");
//!     match result {
//!         Ok(size) => println!("File size: {} KiB", size / 1024),
//!         Err(err) => {
//!             // Dereference `err` to get the actual error value.
//!             match *err {
//!                 MyError::Unlucky => println!("Not this time!"),
//!                 MyError::Io(e) => println!("I/O error: {}", e),
//!                 MyError::TooSmall(size) => println!("File too small: {} bytes", size),
//!             }
//!             println!("Backtrace: {}", err.stack());
//!         }
//!     }
//! }
//! ```
//!
//!
//! # Propagation Using `?`
//!
//! After a [`Result<T, E>`] has been constructed, it will keep a running "stack
//! trace" of the code locations where the `?` operator is invoked on it.
//!
//! ## Coercion Using `From`
//!
//! Any `Result<T, E>` can be coerced to a `Result<T, F>` using the `?` operator
//! if there is a [`From<E>`] defined for type `F`:
//!
//! ```
//! fn f() -> Result<(), String> {
//!     let result: Result<(), &str> = Result::new_err("str slice");
//!     result?
//! }
//! ```
//!
//! ## Coercion from `std::result::Result`
//!
//! To provide easy interoperability with standard library modules that return
//! results, any [`std::result::Result<T, E>`] can be coerced to a
//! `Result<T, E>` using the `?` operator:
//!
//! ```
//! fn f() -> Result<(), io::Error> {
//!     let result: std::result::Result<(), io::Error> = fs::File::open("foo.txt");
//!     result?
//! }
//! ```
//!
//! You can also coerce `std::result::Result<T, E>` to `Result<T, F>` if `F:
//! From<E>`:
//!
//! ```
//! fn f() -> Result<(), String> {
//!     let result = std::result::Result::Err("string slice");
//!     result?
//! }
//! ```
//!
//!
//! # Working with `Result<T, E>`
//!
//! There are a few caveats when using [`Result<T, E>`] as a replacement for the
//! std library result.
//!
//! ## Contained Value
//!
//! `propagate::Result` is defined as such:
//!
//! ```
//! enum Result<T, E> {
//!     Ok(T),
//!     Err(TracedError<E>),
//! }
//! ```
//!
//! [`TracedError<E>`] is a wrapper around an arbitrary error value, and it
//! stores a stack trace alongside the wrapped error value.
//!
//! Thus, when a `Result<T, E>` is equal to `Err(e)`, the value `e` is not of
//! type `E`, but rather it is of type `TracedError<E>`.
//!
//! Because of this, if you want to pattern match a `Result<T, E>` and get a
//! value of `E`, you must dereference the `Err(e)` value first:
//!
//! ```
//! let result: Result<(), String> = function_that_returns_result();
//! match result {
//!     Ok(_) => {}
//!     Err(e) => {
//!         println!("stack: {}", e.stack());
//!         let inner: &String = *e;
//!         println!("inner: {}", inner);
//!     }
//! }
//! ```
//!
//! ## Creating Errors
//!
//! Because `Result<T, E>` is technically a `Result<T, TracedError<E>>`, you
//! cannot construct a new error result by simply doing `Err(error_value)`.
//!
//! You can coerce your error value into an `TracedError` in one of the
//! following ways:
//!
//! ```
//! // Directly
//! let result: Result<(), u64> = Err(TracedError::new(42));
//!
//! // Using Result::new_err()
//! let result: Result<(), u64> = Result::new_err(42);
//! ```
//!
//! ## **IMPORTANT**: Forwarding Errors
//!
//! You must remember to surround result values with `Ok(..?)` when returning
//! them in a function. The compiler will not force you to do this if the result
//! value's type is identical to the function's return type.
//!
//! ```
//! fn gives_error() -> Result<(), &str> {
//!     Result::new_err("Nothing here")
//! }
//!
//! // YES: Result surrounded by Ok(..?), so the stack trace will include foo()
//! fn foo() -> Result<(), &str> {
//!     let result = gives_error();
//!     Ok(result?)
//! }
//!
//! // NO: Result returned directly, so the stack trace will not include bar()
//! fn bar() -> Result<(), &str> {
//!     let result = gives_error();
//!     result
//! }
//! ```
//!

#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(termination_trait_lib)]

pub mod error;
pub mod result;

#[doc(inline)]
pub use self::{
    error::{CodeLocation, CodeLocationStack, TracedError},
    result::{Result, Traced},
};

pub use self::result::Result::{Err, Ok};

pub mod prelude {
    pub use crate::error::TracedError;
    pub use crate::result::Result;
}

mod test;
