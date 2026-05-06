#[derive(Eq, Hash, PartialEq, Debug)]
pub struct CDBKey {
    key: String,
}

pub struct CDBValue {
    value: String,
}

impl CDBKey {
    pub fn new(key: String) -> Self {
        Self { key }
    }
}

impl CDBValue {
    pub fn new(value: String) -> Self {
        Self { value }
    }
    pub fn get_string_val(&self) -> String {
        self.value.clone()
    }
}
