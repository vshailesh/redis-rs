use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::Error;
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufStream, BufWriter,
    ReadHalf, WriteHalf,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

// mod database;
// mod execute_get;
// mod execute_set;

// use crate::config_feature::{ExecuteGet};
use super::database::Database;
use super::execute_get::ExecuteGet;
use super::execute_key::ExecuteKey;
use super::execute_set::ExecuteSet;
use crate::config_feature::*;

// my very own Redis Protocol Parser

// how would you reconcile everything with this parser?

struct ExecuteEcho {}

struct ExecutePing {}

pub struct Execute {
    arr: Option<Vec<String>>,
}

pub struct InputReader {
    value: String,
}

pub struct ParseRedisCliInput {
    pub stringType: TypeString,
}

#[derive(Debug)]
pub struct NotRedisArrayInput {}

pub enum TypeString {
    SimpleString(SimpleString),
    BulkString(BulkString),
    Arrays(Arrays),
}

pub struct SimpleString {
    pub value: String,
}

pub struct BulkString {
    pub value: String,
    pub len: usize,
}

pub struct Arrays {
    pub size: u32,
}

// impl InputReader {
//     type Reader = BufReader<ReadHalf<BufStream<&mut TcpStream>>>;
//     pub fn readNext() -> Result<Self,Error>{

//     }
// }

impl SimpleString {
    pub async fn new(value: String) -> Self {
        let newValue = "+".to_owned() + &value + "\r\n";
        Self { value: newValue }
    }
}

// impl std::fmt::Display for NotRedisArrayInput {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "({}, {})", self.x, self.y)
//     }
// }

impl BulkString {
    pub async fn new(input: String) -> Self {
        let mut newString = String::new();
        let length = input.len().to_string();
        newString = "$".to_string() + &length + &"\r\n".to_string() + &input + &"\r\n".to_string();
        Self {
            len: input.len(),
            value: newString,
        }
    }
    pub async fn toBulkString(input: String) -> Self {
        let mut newString = input.clone();
        let length = input.len().to_string();
        // let length = input.try
        newString = "$".to_string() + &length + &"\r\n".to_string() + &input + &"\r\n".to_string();

        Self {
            len: newString.len(),
            value: newString,
        }
    }

    pub async fn null_bulk_string() -> Self {
        Self {
            len: 2,
            value: "$-1\r\n".to_string(),
        }
    }

    // pub async fn add(b1: BulkString, b2: BulkString) -> BulkString {
    //     let mut len = b1.len + b2.len;
    // }
    pub async fn concatenate_bulk_string(arr: Vec<String>) -> String {
        let mut bs_array: Vec<BulkString> = Vec::new();
        for val in arr {
            let blk_2 = BulkString::new(val).await;
            bs_array.push(blk_2);
        }
        let mut concat_bulk_string = format!(
            "{}{}{}",
            "*".to_string(),
            bs_array.len().to_string(),
            "\r\n".to_string(),
        );
        for val in bs_array {
            concat_bulk_string = concat_bulk_string + val.value.as_str();
        }
        concat_bulk_string
    }
}

impl Arrays {
    fn new(size: u32) -> Self {
        println!("Size of the array is = {size}");
        Self { size }
    }

    pub fn form_array(value: String, mut arr: Option<Vec<String>>) -> Option<Vec<String>> {
        // let inputArr: Vec<String>;
        match arr {
            Some(ref mut array) => {
                array.push(value);
            }
            None => {
                println!("Can't append into None array, please initialize an array");
            }
        }
        arr
    }
}

impl ParseRedisCliInput {
    pub fn new(value: String) -> Result<ParseRedisCliInput, NotRedisArrayInput> {
        if value.chars().nth(0).unwrap() == String::from("*").chars().nth(0).unwrap() {
            Ok(ParseRedisCliInput {
                stringType: TypeString::Arrays(Arrays::new(
                    value.chars().nth(1).unwrap().to_digit(10).unwrap(),
                )),
            })
        } else {
            Err(NotRedisArrayInput {})
        }
    }
    // pub fn parseInputArrayValue(value: String) -> TypeString {

    // }
}

impl ExecuteEcho {
    async fn echo(
        array: Vec<String>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) {
        let mut output: String = String::new();

        for i in 1..array.len() {
            if i > 1 {
                output.push(' ');
            }
            output += array.get(i).unwrap();
        }
        // eprintln!("Reached ExecuteEcho Struct {}", output);

        if let Err(e) = writer_ref
            .write_all(BulkString::toBulkString(output).await.value.as_bytes())
            .await
        {
            eprintln!("Failed to write: {}", e);
        }

        if let Err(e) = writer_ref.flush().await {
            eprintln!("Failed to flush: {}", e);
        }
    }
}

impl ExecutePing {
    async fn ping(
        array: Vec<String>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) {
        let mut str: String = String::new();
        for i in 1..array.len() {
            str += array.get(i).unwrap();
        }
        str = "PONG".to_string();
        eprintln!("Reached ExecutePing Struct");
        // if let Err(e) = writer_ref.write_all(BulkString::toBulkString(str)
        //                         .value
        //                         .as_bytes()).await {
        //                             eprintln!("Failed to write: {}", e);

        if let Err(e) = writer_ref
            .write_all(SimpleString::new(str).await.value.as_bytes())
            .await
        {
            eprintln!("Failed to write: {}", e);
        }
        if let Err(e) = writer_ref.flush().await {
            eprintln!("Failed to flush: {}", e);
        }
    }
}

impl Execute {
    pub async fn execute(
        db: Arc<Database>,
        arr: Option<Vec<String>>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) {
        // eprintln!("Reacing Execute::execute fn");
        match arr {
            Some(array) => {
                match array.get(0) {
                    Some(value) => {
                        if value.eq_ignore_ascii_case("ECHO") {
                            println!("reached Execute Struct");
                            ExecuteEcho::echo(array, writer_ref).await;
                        } else if value.eq_ignore_ascii_case("PING") {
                            ExecutePing::ping(array, writer_ref).await;
                        } else if value.eq_ignore_ascii_case("SET") {
                            ExecuteSet::set(db, array, writer_ref).await;
                        } else if value.eq_ignore_ascii_case("GET") {
                            // println!("Reaching lib.rs");
                            ExecuteGet::get(db, array, writer_ref).await;
                        } else if value.eq_ignore_ascii_case("CONFIG") {
                            decider(array, writer_ref).await;
                        } else if value.eq_ignore_ascii_case("KEYS") {
                            // writer_ref.write_all("hey lady".as_bytes()).await;
                            ExecuteKey::decider(db, array, writer_ref).await;
                        }
                    }
                    None => {
                        println!("Error: Element 0 of array is missing");
                    }
                }
            }
            None => {
                println!("Error: No input array found");
            }
        }
    }
}
