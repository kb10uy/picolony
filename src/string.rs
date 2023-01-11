//! Contains string manipulation.

use core::{
    cmp::Ordering,
    fmt::{Error as FmtError, Result as FmtResult, Write},
    str::{from_utf8, from_utf8_unchecked},
};

/// Width for JIS kuten pages.
pub const JIS_KUTEN_WIDTH: usize = 94;

/// Buffer to use with core::fmt functions.
pub struct FormatBuffer<const BUFFER_SIZE: usize> {
    buffer: [u8; BUFFER_SIZE],
    written: usize,
}

impl<const BUFFER_SIZE: usize> FormatBuffer<BUFFER_SIZE> {
    /// Initializes with constant size.
    pub const fn new() -> FormatBuffer<BUFFER_SIZE> {
        FormatBuffer {
            buffer: [0; BUFFER_SIZE],
            written: 0,
        }
    }

    /// Returns written bytes.
    pub fn bytes(&self) -> &[u8] {
        &self.buffer[..self.written]
    }

    /// Returns valid part as string.
    pub fn valid_str(&self) -> &str {
        let (valid, _) = extract_valid_str(&self.buffer[..self.written]);
        valid
    }

    /// Clears buffer content.
    pub fn clear(&mut self) {
        self.written = 0;
    }
}

impl<const BUFFER_SIZE: usize> Write for FormatBuffer<BUFFER_SIZE> {
    fn write_str(&mut self, s: &str) -> FmtResult {
        let str_bytes = s.as_bytes();
        let str_length = s.len();

        let buffer_left = BUFFER_SIZE - self.written;
        if str_length > buffer_left {
            return Err(FmtError);
        }

        let target = &mut self.buffer[(self.written)..(self.written + str_length)];
        target.copy_from_slice(str_bytes);
        self.written += str_length;
        Ok(())
    }
}

/// Converts Unicode codepoint to JIS kuten code.
pub struct Unicode2JisTable<'a> {
    chain_indices: &'a [u8],
    table_elements: &'a [u8],
    chain_length_bit: u32,
    elements_count: usize,
}

impl<'a> Unicode2JisTable<'a> {
    /// Constructs table by referencing byte slice.
    /// If the header information does not match the whole table size, `Err(_)` will return.
    pub fn new(table_bytes: &'a [u8]) -> Result<Unicode2JisTable<'a>, Uni2JisTableError> {
        if table_bytes.len() < 4 {
            return Err(Uni2JisTableError::InsufficientSize);
        }

        let chain_length = u16::from_le_bytes([table_bytes[0], table_bytes[1]]) as usize;
        let elements_count = u16::from_le_bytes([table_bytes[2], table_bytes[3]]) as usize;
        let chain_length_bit = chain_length.trailing_zeros();
        let chains_count: usize = 0x10000 >> chain_length_bit;

        let expected_size = 4 + (chains_count * 2) + (elements_count * 4);
        if table_bytes.len() != expected_size {
            return Err(Uni2JisTableError::IncorrectData);
        }

        let chain_indices = &table_bytes[4..(4 + chains_count * 2)];
        let table_elements = &table_bytes[(4 + chains_count * 2)..];
        Ok(Unicode2JisTable {
            chain_indices,
            table_elements,
            chain_length_bit,
            elements_count,
        })
    }

    /// Queries Unicode character.
    pub fn query(&self, c: char) -> Option<(u8, u8)> {
        let c = c as u16;
        let chain = (c as u32 >> self.chain_length_bit) as usize;
        let chain_start = u16::from_le_bytes([
            self.chain_indices[chain * 2],
            self.chain_indices[chain * 2 + 1],
        ]) as usize;
        let chain_end = (chain_start + (1 << self.chain_length_bit)).min(self.elements_count);
        for element_index in chain_start..chain_end {
            let element = &self.table_elements[(element_index * 4)..((element_index + 1) * 4)];
            let element_char = u16::from_le_bytes([element[0], element[1]]);
            match c.cmp(&element_char) {
                Ordering::Greater => continue,
                Ordering::Equal => return Some((element[2], element[3])),
                Ordering::Less => return None,
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Uni2JisTableError {
    InsufficientSize,
    IncorrectData,
}

/// Splits input slice into two part: valid UTF-8 string from beginning, and the rest.
pub fn extract_valid_str(source: &[u8]) -> (&str, &[u8]) {
    match from_utf8(source) {
        Ok(s) => (s, &[]),
        Err(err) => {
            // The former is guaranteed to be valid UTF-8.
            let (valid, rest) = source.split_at(err.valid_up_to());
            let valid_str = unsafe { from_utf8_unchecked(valid) };

            (valid_str, rest)
        }
    }
}
