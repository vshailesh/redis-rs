#![allow(unused_imports)]
use std::io;
use std::collections::VecDeque;
use std::collections::HashMap;
use http::{Request, Response, StatusCode};
use tokio::io::AsyncReadExt;
use std::io::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufWriter, AsyncWriteExt, AsyncWrite, BufReader, AsyncRead, BufStream, AsyncBufReadExt};
use tokio::io::split;

#[tokio::main] 
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        let handle = tokio::spawn(async move { 
            handle_connection(&mut socket).await;
        });
    }
}

async fn handle_connection(stream: &mut TcpStream) {
    println!("Handling connection");

    let bufStream = BufStream::new(stream);
    let (mut read_half, mut write_half) = split(bufStream);

    let mut reader = BufReader::new(read_half);
    let mut writer = BufWriter::new(write_half);

    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(data) => {
                if line.contains("PING"){
                    // println!("I hope it is being printed only once");
                    let response = "+PONG\r\n";
                    writer.write_all(response.as_bytes()).await;
                    writer.flush().await;
                }
            } 
            Err(err) => {
                println!("Error: {err}");
            }
        }
    }
}

