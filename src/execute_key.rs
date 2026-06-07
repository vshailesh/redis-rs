use std::fmt::write;

use crate::common::BulkString;
use crate::database::Database;
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, BufStream, BufWriter,
    ReadHalf, WriteHalf,
};
use tokio::net::{TcpListener, TcpStream};
pub struct ExecuteKey {}
impl ExecuteKey {
    async fn get_all_keys(
        db: std::sync::Arc<Database>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) {
        // eprintln!("should be here buddy -> get_all_keys -> execute_key.rs");
        let all_keys = db.get_all_keys_star().await;
        // eprintln!("got the result back buddy");
        if let Ok(vec_keys) = all_keys {
            let response = BulkString::concatenate_bulk_string(vec_keys).await;
            // eprintln!("{:?}", response);
            if let Err(e) = writer_ref.write_all(response.as_bytes()).await {
                eprintln!("Failed to write: {}", e);
            }
            if let Err(e) = writer_ref.flush().await {
                eprintln!("Failed to flush: {}", e);
            }
        } else {
            eprintln!("AN ERROR OCCURED");
        }
    }
    pub async fn decider(
        db: std::sync::Arc<Database>,
        array: Vec<String>,
        writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
    ) {
        let key_pattern = array.get(1).unwrap();
        if key_pattern.contains("*") {
            Self::get_all_keys(std::sync::Arc::clone(&db), writer_ref).await;

            //pritntln all keys using writer_ref
        } else {
            /*implement this piece when needed*/
        }
    }
}
