use std::io::Read;

use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn buf_read_splitter(content_len: usize) {
    let nbr_of_iterations = 100_000_000 / content_len;
    let mut stream = StreamGenerator::new(content_len, "<sep>", nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(b"<SEP>"),
        Options::default(),
    );

    let mut buf = vec![0u8; 255];

    while {
        let sz = reader.read(&mut buf).unwrap();
        if sz > 0 {
            let _to_str = String::from_utf8_lossy(&buf[..sz]);
            true
        } else {
            match reader.next_part().unwrap() {
                //Pass to the next part of the buffer
                Some(_) => true, //There's a next part
                None => false,   //End of the stream
            }
        }
    } {}
}

pub struct StreamGenerator {
    content_len: usize,
    sep_content: Vec<u8>,
    content_pos: usize,
    opt_sep_pos: Option<usize>,
    nbr_of_iterations: usize,
}

impl StreamGenerator {
    pub fn new(content_len: usize, sep: &str, nbr_of_iterations: usize) -> Self {
        Self {
            // public //
            content_len,
            sep_content: Vec::from(sep.as_bytes()),
            nbr_of_iterations,
            // private //
            content_pos: 0,
            opt_sep_pos: None,
        }
    }
}

impl Read for StreamGenerator {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut sz = 0;
        for c in buf.iter_mut() {
            if let Some(sep_pos) = self.opt_sep_pos {
                if sep_pos < self.sep_content.len() {
                    *c = self.sep_content[sep_pos];
                    self.opt_sep_pos = Some(sep_pos + 1);
                    sz += 1;
                } else {
                    self.opt_sep_pos = None;
                }
            };
            if self.opt_sep_pos.is_none() {
                if self.content_pos < self.content_len {
                    *c = b'X';
                    self.content_pos += 1;
                    sz += 1;
                } else {
                    if self.nbr_of_iterations > 0 {
                        self.nbr_of_iterations -= 1;
                    } else {
                        return Ok(0); // <-- End of stream
                    }

                    self.content_pos = 0;
                    self.opt_sep_pos = Some(0);
                }
            }
        }
        Ok(sz)
    }
}
