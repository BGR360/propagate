#![feature(try_blocks)]

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
    let (result, stack) = maybe_int().unpack();
    match result {
        Ok(i) => {
            println!("Got int: {}", i);
            assert!(matches!(stack, None));
        }
        Err(e) => {
            match e {
                MyError::Str(s) => println!("Error: {}", s),
                MyError::Other => println!("Error (other)"),
            }
            println!("\nStack: {}", stack.unwrap());
        }
    }
}
