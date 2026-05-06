use crate::common::BulkString;
use crate::config_feature::execute_set::insert_cdb;

use crate::CONFIG_DB;
pub mod db;
pub mod execute_get;
pub mod execute_set;
use tokio::io::{AsyncWriteExt, BufStream, BufWriter, WriteHalf};
use tokio::net::TcpStream;

pub async fn setup_config_db(cdb_configs: Vec<String>) {
    // eprintln!("reached setup_config_db");
    // for val in cdb_configs {
    //     eprintln!("{}", val);
    // }
    if let Some(data) = cdb_configs.get(2) {
        // let mut lock = CONFIG_DB.lock().await;
        // lock.insert(
        //     CDBKey::new("dir".to_string()),
        //     CDBValue::new(data.to_string()),
        // );
        insert_cdb("dir".to_string(), data.clone()).await;
    }

    if let Some(data) = cdb_configs.get(4) {
        // let mut lock = CONFIG_DB.lock().await;
        // lock.insert(
        //     CDBKey::new("dbfilename".to_string()),
        //     CDBValue::new(data.to_string()),
        // );
        insert_cdb("dbfilename".to_string(), data.clone()).await;
    }
}

pub async fn decider(
    arr: Vec<String>,
    writer_ref: &mut BufWriter<WriteHalf<BufStream<&mut TcpStream>>>,
) {
    // eprintln!("reaching decider");
    // for val in arr {
    //     println!("{}", val);
    // }
    match arr.get(1).unwrap().as_str() {
        "SET" => {
            let key = arr.get(2).unwrap();
            let value = arr.get(3);
            if let Some(value) = value {
                insert_cdb(key.to_string(), value.to_string()).await;
            }
        }
        "GET" => {
            let key = arr.get(2).unwrap();
            let val = execute_get::get_from_cdb(key).await;
            let vector: Vec<String> = vec![key.clone(), val];
            if let Err(e) = writer_ref
                .write_all(BulkString::concatenate_bulk_string(vector).await.as_bytes())
                .await
            {
                eprintln!("Failed to write: {}", e);
            }
            if let Err(e) = writer_ref.flush().await {
                eprintln!("Failed to flush: {}", e);
            }
        }
        _ => {}
    }
}
