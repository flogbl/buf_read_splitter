use std::io::Read;

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
