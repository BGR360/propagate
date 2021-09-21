# Propagate

Error propagation tracing in Rust.

This crate provides `propagate::Result`, a replacement for the standard
library result type that automatically tracks the propagation of error
results using the `?` operator.

## Usage

### Documentation

View the API docs at https://bgr360.github.io/propagate/propagate/.

See [examples/](examples/) for some more examples showing the usage of the
Propagate crate.

### Example

Here is a motivating example where a `propagate::Result` is propagated
from one thread to another:

```rust
use std::fs::File;
use std::io;
use std::sync::mpsc;
use std::thread;

fn open_file(path: &str) -> propagate::Result<File, io::Error> {
    let file = File::open(path)?; // <-------- If open failed, `?` starts a new error trace.
    propagate::Ok(file)
}

fn file_size(file: &File) -> propagate::Result<u64, io::Error> {
    let size = file.metadata()?.len();
    propagate::Ok(size)
}

fn file_summary(path: &'static str) -> propagate::Result<String, io::Error> {
    let (tx, rx) = mpsc::channel();

    // Open the file on a separate thread, send result to this thread.
    thread::spawn(move || {
        let open_result = open_file(path);
        tx.send(open_result).unwrap();
    });

    // Make the summary on this thread.
    let open_result = rx.recv().unwrap();
    let file = open_result?; // <------------- If open failed, `?` continues the error trace.
    let size = file_size(&file)?;

    let summary = format!("{}: {} bytes", path, size);
    propagate::Ok(summary)
}

fn main() {
    let result = file_summary("foo.txt");

    match result {
        propagate::Ok(summary) => {
            println!("{}", summary);
        }
        propagate::Err(err, trace) => {
            println!("Err: {:?}", err);
            println!("\nStack trace: {}", trace);
        }
    }
}
```

Output:

```txt
Err: Os { code: 2, kind: NotFound, message: "No such file or directory" }

Stack trace: 
   0: examples/readme.rs:7
   1: examples/readme.rs:27
```


## Why Propagate?

Being able to trace the cause of an error is critical for many types of
software written in Rust. For easy diagnosis, errors should provide some
sort of **trace** denoting source code locations that contributed to the
error.

Crates such as [`anyhow`][anyhow] provide easy access to backtraces when
creating errors. The Propagate crate provides something similar but more
powerful, which I call **propagation tracing**:
every time the `?` operator is applied to an error result, the code location
of that `?` invocation is appended to a running "stack trace" stored in the
result.

Propagation tracing differs from runtime backtracing in a few important
ways. You should evaluate which approach is appropriate for your
application.

[anyhow]: https://docs.rs/anyhow/latest/anyhow/

### Advantages of Propagation Tracing

**Multithreaded tracing**

A backtrace provides a single point-in-time capture of a call stack on a
single thread. In complex software, error results may pass between multiple
threads on their way up to their final consumers.

Propagate provides a true view into the path that an error takes
through your code, even if it passes between multiple threads.

**Low performance overhead**

Runtime backtracing requires unwinding stacks and mapping addresses to
source code locations symbols at runtime.

With Propagate, the information for each code location is compiled statically
into your application's binary, and the stack trace is built up in real time as
the error propagates from function to function.

### Disadvantages of Propagation Tracing

**Code size**

Propagate stores the code location of every `?` invocation in the static
section of your application or library's binary.

**Boilerplate**

Propagate results require a bit more attention to work with compared to using
the standard library `Result` type. Much of this can be avoided if you elect to
use [`try` blocks][try blocks].

See the crate docs for more details.

[try blocks]: https://doc.rust-lang.org/beta/unstable-book/language-features/try-blocks.html



## Try It Out

### Building

Propagate requires [`#[feature(try_trait_v2)]`][try] and
[`#[feature(control_flow_enum)]`][control]. Build with Rust nightly:

```txt
cargo +nightly build
```

[try]: https://github.com/rust-lang/rust/issues/84277
[control]: https://github.com/rust-lang/rust/issues/75744

### Examples

To run one of the examples, like [`examples/usage.rs`](examples/usage.rs), do:

```txt
cargo +nightly run --example usage
```

### Tests

To run tests:

```txt
cargo +nightly test
```
