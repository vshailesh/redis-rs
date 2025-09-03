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
use chrono::{TimeDelta};
use std::any::type_name; // remove after testing

pub struct ExecuteSet{}

impl ExecuteSet {
    pub async fn set(db: Arc<Database>, array: Vec<String>, writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>) -> Result<(), tokio::io::Error>{
        let mut str : String = "OK".to_string();
        eprintln!("Reached ExecuteSet Struct");

        let key = array.get(1).unwrap();
        let value = array.get(2).unwrap();
        let px = array.get(3);
        let expire_ms_option = array.get(4);

        if let Some(timeout) = expire_ms_option {
            let millisecond_expire_ms_option = timeout.parse::<i64>();            
            if let Ok(millisecond_expire_ms) = millisecond_expire_ms_option {
                let _ = Database::add_to_db_with_expire_ms(&db, key.clone(), value.clone(), TimeDelta::try_milliseconds(millisecond_expire_ms).unwrap()).await;
                
                //Write reponse to the TCP Stream after writing to db.
                if let Err(e) = writer_ref.write_all(SimpleString::new(str).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
                                }

            } else {
                let error_response = "Expire MS value Not an Integer".to_string();
                if let Err(e) = writer_ref.write_all(SimpleString::new(error_response).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
                                }
            }
        } else {
            let _ = Database::add_to_db_without_expire_ms(&db, key.clone(), value.clone()).await;
            if let Err(e) = writer_ref.write_all(SimpleString::new(str).await
                                .value
                                .as_bytes()).await {
                                    eprintln!("Failed to write: {}", e);
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