use std::str::FromStr;

// TODO: Implement Eq and PartialEq manually to compare Strings and Integers
#[derive(Clone,Debug,Eq,PartialEq)]
pub enum StoreValue {
    String(String),
    Integer(i64),
}

impl From<i64> for StoreValue {
    fn from(i: i64) -> StoreValue {
        StoreValue::Integer(i)
    }
}

impl Into<i64> for StoreValue {
    fn into(self) -> i64 {
        match self {
            StoreValue::String(data) => {
                match i64::from_str(&data) {
                    Ok(i) => i,
                    Err(error) => {
                        println!("ERROR: Parsing to integer error {}", error);
                        0
                    }
                }
            }
            StoreValue::Integer(i) => i,
        }
    }
}

impl From<String> for StoreValue {
    fn from(s: String) -> StoreValue {
        StoreValue::String(s)
    }
}

impl Into<String> for StoreValue {
    fn into(self) -> String {
        match self {
            StoreValue::String(data) => {
                data
            }
            StoreValue::Integer(i) => i.to_string(),
        }
    }
}
