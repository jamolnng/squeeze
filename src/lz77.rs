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

pub struct Compressor {
  reference_prefix: char,
  reference_prefix_code: i64,
  reference_int_base: i64,
  reference_int_floor_code: i64,
  reference_int_ceil_code: i64,
  max_string_distance: i64,
  min_string_length: i64,
  max_string_length: i64,
  max_window_length: i64,
  default_window_length: i64,
}

impl Compressor {
  pub fn new() -> Self {
    let reference_prefix: char = '`';
    let reference_prefix_code: i64 = reference_prefix as i64;
    let reference_int_base: i64 = 96;
    let reference_int_floor_code: i64 = b' ' as i64;
    let reference_int_ceil_code: i64 = reference_int_floor_code + reference_int_base - 1;
    let max_string_distance: i64 = reference_int_base.pow(2) - 1;
    let min_string_length: i64 = 5;
    let max_string_length: i64 = reference_int_base - 1 + min_string_length;
    let max_window_length: i64 = max_string_distance + min_string_length;
    let default_window_length = 144;
    Compressor {
      reference_prefix: reference_prefix,
      reference_prefix_code: reference_prefix_code,
      reference_int_base: reference_int_base,
      reference_int_floor_code: reference_int_floor_code,
      reference_int_ceil_code: reference_int_ceil_code,
      max_string_distance: max_string_distance,
      min_string_length: min_string_length,
      max_string_length: max_string_length,
      max_window_length: max_window_length,
      default_window_length: default_window_length,
    }
  }

  pub fn compress(&self, data: &[u8], mut window_length: i64) -> Vec<u8> {
    if window_length == 0 {
      window_length = self.default_window_length;
    }
    let mut out = Vec::new();
    let mut pos = 0;
    let last = data.len() as i64 - self.min_string_length;

    while pos < last {
      let mut search_start = std::cmp::max(pos - window_length, 0);
      let mut match_length = self.min_string_length;
      let mut found_match = false;
      let mut best_match_distance = self.max_string_distance;
      let mut best_match_length = 0;
      let mut new_compressed: Vec<u8>;

      while (search_start + match_length) < pos {
        let m1 = &data[search_start as usize..(search_start + match_length) as usize];
        let m2 = &data[pos as usize..(pos + match_length) as usize];
        let is_valid_match = (m1 == m2) && match_length < self.max_string_length;

        if is_valid_match {
          match_length += 1;
          found_match = true;
        } else {
          let real_match_length = match_length - 1;
          if found_match && real_match_length > best_match_length {
            best_match_distance = pos - search_start - real_match_length;
            best_match_length = real_match_length;
          }
          match_length = self.min_string_length;
          search_start += 1;
          found_match = false;
        }
      }

      if best_match_length != 0 {
        new_compressed = vec![self.reference_prefix as u8];
        for i in self.encode_referennce_int(best_match_distance, 2) {
          new_compressed.push(i);
        }
        for i in self.encode_reference_length(best_match_length) {
          new_compressed.push(i);
        }
        pos += best_match_length;
      } else {
        if data[pos as usize] != self.reference_prefix as u8 {
          new_compressed = vec![data[pos as usize]];
        } else {
          new_compressed = vec![self.reference_prefix as u8, self.reference_prefix as u8];
        }
        pos += 1;
      }

      for i in new_compressed {
        out.push(i);
      }
    }

    let toin = &data[pos as usize..];
    for to in toin {
      if *to == self.reference_prefix as u8 {
        out.push(*to);
      }
      out.push(*to);
    }
    out
  }

  fn encode_referennce_int(&self, mut value: i64, width: i64) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::new();
    if value < 0 || value >= (self.reference_int_base.pow(width as u32) - 1) {
      panic!(
        "Reference value out of range: {} (width = {})",
        value, width
      );
    }
    while value > 0 {
      out.insert(
        0,
        ((value % self.reference_int_base) + self.reference_int_floor_code) as u8,
      );
      value /= self.reference_int_base;
    }
    let missing_length = width - out.len() as i64;
    for i in 0..missing_length {
      out.insert(0, self.reference_int_floor_code as u8);
    }
    out
  }

  fn encode_reference_length(&self, length: i64) -> Vec<u8> {
    self.encode_referennce_int((length - self.min_string_length) as i64, 1)
  }

  pub fn decompress(&self, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut pos: i64 = 0;

    while (pos as usize) < data.len() {
      let cur = data[pos as usize];
      if cur != self.reference_prefix as u8 {
        out.push(cur);
        pos += 1;
      } else {
        let next = data[pos as usize + 1];
        if next != self.reference_prefix as u8 {
          let distance = self.decode_reference_int(&data[pos as usize + 1..pos as usize + 3], 2);
          let length = self.decode_reference_length(&data[pos as usize + 3..pos as usize + 4]);
          let start = out.len() as i64 - distance - length;
          let end = start + length;
          for i in start..end {
            out.push(out[i as usize]);
          }
          pos += self.min_string_length - 1;
        } else {
          out.push(self.reference_prefix as u8);
          pos += 2;
        }
      }
    }

    out
  }

  fn decode_reference_int(&self, data: &[u8], width: u64) -> i64 {
    let mut value = 0;
    for i in 0..width {
      value *= self.reference_int_base;
      let code = data[i as usize] as i64;
      if code >= self.reference_int_floor_code && code <= self.reference_int_ceil_code {
        value += code - self.reference_int_floor_code
      } else {
        panic!("Invalid character code {}", data[i as usize] as char);
      }
    }
    value
  }

  fn decode_reference_length(&self, data: &[u8]) -> i64 {
    self.decode_reference_int(data, 1) + self.min_string_length
  }
}
