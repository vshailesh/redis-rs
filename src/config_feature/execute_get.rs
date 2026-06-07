use crate::config_feature::db::{CDBKey, CDBValue};
use crate::CONFIG_DB;

pub async fn get_from_cdb(key: &String) -> String {
    let lock = CONFIG_DB.lock().await;
    let key = CDBKey::new(key.clone());

    let cdbval = lock.get(&key);
    if let Some(val) = cdbval {
        val.get_string_val()
    } else {
        // Transform this function to return Result later
        "".to_string()
    }
}
