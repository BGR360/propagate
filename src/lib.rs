//! Error propagation tracing in Rust.
//!
//! This crate provides [`propagate::Result`], a wrapper around the standard
//! library result type that automatically tracks the propagation of error
//! results using the `?` operator.
//!
//!
//! # Examples
//!
//! Consider the following custom error type:
//!
//! ```
//! use std::io;
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
//! ```
//!
//! ## Try blocks
//!
//! Here is an example of using Propagate with [`try` blocks]:
//!
//! [`try` blocks]:
//! https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html
//!
//! ```
//! #![feature(try_blocks)]
//! # use std::io;
//! # enum MyError {
//! #     Unlucky,
//! #     Io(io::Error),
//! #     TooSmall(u64),
//! # }
//! # impl From<io::Error> for MyError {
//! #     fn from(e: io::Error) -> Self {
//! #         Self::Io(e)
//! #     }
//! # }
//! use std::fs::File;
//!
//! fn file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     try {
//!         // The `?` operator coerces `std::result::Result<_, io::Error>`
//!         // into `propagate::Result<_, MyError>`.
//!         let size = File::open(path)?.metadata()?.len();
//!
//!         if size < 1024 {
//!             Err(MyError::TooSmall(size))?
//!         }
//!
//!         size
//!     }
//! }
//!
//! fn maybe_file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     let lucky = (path.len() % 2) == 0;
//!
//!     try {
//!         if !lucky {
//!             Err(MyError::Unlucky)?
//!         }
//!
//!         file_size(path)?
//!     }
//! }
//!
//! # fn main() {
//! #     let (result, stack) = maybe_file_size("foo.txt").unpack();
//! #     match result {
//! #         Ok(size) => println!("File size: {} KiB", size / 1024),
//! #         Err(err) => {
//! #             match err {
//! #                 MyError::Unlucky => println!("Not this time!"),
//! #                 MyError::Io(e) => println!("I/O error: {}", e),
//! #                 MyError::TooSmall(size) => println!("File too small: {} bytes", size),
//! #             }
//! #             println!("Backtrace: {}", stack.unwrap());
//! #         }
//! #     }
//! # }
//! ```
//!
//! ## No try blocks
//!
//! And here is what it would look like without `try` blocks:
//!
//! ```
//! # use std::io;
//! # enum MyError {
//! #     Unlucky,
//! #     Io(io::Error),
//! #     TooSmall(u64),
//! # }
//! # impl From<io::Error> for MyError {
//! #     fn from(e: io::Error) -> Self {
//! #         Self::Io(e)
//! #     }
//! # }
//! use std::fs::File;
//!
//! fn file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     // The `?` operator coerces `std::result::Result<_, io::Error>`
//!     // into `propagate::Result<_, MyError>`.
//!     let size = File::open(path)?.metadata()?.len();
//!
//!     if size < 1024 {
//!         propagate::err(MyError::TooSmall(size))
//!     } else {
//!         propagate::ok(size)
//!     }
//! }
//!
//! fn maybe_file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     let lucky = (path.len() % 2) == 0;
//!
//!     if !lucky {
//!         propagate::err(MyError::Unlucky)
//!     } else {
//!         // Must remember to surround with `ok(..?)`.
//!         propagate::ok(file_size(path)?)
//!     }
//! }
//! ```
//!
//! [`propagate::Result`]: crate::Result

#![feature(try_trait_v2)]
#![feature(control_flow_enum)]
#![feature(termination_trait_lib)]

// TODO:
// * Add a feature flag to fall back to standard library results.
// * Massage `CodeLocation` and `CodeLocationStack` a bit.
// * Improve crate-level docs a bit.

pub mod error;
pub mod result;

#[doc(inline)]
pub use self::{
    error::{CodeLocation, CodeLocationStack, TracedError},
    result::{err, ok, Result, Traced},
};

mod test;
