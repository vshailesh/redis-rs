use tokio::net::{TcpListener, TcpStream};
use tokio::io::{
    BufWriter, AsyncWriteExt, AsyncWrite, BufReader, AsyncRead, BufStream, AsyncBufReadExt, ReadHalf, WriteHalf
};
use tokio::io::{self, ErrorKind};
use tokio::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use chrono::{Utc, DateTime, TimeDelta};

#[derive(Debug)]
struct DBValueExpired;

#[derive(Hash, Eq, PartialEq, Debug)]
struct DBKey {
    value: String
}

pub enum DBError{
    DBValueExpired,
    DBKeyDoesNotExist
}


struct DBValue {
    creation_time: DateTime<Utc>,
    updated_time: DateTime<Utc>,
    expire_ms: TimeDelta,
    value: String
}

pub struct Database {
    db: Arc<Mutex<HashMap<DBKey, DBValue>>>
}

impl Database {
    pub async fn new() -> Self {
        let mut db = Arc::new(Mutex::new(HashMap::new()));
        Self {
            db
        }
    }

    pub async fn add_to_db_without_expire_ms(&self, key: String, value: String) -> Result<(), tokio::io::Error> {
        // eprintln!("added value into the Db");
        let mut db = self.db.lock().await;
        let dbkey = DBKey::new(key);
        let if_key_exists = db.contains_key(&dbkey);
        
        let dbvalue = DBValue::new_without_expire_ms(value);
        db.insert(dbkey, dbvalue);
        Ok(())
    }

    pub async fn add_to_db_with_expire_ms(&self, key: String, value: String, expire_ms: TimeDelta) -> Result<(), tokio::io::Error> {
        // eprintln!("added value into the Db");
        let mut db = self.db.lock().await;
        let dbkey = DBKey::new(key);
        let if_key_exists = db.contains_key(&dbkey);
        
        let dbvalue = DBValue::new_with_expire_ms(value, expire_ms);
        db.insert(dbkey, dbvalue);
        Ok(())
    }

    pub async fn remove_from_db(&self, key: String, writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>) -> Result<String, tokio::io::Error> {
        let mut db = self.db.lock().await;
        let dbkey = DBKey::new(key);
        
        if !db.contains_key(&dbkey) {
            Err(io::Error::new(ErrorKind::Other, "Key Not Found"))
        } else {
            db.remove(&dbkey);
            Ok("Removed Key".to_string())
        }
    }

    pub async fn get_from_db(&self, key: String) -> Result<String, DBError> {
        // eprintln!("got to the GET from DB function");
        let db = self.db.lock().await;

        let dbkey = DBKey::new(key);

        match db.get(&dbkey) {
            Some(value) => {
                // let creation_time = value.creation_time;
                let updated_time = value.updated_time;
                let expire_ms = value.expire_ms;

                if updated_time + expire_ms > Utc::now() {
                    Ok(value.value.clone())
                } else {
                    Err(DBError::DBValueExpired)
                }
            } 
            None => {
                Err(DBError::DBKeyDoesNotExist)
            }
        }        
    }
}


impl DBKey {
    fn new(key: String) -> Self {
        Self {
            value: key
        }
    }
}

impl DBValue {
    fn new_without_expire_ms(value: String) -> Self {
        let time_now = Utc::now();
        Self {
            creation_time: time_now,
            updated_time: time_now,
            expire_ms: TimeDelta::try_milliseconds(31_556_952_000).unwrap(),
            value: value
        }         
    }
    fn new_with_expire_ms(value: String, expire_ms: TimeDelta) -> Self {
        let time_now = Utc::now();
        Self {
            creation_time: time_now,
            updated_time: time_now,
            expire_ms: expire_ms,
            value: value
        }         
    }

    fn update_value(&self, value: String) -> Self {
        let time_now = Utc::now();
        Self {
                creation_time: self.creation_time,
                updated_time: time_now,
                expire_ms: self.expire_ms,
                value: value
        }
    }

    fn update_value_with_expire_time(&self, value: String, expire_ms: TimeDelta) -> Self {
        let time_now = Utc::now();
        Self {
                creation_time: self.creation_time,
                updated_time: time_now,
                expire_ms: expire_ms,
                value: value
        }
    }

    fn update_expire_time(&self, expire_ms: TimeDelta) -> Self {
        let time_now = Utc::now();
        Self {
                creation_time: self.creation_time,
                updated_time: time_now,
                expire_ms: expire_ms,
                value: self.value.clone()
        }
    }
}



impl std::error::Error for DBValueExpired {}
impl std::fmt::Display for DBValueExpired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Value Expired")
    }
}