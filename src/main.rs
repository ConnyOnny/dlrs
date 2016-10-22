extern crate hyper;

use std::io::Read;

const BUFFER_SIZE : usize = 1024;

fn main() {
    let mut args = std::env::args();
    let programname = args.next().expect("Program name not given as argument 0. This is not supported.");
    let usagestring = format!("Usage: {} url [outfilename]\nIf no outfilename is given, will download to stdout", programname);
    let url = args.next().expect(&usagestring);
    let stdout = std::io::stdout();
    let mut writer : Box<std::io::Write> = match args.next() {
        Some(fname) => Box::new(std::fs::File::create(fname).expect("Couldn't create output file.")),
        None => Box::new(stdout.lock()),
    };
    if args.next().is_some() {
        panic!("Too many arguments.");
    }

    let mut buffer = [0u8;BUFFER_SIZE];

    // TODO if http
    let client = hyper::client::Client::new();
    let mut response = client.get(&url).send().unwrap();
    loop {
        let read_bytes = response.read(&mut buffer).expect("reading error");
        if read_bytes == 0 {
            break;
        }
        writer.write_all(&buffer[0..read_bytes]).expect("writing error");
    }
}
