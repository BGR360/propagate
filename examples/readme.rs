use std::fs::File;
use std::io;
use std::sync::mpsc;
use std::thread;

fn main() {
    let path = "foo.txt"; // <------------------- Does not exist.

    match file_summary(path) {
        propagate::Ok(summary) => {
            println!("{}", summary);
        }
        propagate::Err(err, trace) => {
            println!("Err: {:?}", err);
            println!("\nReturn trace: {}", trace);
        }
    }
}

fn open_file(path: &str) -> propagate::Result<File, io::Error> {
    let file = File::open(path)?; // <----------- `?` starts a new error trace.
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
    let file = open_result?; // <---------------- `?` continues the error trace.
    let size = file_size(&file)?;

    let summary = format!("{}: {} bytes", path, size);
    propagate::Ok(summary)
}
