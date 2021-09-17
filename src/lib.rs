//! Error propagation tracing in Rust.
//!
//! This crate provides [`propagate::Result`], a replacement for the standard
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
//! #     let result = maybe_file_size("foo.txt");
//! #     match result {
//! #         propagate::Ok(size) => println!("File size: {} KiB", size / 1024),
//! #         propagate::Err(err, trace) => {
//! #             match err {
//! #                 MyError::Unlucky => println!("Not this time!"),
//! #                 MyError::Io(e) => println!("I/O error: {}", e),
//! #                 MyError::TooSmall(size) => println!("File too small: {} bytes", size),
//! #             }
//! #             println!("Stack trace: {}", trace);
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
//! use propagate::CodeLocationStack;
//! use std::fs::File;
//!
//! fn file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     // The `?` operator coerces `std::result::Result<_, io::Error>`
//!     // into `propagate::Result<_, MyError>`.
//!     let size = File::open(path)?.metadata()?.len();
//!
//!     if size < 1024 {
//!         // Option 1: Coerce a `std::result::Result` to a`propagate::Result`
//!         // using `?`.
//!         Err(MyError::TooSmall(size))?
//!     } else {
//!         propagate::Ok(size)
//!     }
//! }
//!
//! fn maybe_file_size(path: &str) -> propagate::Result<u64, MyError> {
//!     let lucky = (path.len() % 2) == 0;
//!
//!     if !lucky {
//!         // Option 2: Directly construct a `propagate::Result`
//!         // using `CodeLocationStack::new()`.
//!         propagate::Err(MyError::Unlucky, CodeLocationStack::new())
//!     } else {
//!         // Must remember to surround with `Ok(..?)`.
//!         propagate::Ok(file_size(path)?)
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

pub mod result;
pub mod trace;

#[doc(inline)]
pub use self::{
    result::{Result, Traced},
    trace::{CodeLocation, CodeLocationStack},
};

pub use self::result::Result::{Err, Ok};

mod test;

// Test that all code snippets in README.md compile.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;
