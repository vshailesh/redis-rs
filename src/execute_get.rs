use crate::database::DBError;
use crate::database::Database;
use crate::BulkString;
use crate::SimpleString;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::Error;
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufStream, BufWriter,
    ReadHalf, WriteHalf,
};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct ExecuteGet {}

impl ExecuteGet {
    pub async fn get(
        db: Arc<Database>,
        array: Vec<String>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) -> Result<(), tokio::io::Error> {
        let key = array.get(1).unwrap();
        // println!("in our Execute Get");
        match Database::get_from_db(&db, key.clone()).await {
            Ok(value) => {
                if let Err(e) = writer_ref
                    .write_all(BulkString::new(value).await.value.as_bytes())
                    .await
                {
                    eprintln!("Failed to write: {}", e);
                }
            }
            Err(DBError::DBValueExpired) => {
                if let Err(e) = writer_ref
                    .write_all(BulkString::null_bulk_string().await.value.as_bytes())
                    .await
                {
                    eprintln!("Failed to write: {}", e);
                }
            }
            Err(DBError::DBKeyDoesNotExist) => {
                let response = "Key Not Found".to_string();
                if let Err(e) = writer_ref
                    .write_all(BulkString::new(response).await.value.as_bytes())
                    .await
                {
                    eprintln!("Failed to write: {}", e);
                }
            }
            Err(DBError::DBEmpty) => {
                let response = "DB is empty".to_string();
                if let Err(e) = writer_ref
                    .write_all(BulkString::new(response).await.value.as_bytes())
                    .await
                {
                    eprintln!("Failed to write: {}", e);
                }
            } // Err()
        }

        if let Err(e) = writer_ref.flush().await {
            eprintln!("Failed to flush: {}", e);
            Err(e)
        } else {
            Ok(())
        }
    }
}
