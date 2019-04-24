use failure::Error;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
  fs::{self, File},
  io::{self, prelude::*},
  net::{Ipv4Addr, SocketAddrV4, TcpListener},
  path::{Path, PathBuf},
  thread,
};

fn main() { run().unwrap() }

fn run() -> Result<(), Error> {
  let sock = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080))?;

  loop {
    let accepted = sock.accept()?;

    thread::spawn(move || {
      lazy_static! {
        static ref FIRST_LINE_REGEX: Regex =
          Regex::new("^(?P<verb>\\S+)\\s+(?P<path>\\S+)\\s+HTTP/(?P<ver>\\S+)\\s*\r\n$").unwrap();
      }

      let (stream, _addr) = accepted;
      let mut stream = io::BufReader::new(stream);
      let mut string = String::new();
      let path;

      {
        string.clear();
        stream.read_line(&mut string).unwrap();

        match FIRST_LINE_REGEX.captures(&*string) {
          Some(c) => {
            // let verb = c.name("verb").unwrap();
            let path_cap = c.name("path").unwrap();
            // let ver = c.name("ver").unwrap();

            path = path_cap.as_str().to_owned();
          },
          None => return,
        }
      }

      loop {
        string.clear();
        stream.read_line(&mut string).unwrap();

        match &*string {
          "\r\n" => break,
          _ => {},
        }
      }

      let mut stream = io::BufWriter::new(stream.into_inner());
      let path = PathBuf::from(path.trim_start_matches("/"));

      fn four04<W: Write>(stream: &mut W) {
        write!(stream, "HTTP/1.1 404 NotFound\r\n").unwrap();
        write!(stream, "Content-Type: text/plain; charset=UTF-8\r\n").unwrap();
        write!(stream, "\r\nThe file you requested was not found.").unwrap();
      }

      fn five00<W: Write, E: Into<Error>>(stream: &mut W, err: E) {
        write!(stream, "HTTP/1.1 500 ServerError\r\n").unwrap();
        write!(stream, "Content-Type: text/plain; charset=UTF-8").unwrap();
        write!(stream, "\r\n{:#?}", err.into()).unwrap();
      }

      println!("{:?}", path);

      match fs::metadata(path.clone()) {
        Ok(m) => {
          if !m.is_file() {
            four04(&mut stream);
            return;
          }
        },
        Err(ref e) if e.kind() == io::ErrorKind::NotFound => {
          four04(&mut stream);
          return;
        },
        Err(e) => {
          five00(&mut stream, e);
          return;
        },
      }

      match File::open(path) {
        Ok(mut f) => {
          write!(stream, "HTTP/1.1 200 OK\r\n").unwrap();
          write!(stream, "Content-Type: text/plain; charset=UTF-8\r\n").unwrap();
          write!(stream, "\r\n").unwrap();

          io::copy(&mut f, &mut stream).unwrap();
        },
        Err(e) => five00(&mut stream, e),
      }

      stream.flush().unwrap();
    });
  }
}
