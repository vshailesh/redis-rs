use tokio::net::{TcpListener, TcpStream};
use tokio::io::{
    BufWriter, AsyncWriteExt, AsyncWrite, BufReader, AsyncRead, BufStream, AsyncBufReadExt, ReadHalf, WriteHalf
};
use tokio::io::{self, ErrorKind};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;


pub struct Database {
    db: Arc<Mutex<HashMap<String, String>>>
}

impl Database {
    pub async fn new() -> Self {
        let mut db = Arc::new(Mutex::new(HashMap::new()));
        Self {
            db
        }
    }

    pub async fn add_to_db(&self, key: String, value: String) -> Result<(), tokio::io::Error> {
        eprintln!("added value into the Db");
        let mut db = self.db.lock().await;
        db.insert(key, value);
        Ok(())
    }

    pub async fn remove_from_db(&self, key: String, writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>) -> Result<String, tokio::io::Error> {
        let mut db = self.db.lock().await;
        if !db.contains_key(&key) {
            Err(io::Error::new(ErrorKind::Other, "Key Not Found"))
        } else {
            db.remove(&key);
            Ok("Removed Key".to_string())
        }
    }

    pub async fn get_from_db(&self, key: String) -> Result<String, tokio::io::Error> {
        eprintln!("got to the GET from DB function");
        let db = self.db.lock().await;
        match db.get(&key) {
            Some(value) => {
                Ok(value.to_string())
            } 
            None => {
                Err(io::Error::new(ErrorKind::Other, "Requested key does not exist."))
            }
        }        
    }
}