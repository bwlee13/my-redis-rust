use crate::redis::Value;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug)]
pub struct RedisItem {
    value: String,
    created_at: Instant,
    expiration: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct Server {
    cache: Arc<Mutex<HashMap<String, RedisItem>>>,
}

impl Server {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn set(&mut self, args: Vec<Value>) -> Value {
        let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
        let value = unpack_bulk_str(args.get(1).unwrap().clone()).unwrap();
        let mut cache = self.cache.lock().unwrap();

        let expiration = match args.get(2) {
            None => None,
            Some(Value::BulkString(sub_cmd)) => {
                if sub_cmd != "px" {
                    panic!("Invalid command for expiration time")
                }
                match args.get(3) {
                    None => None,
                    Some(Value::BulkString(time)) => Some(time.parse::<i64>().unwrap()),
                    _ => panic!("Invalid expiration time"),
                }
            }
            _ => panic!("Invalid expiration time"),
        };

        let redis_item = RedisItem {
            value,
            created_at: Instant::now(),
            expiration,
        };

        cache.insert(key, redis_item);

        Value::SimpleString("OK".to_string())
    }

    pub fn get(&self, args: Vec<Value>) -> Value {
        let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
        let cache = self.cache.lock().unwrap();

        match cache.get(&key) {
            Some(value) => {
                if let Some(expiration) = value.expiration {
                    let now = Instant::now();
                    if now.duration_since(value.created_at).as_millis() > expiration as u128 {
                        Value::NullBulkString
                    } else {
                        Value::SimpleString(value.value.clone())
                    }
                } else {
                    Value::SimpleString(value.value.clone())
                }
            }
            None => Value::NullBulkString,
        }
    }
}

fn unpack_bulk_str(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => Ok(s),
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}
