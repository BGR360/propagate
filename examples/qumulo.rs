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

fn main() {
    let result = maybe_file_size("foo.txt");
    match result {
        QOk(size) => println!("File size: {} KiB", size / 1024),
        QErr(err, trace) => {
            match err {
                MyError::Unlucky => println!("Not this time!"),
                MyError::Io(_) => println!("I/O error"),
                MyError::TooSmall(size) => println!("File too small: {} bytes", size),
            }
            println!("Stack trace: {}", trace);
        }
    }
}
