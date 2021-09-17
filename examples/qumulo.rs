use core::fmt;
use std::error::Error;
use std::fs::File;
use std::io;

pub mod qresult {
    pub type QTrace = propagate::ErrorTrace; // In reality, is something else.

    pub type QResult<T, E> = propagate::Result<T, E, QTrace>;

    pub use propagate::Err as QErr;
    pub use propagate::Ok as QOk;
}

#[derive(Debug)]
enum MyError {
    Unlucky,
    Io(io::Error),
    TooSmall(u64),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unlucky => write!(f, "Not this time!"),
            Self::Io(_) => write!(f, "I/O error"),
            Self::TooSmall(size) => write!(f, "File too small: {} bytes", size),
        }
    }
}

impl Error for MyError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            MyError::Unlucky => None,
            MyError::Io(e) => e.source(),
            MyError::TooSmall(_) => None,
        }
    }
}

impl From<io::Error> for MyError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

use qresult::{QErr, QOk, QResult, QTrace};

fn file_size(path: &str) -> QResult<u64, MyError> {
    let size = File::open(path)?.metadata()?.len();

    if size < 1024 {
        return QErr(MyError::TooSmall(size), QTrace::new());
    }

    QOk(size)
}

fn maybe_file_size(path: &str) -> QResult<u64, MyError> {
    let lucky = (path.len() % 2) == 0;

    if !lucky {
        return QErr(MyError::Unlucky, QTrace::new());
    }

    // Must remember to surround with `Ok(..?)`.
    QOk(file_size(path)?)
}

fn main() -> QResult<(), MyError> {
    let size = maybe_file_size("foo.txt")?;
    println!("File size: {} KiB", size / 1024);
    QOk(())
}
