use std::fmt::Display;

use crate::deserialization::{pop_collection, pop_u8, FromBytes};

#[derive(Debug, Clone, Default)]
pub struct DomainName {
    inner: String,
}

impl DomainName {
    pub fn new(name: &str) -> DomainName {
        DomainName {
            inner: name.to_string(),
        }
    }
    pub fn empty() -> DomainName {
        DomainName::new("")
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.inner.len());
        let parts = self.inner.split('.');
        for part in parts {
            let len = part.len();
            buf.push(len as u8);
            buf.extend_from_slice(part.as_bytes());
        }
        buf.push(0);
        buf
    }
}

impl Display for DomainName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <String as Display>::fmt(&self.inner, f)
    }
}

impl FromBytes for DomainName {
    fn from_bytes(buf: &[u8], cursor: &mut usize) -> Option<Self> {
        let max_cursor: usize = *cursor;
        let mut parts = Vec::new();
        loop {
            let len = pop_u8(buf, cursor)? as u16;
            if len == 0 {
                break;
            } else if (len & 0b11000000) == 0b11000000 {
                let lo = pop_u8(buf, cursor)? as u16;
                let hi = (len & 0b00111111) << 8;
                let mut pointer = (hi | lo) as usize;
                if pointer < max_cursor {
                    // recurse
                    let DomainName { inner: ending } =
                        <DomainName as FromBytes>::from_bytes(buf, &mut pointer)?;
                    let mut start = parts.join(".");
                    if start.is_empty() {
                        start.push('.');
                    }
                    start.push_str(&ending);
                    return Some(DomainName { inner: start });
                } else {
                    // todo: should be an error
                    return None;
                }
            }
            let string: String = pop_collection::<char>(buf, cursor, len as usize)?
                .iter()
                .collect();
            parts.push(string);
        }
        Some(DomainName {
            inner: parts.join("."),
        })
    }
}
