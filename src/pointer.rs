#![allow(
    clippy::module_name_repetitions,
)]

//!

use core::str::FromStr;

use anyhow::{anyhow, bail};

use crate::value::Value;

///
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValuePointer {
    ///
    tokens: Vec<String>,
}

impl ValuePointer {
    ///
    pub fn get<'value>(&self, root: &'value Value) -> anyhow::Result<&'value Value> {
        self.tokens.iter().fold(Ok(root), |acc, next| {
            acc.and_then(|acc| match acc {
                &Value::Array(ref arr) => {
                    let idx = next.parse::<usize>()?;
                    arr.get(idx)
                        .ok_or_else(|| anyhow!("Index {} out of bounds", idx))
                }
                &Value::Object(ref obj) => obj
                    .get(next)
                    .ok_or_else(|| anyhow!("Key {} not found", next)),
                &(Value::Null | Value::Bool(_) | Value::String(_) | Value::Number(_)) => {
                    Err(anyhow!("Cannot index into a non-object or array value"))
                }
            })
        })
    }

    ///
    pub fn get_mut<'value>(&self, root: &'value mut Value) -> anyhow::Result<&'value mut Value> {
        self.tokens.iter().fold(Ok(root), |acc, next| {
            acc.and_then(|acc| match acc {
                &mut Value::Array(ref mut arr) => {
                    let idx = next.parse::<usize>()?;
                    arr.get_mut(idx)
                        .ok_or_else(|| anyhow!("Index {} out of bounds", idx))
                }
                &mut Value::Object(ref mut obj) => obj
                    .get_mut(next)
                    .ok_or_else(|| anyhow!("Key {} not found", next)),
                &mut (Value::Null | Value::Bool(_) | Value::String(_) | Value::Number(_)) => {
                    Err(anyhow!("Cannot index into a non-object or array value"))
                }
            })
        })
    }
}

impl FromStr for ValuePointer {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();

        let mut tokens = Vec::with_capacity(bytes.len());
        let mut buffer = Vec::with_capacity(bytes.len());

        let mut bytes_iter = bytes.iter();
        if let Some(&b'#') = bytes.first() {
            bytes_iter.next();
        }

        while let Some(byte) = bytes_iter.next() {
            match *byte {
                b'/' => {
                    if !buffer.is_empty() {
                        let token = core::str::from_utf8(&buffer)?.to_owned();
                        tokens.push(token);
                        buffer.clear();
                    }
                }
                b'~' => match bytes_iter.next() {
                    Some(&b'0') => buffer.push(b'~'),
                    Some(&b'1') => buffer.push(b'/'),
                    _ => bail!("Invalid pointer"),
                },
                _ => buffer.push(*byte),
            }
        }

        if !buffer.is_empty() {
            let token = core::str::from_utf8(&buffer)?.to_owned();
            tokens.push(token);
        }

        Ok(ValuePointer { tokens })
    }
}
