#![feature(try_blocks)]
// Overrides the std library's `Result` type.

use std::fs::File;
use std::io;

enum MyError {
    Unlucky,
    Io(io::Error),
    TooSmall(u64),
}

impl From<io::Error> for MyError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

fn file_size(path: &str) -> propagate::Result<u64, MyError> {
    try {
        // The `?` operator coerces `std::result::Result<_, io::Error>` into `Result<_, MyError>`.
        let size = File::open(path)?.metadata()?.len();

        if size < 1024 {
            Err(MyError::TooSmall(size))?
        }

        size
    }
}

fn maybe_file_size(path: &str) -> propagate::Result<u64, MyError> {
    let lucky = (path.len() % 2) == 0;

    try {
        if !lucky {
            Err(MyError::Unlucky)?
        }

        file_size(path)?
    }
}

fn main() {
    let result = maybe_file_size("foo.txt");
    match result {
        propagate::Ok(size) => println!("File size: {} KiB", size / 1024),
        propagate::Err(err) => {
            // Dereference `err` to get the actual error value.
            match &*err {
                MyError::Unlucky => println!("Not this time!"),
                MyError::Io(e) => println!("I/O error: {}", e),
                MyError::TooSmall(size) => println!("File too small: {} bytes", size),
            }
            println!("Backtrace: {:?}", err.stack());
        }
    }
}
