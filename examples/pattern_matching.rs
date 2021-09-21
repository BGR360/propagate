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
    let result = maybe_int();
    match result {
        propagate::Ok(i) => println!("Got int: {}", i),
        propagate::Err(err, trace) => {
            match err {
                MyError::Str(s) => println!("Error: {}", s),
                MyError::Other => println!("Error (other)"),
            }

            println!("\nReturn trace: {}", trace);
        }
    }
}
