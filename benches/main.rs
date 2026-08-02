use std::io::Read;

use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn buf_read_splitter(content_len: usize) {
    let nbr_of_iterations = 10_000_000 / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(separator.as_bytes()),
        Options::default(),
    );

    let mut buf = vec![0u8; 255];

    let mut nb_part_found = 0usize;

    while {
        let sz = reader.read(&mut buf).unwrap();
        if sz > 0 {
            true
        } else {
            nb_part_found += 1;
            match reader.next_part().unwrap() {
                //Pass to the next part of the buffer
                Some(_) => true, //There's a next part
                None => false,   //End of the stream
            }
        }
    } {}
    assert!(
        nb_part_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ( {nb_part_found} != {nbr_of_iterations} )"
    )
}

#[divan::bench(args = [10, 100, 1_000, 10_000, 100_000])]
fn minimal_count(content_len: usize) {
    let nbr_of_iterations = 10_000_000 / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut buf = vec![0u8; 255];
    let mut nb_sep_found = 0usize;

    let mut pos_found = 0usize;
    let separator_bytes = separator.as_bytes();

    while {
        let sz = stream.read(&mut buf).unwrap();
        if sz > 0 {
            for &b in &buf[..sz] {
                if b == separator_bytes[pos_found] {
                    pos_found += 1;
                    if pos_found == separator_bytes.len() {
                        nb_sep_found += 1;
                        pos_found = 0;
                    }
                } else if b == separator_bytes[0] {
                    pos_found = 1;
                    if pos_found == separator_bytes.len() {
                        nb_sep_found += 1;
                        pos_found = 0;
                    }
                } else {
                    if pos_found > 0 {
                        pos_found = 0;
                    }
                }
            }
            true
        } else {
            false
        }
    } {}

    assert!(
        nb_sep_found + 1 == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({nb_sep_found}!={nbr_of_iterations}) "
    )
}
pub struct StreamGenerator {
    content_len: usize,
    sep_content: Vec<u8>,
    actual_content: Actual,
    nbr_of_iterations: usize,
    current_nbr_of_iterations: usize,
}

enum Actual {
    Separator(usize),
    Content(usize),
}

impl StreamGenerator {
    pub fn new(content_len: usize, sep: &str, nbr_of_iterations: usize) -> Self {
        Self {
            // public //
            content_len,
            sep_content: Vec::from(sep.as_bytes()),
            nbr_of_iterations,
            // private //
            actual_content: Actual::Content(0),
            current_nbr_of_iterations: 0,
        }
    }
}

impl Read for StreamGenerator {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.nbr_of_iterations == self.current_nbr_of_iterations {
            return Ok(0);
        }

        for (i, c) in buf.iter_mut().enumerate() {
            *c = match self.actual_content {
                Actual::Separator(pos) => {
                    let to_ret = self.sep_content[pos];
                    // Determine what to do at the next iteration
                    self.actual_content = if pos + 1 == self.sep_content.len() {
                        Actual::Content(0)
                    } else {
                        Actual::Separator(pos + 1)
                    };
                    // Assign value
                    to_ret
                }
                Actual::Content(pos) => {
                    // Determine what to do at the next iteration
                    self.actual_content = if pos + 1 == self.content_len {
                        self.current_nbr_of_iterations += 1;
                        Actual::Separator(0)
                    } else {
                        Actual::Content(pos + 1)
                    };
                    // Assign value
                    b'X'
                }
            };
            if self.nbr_of_iterations == self.current_nbr_of_iterations {
                return Ok(i + 1);
            }
        }
        Ok(buf.len())
    }
}
