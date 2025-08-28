#![allow(unused_imports)]
use std::io;
use std::collections::VecDeque;
use std::collections::HashMap;
use http::{Request, Response, StatusCode};
use std::convert::TryInto;
use tokio::io::AsyncReadExt;
use std::io::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{BufWriter, AsyncWriteExt, AsyncWrite, BufReader, AsyncRead, BufStream, AsyncBufReadExt, ReadHalf, WriteHalf};
use tokio::io::split;
use codecrafters_redis::ParseRedisCliInput;
use codecrafters_redis::TypeString; // import Enum
use codecrafters_redis::SimpleString; // import SimpleString struct
use codecrafters_redis::BulkString; // import BulkString struct
use codecrafters_redis::Arrays; // import Redis Arrays input struct
use codecrafters_redis::InputReader;
use codecrafters_redis::Execute;


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

    let mut reader:BufReader<ReadHalf<BufStream<&mut TcpStream>>> = BufReader::new(read_half);
    let mut writer:BufWriter<WriteHalf<BufStream<&mut TcpStream>>> = BufWriter::new(write_half);
    
    loop {
        let mut line = String::new();
    
        //check the length of the BulkString array
        let mut parsed_client_input : Option<ParseRedisCliInput> = None;
        let mut full_input_array: Option<Vec<String>> = None;
    
        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("Client Disconnected")
            } 
            Ok(total_bytes_received) => {
                println!("{line}");
                let redis_bsa_len = ParseRedisCliInput::new(line.clone());
                match redis_bsa_len {
                    Ok(obj) =>  {
                        parsed_client_input = Some(obj);
                        // println!("{parsed_client_input.stringType}");
                    }
                    Err(error) => {
                        println!("Some Error Occured");
                    }
                }
            }
            Err(error) => {
                println!("Error {error}")
            }
        }
    
        match parsed_client_input {
            Some(value) => {
                match value.stringType {
                    TypeString::Arrays(array_instance) => {
                    //iterate over array elements and get the whole input
                        let mut input_arr= Vec::new();
                        for i in 0..array_instance.size {
                            // Read the length indicator line
                            let mut length_line = String::new();
                            match reader.read_line(&mut length_line).await {
                                Ok(0) => {
                                    // eprintln!("Unexpected EOF at length indicator for element {}", i);
                                    break;
                                },
                                Ok(_) => {
                                    // println!("Read length indicator {}: '{}'", i, length_line.trim());
                                },
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
                                },
                                Ok(_) => {
                                    input_arr.push(data_line.trim().to_string());
                                    // println!("Read data element {}: '{}'", i, data_line.trim());
                                },
                                Err(e) => {
                                    eprintln!("Error reading data for element {}: {}", i, e);
                                    break;
                                }
                            }
                        }
                        full_input_array = Some(input_arr)
                    }
                    TypeString::BulkString(_value) => {
                        
                    } 
                    TypeString::SimpleString(_value) => {
                        
                    }
                }
            }
            None => {
                println!("Client Input was not parsed properly");
            }
        }
    
        Execute::execute(full_input_array, &mut writer).await;
    }



    // just check if the input array is 
    // println!("printing elements of the array");
    // print_input_array(full_input_array).await;

    // loop {
    //     line.clear();
    //     match reader.read_line(&mut line).await {
    //         Ok(0) => {
    //             println!("Client disconnected");
    //             break;
    //         }
    //         Ok(total_bytes_received) => {
    //             // let obj = ParseRedisCliInput::new();

    //             println!("total_bytes_received = {total_bytes_received} ");
    //             println!("line = {line}");

    //             if line.contains("PING"){
    //                 println!("I hope it is being printed only once");
    //                 let response = "+PONG\r\n";
    //                 writer.write_all(response.as_bytes()).await;
    //                 writer.flush().await;
    //             }

    //         } 
    //         Err(err) => {
    //             println!("Error: {err}");
    //         }
    //     }
    // }


    // after putting the whole input in an array do operation on the input according to the command
    // that will be in the position 0 always, lets hope, Redis Serialization Protocol is like that
    // process input from Redis Protocol

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
