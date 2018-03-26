use std::io::Write;
use std::io::Result;

use illumina::qual;

/// Nucleotide encoder: pack nucleotides into 2 bits each
pub struct Encoder<W> {
    w: W,
    limit: usize,
    buffer: Vec<u8>,
    offset: u8,
    byte: u8,
}

impl<W: Write> Encoder<W> {
    /// Immutable writer reference.
    pub fn writer(&self) -> &W {
        &self.w
    }

    pub fn finish(mut self) -> (W, Result<()>) {
        (self.w, Ok(()))
    }
}

// high nibble first: 1st base in highest 2-bit of the 1st byte
// 00 A
// 01 C
// 10 G
// 11 T
impl<W: Write> Write for Encoder<W> {
    /// Pack content in buffer into own buffer.
    fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        let mut n = 0;
        self.buffer.reserve(buffer.len() / 2 + 1);
        self.buffer.clear();
        for q in buffer {
            self.byte |= qual::pack(*q) << (4 - 4 * self.offset);
            self.offset += 1;
            if self.offset == 2 {
                // write packed byte to buffer
                self.buffer.push(self.byte);

                self.offset = 0;
                self.byte = 0;
            }
            n += 1;
        }
        try!(self.w.write_all(&self.buffer));
        Ok(n)
    }
    
    fn flush(&mut self) -> Result<()> {
        if self.offset != 0 {
            // write the partial last byte
            try!(self.w.write(&[self.byte]));
        }
        self.w.flush()
    }
}

#[derive(Clone)]
/// Only two-bit encoding supported currently
pub struct EncoderBuilder {
    bits: u8,
}

impl EncoderBuilder {
    pub fn new() -> Self {
        EncoderBuilder {
            bits: 2,
        }
    }

    pub fn build<W: Write>(&self, w: W) -> Result<Encoder<W>> {
        let mut encoder = Encoder {
            w: w,
            limit: 0,
            buffer: Vec::new(),
            offset: 0,
            byte: 0
        };
        // TODO write header?
        Ok(encoder)
    }
}

