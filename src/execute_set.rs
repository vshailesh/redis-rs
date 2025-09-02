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

pub struct ExecuteSet{}

impl ExecuteSet {
    pub async fn set(db: Arc<Database>, array: Vec<String>, writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>) -> Result<(), tokio::io::Error>{
        let mut str : String = "OK".to_string();
        eprintln!("Reached ExecuteSet Struct");

        let key = array.get(1).unwrap();
        let value = array.get(2).unwrap();

        Database::add_to_db(&db, key.clone(), value.clone()).await;

        if let Err(e) = writer_ref.write_all(SimpleString::new(str).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
                                }
        if let Err(e) = writer_ref.flush().await {
            eprintln!("Failed to flush: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}