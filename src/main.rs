#![allow(unused_imports)]
use std::net::TcpListener;
use std::io::Write;
use std::io::Read;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(mut _stream) => {
                let mut buf = [0;512];
                loop {
                    let read_count = _stream.read(&mut buf).unwrap();
                    if read_count == 0 {
                        break;
                    }
                    _stream.write_all(b"+PONG\r\n").unwrap();
                }
                println!("accepted new connection");
                _stream.write_all(b"+PONG\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
