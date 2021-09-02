//! An experiment using the unstable [`std::ops::Try`] trait to do error chaining in Rust.
//!
//! # Working with `Result<T, E>`
//!
//! When a [`Result<T, E>`] is equal to `Err(e)`, the value `e` is not actually of type `E`.
//! It is of type [`StackError<E>`].
//!
//! [`StackError<E>`] is a wrapper around an arbitrary error value, and it stores a stack trace
//! alongside the wrapped error value.
//!
//! Because of this, if you want to pattern match a `Result<T, E>` and get a value of `E`, you
//! must dereference the `Err(e)` value first. See the example below.
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
//!         // Use `error_new` to start a new error chain.
//!         error_new(MyError::TooSmall(size))
//!     }
//! }
//!
//! fn maybe_file_size(path: &str) -> Result<u64, MyError> {
//!     let lucky = (path.len() % 2) == 0;
//!     if !lucky {
//!         error_new(MyError::Unlucky)
//!     } else {
//!         // Use `eforward` when directly returning a `Result<T, E>`.
//!         eforward(file_size(path))
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

#![feature(try_trait_v2)]
#![feature(control_flow_enum)]

pub mod error;
pub mod result;

#[doc(inline)]
pub use self::{
    error::{CodeLocation, CodeLocationStack, StackError},
    result::Result,
    result::{eforward, error_new},
};

pub use self::result::Result::{Err, Ok};

pub mod prelude {
    use super::result;

    pub use result::Result;
    pub use result::Result::{Err, Ok};
    pub use result::{eforward, error_new};
}

mod test;
