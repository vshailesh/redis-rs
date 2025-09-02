use tokio::net::{TcpListener, TcpStream};
use tokio::io::{
    BufWriter, AsyncWriteExt, AsyncWrite, BufReader, AsyncRead, BufStream, AsyncBufReadExt, ReadHalf, WriteHalf
};
use tokio::io::Error;
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use crate::SimpleString;
use crate::BulkString;
use crate::database::Database;

pub struct ExecuteGet{}

impl ExecuteGet {
    pub async fn get(db: Arc<Database>, array: Vec<String>, writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>) -> Result<(), tokio::io::Error>{
        // let mut str : String = "OK".to_string();
        // eprintln!("Reached ExecuteSet Struct");

        let key = array.get(1).unwrap();
        // println!("in our Execute Get");
        match Database::get_from_db(&db, key.clone()).await {
            Ok(value) => {
                if let Err(e) = writer_ref.write_all(BulkString::new(value).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
                                }
            }
            Err(db_error) => {
                let response = "Key Not Found".to_string();
                if let Err(e) = writer_ref.write_all(BulkString::new(response).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
                                }
            }
        }

        if let Err(e) = writer_ref.flush().await {
            eprintln!("Failed to flush: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}