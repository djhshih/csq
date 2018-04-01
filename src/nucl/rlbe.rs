use std;
use std::io::Write;
use std::io::Result;

/// Nucleotide encoder: pack nucleotides into 2 bits each
pub struct Encoder<W> {
    w: W,
    threshold: u8,
    buffer: Vec<u8>,
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

impl<W: Write> Write for Encoder<W> {
    /// Pack content in buffer into own buffer.
    fn write(&mut self, x: &[u8]) -> Result<usize> {
        // unmatched prefix size
        let mut u = 0u8;
        // matched suffix size
        let mut m = 0u8;
        // starting position
        let mut s = 0;
        // current position
        let mut i = s;

        self.buffer.clear();
        self.buffer.reserve(x.len());

        while i < x.len() {
            // look for run of identical symbols
            let mut j = i + 1;
            while j < x.len() && j - i < (255 - self.threshold as usize) - 1 {
                if x[i] != x[j] {
                    break;
                }
                j += 1;
            }

            // matched length [i, j - 1]
            m = (j - i) as u8;
            println!("m = {}", m);

            // m < 2 does not achieve compression
            if m >= 3 {
                // write unmatched prefix
                if i > s {
                    // unmatched length [s, i-1]
                    u = (i - s) as u8;
                    // header symbol n = u - 1: copy n + 1 symbols verbatim
                    self.buffer.push(u - 1);
                    self.buffer.extend(&x[s..i]);
                }
                // write matched suffix
                // header symbol n = m + threshold - 3; repeat next symbol for n - threshold + 3 times
                self.buffer.push(m + self.threshold - 3);
                self.buffer.push(x[i]);

                // move onto next block
                i = j;
                s = j;
            } else {
                i += 1;

                // unmatched length [s, i-1]
                u = (i - s) as u8;

                // write unmatched prefix if it is too long
                if u - 1 == self.threshold - 1 {
                    // header symbol n = u - 1: copy n + 1 symbols verbatim
                    self.buffer.push(u - 1);
                    self.buffer.extend(&x[s..i]);
                    // move onto next block
                    s = i;
                }
            }
        }

        // write unmatched prefix
        if i > s {
            // unmatched length [s, i-1]
            u = (i - s) as u8;
            // header symbol n = u - 1: copy n + 1 symbols verbatim
            self.buffer.push(u - 1);
            self.buffer.extend(&x[s..i]);
        }

        println!("original len: {}, buffer len: {}", x.len(), self.buffer.len());
        try!(self.w.write_all(&self.buffer));

        let n = x.len();
        Ok(n)
    }
    
    fn flush(&mut self) -> Result<()> {
        self.w.flush()
    }
}

#[derive(Clone)]
/// Only two-bit encoding supported currently
pub struct EncoderBuilder {
}

impl EncoderBuilder {
    pub fn new() -> Self {
        EncoderBuilder {
        }
    }

    pub fn build<W: Write>(&self, w: W) -> Result<Encoder<W>> {
        let mut encoder = Encoder {
            w: w,
            //threshold: std::u8::MAX / 2 + 1,
            threshold: 220,
            buffer: Vec::new(),
        };
        // TODO write header?
        Ok(encoder)
    }
}

