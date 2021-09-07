use core::fmt;
use std::error::Error;
use std::fs::File;
use std::io;

use propagate::TracedError;

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

fn file_size(path: &str) -> propagate::Result<u64, MyError> {
    // `?` coerces `std::result::Result<_, io::Error>` into `propagate::Result<_, MyError>`.
    let size = File::open(path)?.metadata()?.len();

    if size < 1024 {
        // Option 1: Coerce a `std::result::Result` to a `propagate::Result` using `?`.
        Err(MyError::TooSmall(size))?
    } else {
        propagate::Ok(size)
    }
}

fn maybe_file_size(path: &str) -> propagate::Result<u64, MyError> {
    let lucky = (path.len() % 2) == 0;

    if !lucky {
        // Option 2: Directly construct a `propagate::Result` using `TracedError::new()`.
        propagate::Err(TracedError::new(MyError::Unlucky))
    } else {
        propagate::Ok(file_size(path)?)
    }
}

fn main() -> propagate::Result<(), MyError> {
    let size = maybe_file_size("foo.txt")?;
    println!("File size: {} KiB", size / 1024);
    propagate::Ok(())
}
