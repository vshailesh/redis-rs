#![allow(unused_imports)]
mod common;
mod database;
mod execute_get;
mod execute_set;
use common::Arrays;
use common::BulkString;
use common::Execute;
use common::InputReader;
use common::ParseRedisCliInput;
use common::SimpleString; // import SimpleString struct
use common::TypeString; // import BulkString struct // import Redis Arrays input struct // import Enum

mod config_feature;
use crate::config_feature::db::{CDBKey, CDBValue};
use crate::config_feature::setup_config_db;

use http::{Request, Response, StatusCode};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;
use tokio::io::split;
use tokio::io::AsyncReadExt;
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufStream, BufWriter,
    ReadHalf, WriteHalf,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
lazy_static! {
    static ref CONFIG_DB: Arc<Mutex<HashMap<CDBKey, CDBValue>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let args: Vec<String> = std::env::args().collect();
    setup_config_db(args).await;
    let db = std::sync::Arc::new(database::Database::new().await);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let db_clone = std::sync::Arc::clone(&db);

        let handle = tokio::spawn(async move {
            handle_connection(&mut socket, db_clone).await;
        });
    }
}

async fn handle_connection(stream: &mut TcpStream, db: Arc<database::Database>) {
    println!("Handling connection");

    let bufStream = BufStream::new(stream);
    let (mut read_half, mut write_half) = split(bufStream);

    let mut reader: BufReader<ReadHalf<BufStream<&mut TcpStream>>> = BufReader::new(read_half);
    let mut writer: BufWriter<WriteHalf<BufStream<&mut TcpStream>>> = BufWriter::new(write_half);

    loop {
        let mut line = String::new();

        //check the length of the BulkString array
        let mut parsed_client_input: Option<ParseRedisCliInput> = None;
        let mut full_input_array: Option<Vec<String>> = None;

        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("Client Disconnected");
                break;
            }
            Ok(total_bytes_received) => {
                println!("{line}");
                let redis_bsa_len = ParseRedisCliInput::new(line.clone());
                match redis_bsa_len {
                    Ok(obj) => {
                        parsed_client_input = Some(obj);
                        // println!("{parsed_client_input.stringType}");
                    }
                    Err(error) => {
                        println!("Some Error Occured");
                    }
                }
            }
            Err(error) => {
                println!("Error {error}");
                break;
            }
        }

        match parsed_client_input {
            Some(value) => {
                match value.stringType {
                    TypeString::Arrays(array_instance) => {
                        //iterate over array elements and get the whole input
                        let mut input_arr = Vec::new();
                        for i in 0..array_instance.size {
                            // Read the length indicator line
                            let mut length_line = String::new();
                            match reader.read_line(&mut length_line).await {
                                Ok(0) => {
                                    // eprintln!("Unexpected EOF at length indicator for element {}", i);
                                    break;
                                }
                                Ok(_) => {
                                    // println!("Read length indicator {}: '{}'", i, length_line.trim());
                                }
                                Err(e) => {
                                    // eprintln!("Error reading length indicator for element {}: {}", i, e);
                                    break;
                                }
                            }

                            // Read the actual data line
                            let mut data_line = String::new();
                            match reader.read_line(&mut data_line).await {
                                Ok(0) => {
                                    eprintln!("Error: Unexpected EOF at data for element {}", i);
                                    break;
                                }
                                Ok(_) => {
                                    input_arr.push(data_line.trim().to_string());
                                    // println!("Read data element {}: '{}'", i, data_line.trim());
                                }
                                Err(e) => {
                                    eprintln!("Error reading data for element {}: {}", i, e);
                                    break;
                                }
                            }
                        }
                        full_input_array = Some(input_arr)
                    }
                    TypeString::BulkString(_value) => {}
                    TypeString::SimpleString(_value) => {}
                }
            }
            None => {
                println!("Client Input was not parsed properly");
                continue;
            }
        }
        let db_clone = Arc::clone(&db);
        Execute::execute(Arc::clone(&db_clone), full_input_array, &mut writer).await;
    }
}

pub async fn print_input_array(arr: Option<Vec<String>>) {
    match arr {
        Some(array) => {
            for val in array {
                println!("{val} ");
            }
        }
        None => {
            println!("Input Array is empty");
        }
    }
}
