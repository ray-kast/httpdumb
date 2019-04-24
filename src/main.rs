use failure::Error;
use lazy_static::lazy_static;
use regex::Regex;
use std::{
  io::{self, prelude::*},
  net::{Ipv4Addr, SocketAddrV4, TcpListener},
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

      {
        let mut string = String::new();
        stream.read_line(&mut string).unwrap();

        match FIRST_LINE_REGEX.captures(&*string) {
          Some(c) => {
            let verb = c.name("verb").unwrap();
            let path = c.name("path").unwrap();
            let ver = c.name("ver").unwrap();

            println!("verb: {:?}; path: {:?}; version: {:?}", verb, path, ver);
          },
          None => return,
        }
      }

      loop {
        let mut string = String::new();
        stream.read_line(&mut string).unwrap();

        match &*string {
          "\r\n" => break,
          s => println!("{:?}", s),
        }
      }

      let mut stream = io::BufWriter::new(stream.into_inner());

      write!(stream, "HTTP/1.1 200 OK\r\n").unwrap();
      write!(stream, "Content-Type: text/plain; charset=UTF-8\r\n").unwrap();
      write!(stream, "\r\n").unwrap();
      write!(stream, "get fukt").unwrap();

      stream.flush().unwrap();
    });
  }
}
