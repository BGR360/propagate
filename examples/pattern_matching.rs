#![feature(try_blocks)]

use propagate::CodeLocationStack;

#[derive(Debug)]
enum MyError {
    Str(&'static str),
    Other,
}

fn maybe_int() -> propagate::Result<u32, MyError> {
    try {
        Err(MyError::Str("oops"))?;
        Err(MyError::Other)?;

        42
    }
}

fn main() {
    let result = maybe_int();
    match result {
        propagate::Ok(i) => println!("Got int: {}", i),
        propagate::Err(e) => {
            let error: &MyError = e.error();
            match error {
                MyError::Str(s) => println!("Error: {}", s),
                MyError::Other => println!("Error (other)"),
            }

            let stack: &CodeLocationStack = e.stack();
            println!("\nStack: {}", stack);
        }
    }
}
