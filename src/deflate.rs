#![allow(dead_code)]
#![allow(unused_variables)]

use std::fmt;

pub struct Error {
  kind: ErrorKind,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
  Other,
}

impl ErrorKind {
  pub fn as_str(&self) -> &'static str {
    match *self {
      ErrorKind::Other => "other deflate error",
    }
  }
}

impl fmt::Debug for Error {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt.debug_tuple("Kind").field(&self.kind).finish()
  }
}

impl fmt::Display for Error {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(fmt, "{}", self.kind.as_str())
  }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

pub fn deflate(buf: &[u8]) -> Result<Vec<u8>> {
  Ok(vec![0u8])
}

pub fn inflate(buf: &[u8]) -> Result<Vec<u8>> {
  Ok(vec![0u8])
}
