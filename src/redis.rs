use anyhow::Result;
use bytes::BytesMut;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

// 1.: *x = x number of components, command, command + args etc
// 2.: $x = x length command
// 3.: command
// 4?: $x = x length of argument
// 5?: argument/s

// "hello"
// $5\r\nhello\r\n

// "hello world"
// *2\r\n$5\r\nhello\r\n$5\r\nworld\r\n
// ["hello", "world"]

#[derive(Clone, Debug)]
pub enum Value {
    SimpleString(String),
    BulkString(String),
    Array(Vec<Value>),
    NullBulkString,
}

impl Value {
    pub fn serialize(self) -> String {
        match self {
            Value::SimpleString(s) => format!("+{}\r\n", s),
            Value::BulkString(s) => format!("${}\r\n{}\r\n", s.len(), s),
            Value::NullBulkString => "$-1\r\n".to_string(),
            _ => panic!("Unsupported value for serialize"),
        }
    }
}

pub struct RedisHandler {
    stream: TcpStream,
    buffer: BytesMut,
}

impl RedisHandler {
    pub fn new(stream: TcpStream) -> Self {
        RedisHandler {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn read_value(&mut self) -> Result<Option<Value>> {
        let bytes_read = self.stream.read_buf(&mut self.buffer).await?;

        if bytes_read == 0 {
            return Ok(None);
        }
        let (v, _) = parse_message(self.buffer.split())?;
        Ok(Some(v))
    }

    pub async fn write_value(&mut self, value: Value) -> Result<()> {
        let _ = self.stream.write(value.serialize().as_bytes()).await;
        Ok(())
    }
}

fn parse_message(buffer: BytesMut) -> Result<(Value, usize)> {
    match buffer[0] as char {
        '+' => parse_simple_string(buffer),
        '*' => parse_array(buffer),
        '$' => parse_bulk_string(buffer),
        _ => Err(anyhow::anyhow!("Not a known value type {:?}", buffer)),
    }
}

fn parse_simple_string(buffer: BytesMut) -> Result<(Value, usize)> {
    if let Some((curr, len)) = read_until_crlf(&buffer[1..]) {
        let curr_string = String::from_utf8(curr.to_vec()).unwrap();

        return Ok((Value::SimpleString(curr_string), len + 1));
    }

    Err(anyhow::anyhow!("Invalid string: {:?}", buffer))
}

fn parse_array(buffer: BytesMut) -> Result<(Value, usize)> {
    let (num_vals, mut bytes_consumed) = if let Some((curr, len)) = read_until_crlf(&buffer[1..]) {
        let num_vals = parse_int(curr)?;
        (num_vals, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let mut items = vec![];
    for _ in 0..num_vals {
        let (arr_item, len) = parse_message(BytesMut::from(&buffer[bytes_consumed..]))?;
        items.push(arr_item);
        bytes_consumed += len
    }

    Ok((Value::Array(items), bytes_consumed))
}

fn parse_bulk_string(buffer: BytesMut) -> Result<(Value, usize)> {
    let (bulk_str_len, bytes_consumed) = if let Some((curr, len)) = read_until_crlf(&buffer[1..]) {
        let bulk_str_len = parse_int(curr)?;
        (bulk_str_len, len + 1)
    } else {
        return Err(anyhow::anyhow!("Invalid array format {:?}", buffer));
    };

    let end_bulk_str = bytes_consumed + bulk_str_len as usize;
    let total_parsed = end_bulk_str + 2; // 2 for \r\n

    Ok((
        Value::BulkString(String::from_utf8(
            buffer[bytes_consumed..end_bulk_str].to_vec(),
        )?),
        total_parsed,
    ))
}

fn read_until_crlf(buffer: &[u8]) -> Option<(&[u8], usize)> {
    for i in 1..buffer.len() {
        if buffer[i - 1] == b'\r' && buffer[i] == b'\n' {
            return Some((&buffer[0..(i - 1)], i + 1));
        }
    }

    None
}

fn parse_int(buffer: &[u8]) -> Result<i64> {
    Ok(String::from_utf8(buffer.to_vec())?.parse::<i64>()?)
}
