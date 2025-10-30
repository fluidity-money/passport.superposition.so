use std::io::{self, BufRead, BufReader, Read};

pub struct RecipeReader<R> {
    inner: BufReader<R>,
    buffer: Vec<u8>,
    pos: usize,
}

impl<R: Read> RecipeReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: BufReader::new(reader),
            buffer: Vec::new(),
            pos: 0,
        }
    }

    fn fill_buffer(&mut self) -> io::Result<()> {
        self.buffer.clear();
        self.pos = 0;

        loop {
            let mut line = String::new();
            let bytes_read = self.inner.read_line(&mut line)?;

            if bytes_read == 0 {
                // EOF
                return Ok(());
            }

            // Skip lines starting with '#'
            if !line.starts_with('#') {
                self.buffer.extend_from_slice(line.as_bytes());
                return Ok(());
            }
            // Otherwise, continue to next line
        }
    }
}

impl<R: Read> Read for RecipeReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.pos >= self.buffer.len() {
            self.fill_buffer()?;

            if self.buffer.is_empty() {
                return Ok(0); // EOF
            }
        }

        let remaining = &self.buffer[self.pos..];
        let to_copy = remaining.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&remaining[..to_copy]);
        self.pos += to_copy;

        Ok(to_copy)
    }
}