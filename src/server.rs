use crate::redis::Value;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug)]
pub struct RedisItem {
    value: String,
    created_at: Instant,
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

        let redis_item = RedisItem {
            value,
            created_at: Instant::now(),
        };
        cache.insert(key, redis_item);

        Value::SimpleString("OK".to_string())
    }

    pub fn get(&self, args: Vec<Value>) -> Value {
        let key = unpack_bulk_str(args.first().unwrap().clone()).unwrap();
        let cache = self.cache.lock().unwrap();

        match cache.get(&key) {
            Some(value) => Value::SimpleString(value.value.clone()),
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
