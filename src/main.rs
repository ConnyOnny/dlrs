extern crate hyper;
extern crate ftp;
extern crate url as urllib;

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

    let mut reader : Box<std::io::Read> = if url.starts_with("ftp://") {
        let parsed_url = urllib::Url::parse(&url).expect("invalid url");
        let mut client = ftp::FtpStream::connect(&parsed_url).unwrap();
        let mut username = parsed_url.username();
        if username.is_empty() {
            username = "anonymous";
        }
        let password = parsed_url.password().unwrap_or("blub@bla.com");
        client.login(&username, &password).unwrap();
        Box::new(client.get(parsed_url.path()).unwrap())
    } else {
        let client = hyper::client::Client::new();
        Box::new(client.get(&url).send().unwrap())
    };

    loop {
        let read_bytes = reader.read(&mut buffer).expect("reading error");
        if read_bytes == 0 {
            break;
        }
        writer.write_all(&buffer[0..read_bytes]).expect("writing error");
    }
}
