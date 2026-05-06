use crate::config_feature::db::{CDBKey, CDBValue};
use crate::CONFIG_DB;

pub async fn insert_cdb(key: String, value: String) {
    let mut lock = CONFIG_DB.lock().await;
    let key = CDBKey::new(key);
    let value = CDBValue::new(value);

    lock.insert(key, value);
}
