extern crate hyper;
extern crate ftp;
extern crate url as urllib;

use std::io::Read;
use std::fs::OpenOptions;

const BUFFER_SIZE : usize = 1024;

struct Config {
    continue_mode : ContinueMode,
}

impl Config {
    fn new() -> Config {
        Config { continue_mode : ContinueMode::Continue }
    }
}

enum ContinueMode {
    AlwaysOverwrite,
    Continue,
    // TODO auto mode: check some bytes for equality then continue
}

enum GetError {
    MustReadFromBeginning(Box<Read>),
    // TODO other errors with nice into() support for try!
}

impl std::fmt::Debug for GetError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            GetError::MustReadFromBeginning(_) => try!(f.write_str("GetError::MustReadFromBeginning"))
        };
        Ok(())
    }
}

type GetResult = Result<Box<Read>,GetError>;

fn must_read_from_beginning(get_result : &GetResult) -> bool {
    match *get_result {
        Ok(_) => false,
        Err(ref e) => {
            match *e {
                GetError::MustReadFromBeginning(_) => true
            }
        }
    }
}

trait Transport {
    fn get(url: &str, start_byte: u64) -> GetResult;
}

struct HttpTransport;

impl Transport for HttpTransport {
    fn get(url: &str, start_byte: u64) -> Result<Box<Read>,GetError> {
        use hyper::header::{Range, ByteRangeSpec};
        let client = hyper::client::Client::new();
        let mut request_builder = client.get(url);
        if start_byte != 0 {
            request_builder = request_builder.header(Range::Bytes(vec![ByteRangeSpec::AllFrom(start_byte)]));
        }
        Ok(Box::new(request_builder.send().unwrap()))
    }
}

struct FtpTransport;

impl Transport for FtpTransport {
    fn get(url: &str, start_byte: u64) -> Result<Box<Read>,GetError> {
        let parsed_url = urllib::Url::parse(url).expect("invalid url");
        let mut client = ftp::FtpStream::connect(&parsed_url).unwrap();
        let username = match parsed_url.username() {
            "" => "anonymous",
            x => x,
        };
        let password = parsed_url.password().unwrap_or("anonymous@example.com");
        client.login(&username, &password).unwrap();
        let reader_box = Box::new(client.get(parsed_url.path()).unwrap());
        if start_byte == 0 {
            Ok(reader_box)
        } else {
            Err(GetError::MustReadFromBeginning(reader_box))
        }
    }
}

fn main() {
    let mut config = Config::new();
    // TODO read config from config file, environment, command line arguments
    let mut args = std::env::args();
    let programname = args.next().expect("Program name not given as argument 0. This is not supported.");
    let usagestring = format!("Usage: {} url [outfilename]\nIf no outfilename is given, will download to stdout", programname);
    let url = args.next().expect(&usagestring);
    let stdout = std::io::stdout();
    let fileparam = args.next();
    if args.next().is_some() {
        panic!("Too many arguments.");
    }
    let previous_file_size = match config.continue_mode {
        ContinueMode::Continue => {
            match fileparam {
                Some(ref fname) => {
                    match std::fs::File::open(fname) {
                        Ok(file) => Some(file.metadata().unwrap().len()),
                        Err(_) => None,
                    }
                }
                None => None,
            }
        }
        ContinueMode::AlwaysOverwrite => None,
    };
    let mut writer : Box<std::io::Write> = match fileparam {
        Some(fname) => {
            if previous_file_size.is_some() {
                let mut oo = OpenOptions::new();
                oo.append(true);
                Box::new(oo.open(fname).expect("Couldn't create output file."))
            } else {
                Box::new(std::fs::File::create(fname).unwrap())
            }
        }
        None => Box::new(stdout.lock()),
    };

    let mut buffer = [0u8;BUFFER_SIZE];
    let get_result : GetResult = if url.starts_with("ftp://") {
        FtpTransport::get(&url, previous_file_size.unwrap_or(0))
    } else {
        HttpTransport::get(&url, previous_file_size.unwrap_or(0))
    };
    // TODO handle the case if continuation is not supported
    let mut mrfb = must_read_from_beginning(&get_result);
    let mut reader = match get_result {
        Ok(r) => r,
        Err(e) => match e {
            GetError::MustReadFromBeginning(r) =>  {
                // TODO skip the needed amount of bytes
                r
            }
        },
    };
    loop {
        let read_bytes = reader.read(&mut buffer).expect("reading error");
        if read_bytes == 0 {
            break;
        }
        writer.write_all(&buffer[0..read_bytes]).expect("writing error");
    }
}
