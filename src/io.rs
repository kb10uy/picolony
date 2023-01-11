//! Formatting functionalities for no_std environment.

use crate::string::extract_valid_str;

/// Constructs string lines from small packets.
pub struct LineReader<const BUFFER_SIZE: usize> {
    ongoing_buffer: [u8; BUFFER_SIZE],
    ongoing_written: usize,
    ready_buffer: [u8; BUFFER_SIZE],
    ready_size: usize,
}

impl<const BUFFER_SIZE: usize> LineReader<BUFFER_SIZE> {
    /// Initializes with constant size.
    pub const fn new() -> LineReader<BUFFER_SIZE> {
        LineReader {
            ongoing_buffer: [0; BUFFER_SIZE],
            ongoing_written: 0,
            ready_buffer: [0; BUFFER_SIZE],
            ready_size: 0,
        }
    }

    /// If any line is ready, returns it.
    pub fn ready_bytes(&self) -> Option<&[u8]> {
        (self.ready_size != 0).then(|| &self.ready_buffer[..(self.ready_size)])
    }

    /// If no bytes are received or first byte is invalid, returns `None`.
    /// Otherwise, non-empty string will return.
    pub fn ready_str(&self) -> Option<&str> {
        if self.ready_size == 0 {
            return None;
        }

        let (s, _) = extract_valid_str(&self.ready_buffer[..(self.ready_size)]);
        if s != "" {
            Some(s)
        } else {
            None
        }
    }

    /// Clears line.
    pub fn clear(&mut self) {
        self.ready_size = 0;
    }

    /// Polls to read new packet data.
    /// If newline bytes are found, `ready_bytes` will be updated.
    pub fn poll_read(&mut self, arrived_bytes: &[u8]) -> bool {
        let mut ready_updated = false;
        for &byte in arrived_bytes {
            match byte {
                b'\n' | b'\r' if self.ongoing_written == 0 => continue,
                b'\n' | b'\r' => {
                    let ready_target = &mut self.ready_buffer[..(self.ongoing_written)];
                    ready_target.copy_from_slice(&self.ongoing_buffer[..(self.ongoing_written)]);
                    self.ready_size = self.ongoing_written;
                    self.ongoing_written = 0;
                    ready_updated = true;
                }

                _ if self.ongoing_written >= BUFFER_SIZE => continue,
                b => {
                    self.ongoing_buffer[self.ongoing_written] = b;
                    self.ongoing_written += 1;
                }
            }
        }

        ready_updated
    }
}

/// Keeps bytes to write and manages the position.
pub struct LineWriter<const BUFFER_SIZE: usize> {
    buffer: [u8; BUFFER_SIZE],
    size: usize,
    written: usize,
}

impl<const BUFFER_SIZE: usize> LineWriter<BUFFER_SIZE> {
    /// Initializes with constant size.
    pub const fn new() -> LineWriter<BUFFER_SIZE> {
        LineWriter {
            buffer: [0; BUFFER_SIZE],
            size: 0,
            written: 0,
        }
    }

    /// Whether current data is written completely.
    pub fn is_completed(&self) -> bool {
        self.written >= self.size
    }

    /// Whether it started to write current data.
    pub fn is_writing(&self) -> bool {
        self.written != 0
    }

    /// Clears buffer.
    pub fn clear(&mut self) {
        if self.is_writing() && !self.is_completed() {
            return;
        }

        self.abort();
    }

    /// Forcely clears buffer.
    pub fn abort(&mut self) {
        self.size = 0;
        self.written = 0;
    }

    /// Sets new line.
    /// Only available if previous line was written completely.
    pub fn set_line(&mut self, bytes: &[u8]) {
        let length = bytes.len();
        if length > BUFFER_SIZE || (self.is_writing() && !self.is_completed()) {
            return;
        }

        let target = &mut self.buffer[..length];
        target.copy_from_slice(bytes);
        self.size = length;
        self.written = 0;
    }

    /// Appends bytes.
    /// Only available if no bytes are written.
    pub fn append(&mut self, bytes: &[u8]) {
        let length = bytes.len();
        let buffer_left = BUFFER_SIZE - self.size;
        if self.is_writing() || length > buffer_left {
            return;
        }

        let target = &mut self.buffer[(self.size)..(self.size + length)];
        target.copy_from_slice(bytes);
        self.size += length;
    }

    /// Polls to write bytes left.
    /// `write_func` receives left slice, and should return bytes count actually written.
    pub fn poll_write<E>(
        &mut self,
        write_func: impl FnOnce(&[u8]) -> Result<usize, E>,
    ) -> Result<usize, E> {
        if self.is_completed() {
            return Ok(0);
        }

        let left_bytes = &self.buffer[(self.written)..(self.size)];
        let written_bytes = write_func(left_bytes)?;
        self.written += written_bytes;

        Ok(written_bytes)
    }
}
