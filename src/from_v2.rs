use core::fmt;
use std::{cmp, io::Read};

///
/// BufReadSplitter : See unit test or lib documentations for an example
pub struct BufReadSplitter__new<'a> {
    reader: &'a mut dyn std::io::Read,
    buf_extend: Vec<u8>,
    options: Options,
    matched: bool,
}
///
/// Implementation
impl<'a> BufReadSplitter__new<'a> {
    pub fn new(reader: &'a mut dyn std::io::Read, options: Options) -> Self {
        Self {
            reader,
            buf_extend: Vec::new(),
            options,
            matched: false,
        }
    }
    ///
    /// Return true if the buffer has stopped because it stop at the slice to match
    pub fn matched(&self) -> bool {
        self.matched
    }
    ///
    /// Change the match pattern
    pub fn set_array_to_match(&mut self, to_match: &[u8]) {
        //TODO: Optimize
        if self.options.to_match.capacity() < to_match.len() {
            let diff = to_match.len() - self.options.to_match.capacity();
            self.options.to_match.reserve(diff);
        }
        unsafe { self.options.to_match.set_len(to_match.len()) };
        self.options.to_match.copy_from_slice(to_match);
    }
    ///
    /// Unstack the buffer extender
    fn pop_buf_extend(v: &mut Vec<u8>, buf: &mut [u8]) -> usize {
        let sz = cmp::min(v.len(), buf.len());
        buf[..sz].copy_from_slice(&v[..sz]);
        v.drain(..sz);
        sz
    }
    ///
    /// Read the buffer pushing in the buffer extender
    /// Return the position where news datas from the "read" starts
    fn read_in_buf_extend_at_end(&mut self) -> std::io::Result<usize> {
        if self.buf_extend.capacity() < self.buf_extend.len() + self.options.chunk_sz {
            self.buf_extend.reserve(self.options.chunk_sz);
        }

        let start = self.buf_extend.len();

        unsafe {
            self.buf_extend
                .set_len(self.buf_extend.len() + self.options.chunk_sz);
        }

        let sz_read = self.reader.read(&mut self.buf_extend[start..])?;

        if start + sz_read < self.buf_extend.len() {
            unsafe {
                self.buf_extend.set_len(start + sz_read);
            }
        }

        // Return the position of the readed part
        Ok(start)
    }
    ///
    /// Sequel of the search
    fn sequel(&self, el_buf: &u8, pos: usize) -> MatchResult {
        if cfg!(debug_assertions) {
            if pos > self.options.to_match.len() {
                panic!(
                    "Line {} : Unexpected overflow : {} > {}",
                    line!(),
                    pos,
                    self.options.to_match.len()
                )
            }
        }
        if pos == self.options.to_match.len()
            || *el_buf != unsafe { *self.options.to_match.get_unchecked(pos) }
        {
            MatchResult::Mismatch
        } else {
            if self.options.to_match.len() == pos + 1 {
                MatchResult::Match
            } else {
                MatchResult::NeedNext
            }
        }
    }
}
///
/// Implementation
impl<'a> Read for BufReadSplitter__new<'a> {
    ///
    /// Read until the begin of a match`match()==true`, or end of the buffer (returned size = 0)
    /// match()==true and returned size = 0 is not the end of the buffer
    /// The end of the buffer is when : match()==false and returned size=0
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.matched = false;

        let mut sz_output = 0;
        if self.buf_extend.len() > 0 {
            sz_output = Self::pop_buf_extend(&mut self.buf_extend, buf)
        }
        if sz_output < buf.len() {
            sz_output += self.reader.read(&mut buf[sz_output..])?;
        }
        let mut sz_found = 0;
        for (i, el) in buf[..sz_output].iter().enumerate() {
            match self.sequel(el, sz_found) {
                MatchResult::NeedNext => {
                    sz_found += 1;
                }
                MatchResult::Mismatch => {
                    sz_found = 0;
                }
                MatchResult::Match => {
                    if i + 1 < buf.len() {
                        // Save the part next to the matched part
                        self.buf_extend
                            .splice(0..0, buf[i + 1..sz_output].iter().copied());
                    }
                    sz_found += 1;
                    // The buffer ending exactly before the matched part
                    // (sz_output is a size, so we have to add 1)
                    sz_output = i + 1 - sz_found;
                    self.matched = true;
                    return Ok(sz_output);
                }
            }
        }
        // In fact it's : sz_found>0 AND NeedNext, but we can bypass NeedNext test because Match raise a `return`
        if sz_found > 0 {
            'loop_extend_buffer: loop {
                let search_from = {
                    // Extend the internal buffer if there's no sufficient size to determine if there's a match
                    if self.buf_extend.len() < sz_found {
                        self.read_in_buf_extend_at_end()?
                    } else {
                        0
                    }
                };
                // Scan of the last read to continue to determine the match/no match
                for (i, el) in self.buf_extend[search_from..].iter().enumerate() {
                    match self.sequel(el, sz_found) {
                        MatchResult::NeedNext => {
                            sz_found += 1;
                        }
                        MatchResult::Mismatch => {
                            break 'loop_extend_buffer;
                        }
                        MatchResult::Match => {
                            sz_found += 1;

                            // The size to return had to exclude the matched part
                            // So it's, in a point of view [buffer]+[buffer extend[last read]] :
                            //    buf.len() <-- Start position of <buffer extend>
                            //    search_from <-- Start position of <last read>
                            //    i <-- position of the last byte that validate the match
                            //    so we have to subtract the sz_found to have the position of the latest byte to return
                            //    and because it's a position and not a length, we have to add 1
                            sz_output = buf.len() + search_from + i + 1 - sz_found;

                            // Remove the matched part because we have to use the remain part so it will feed the next <read>
                            self.buf_extend.drain(..search_from + i + 1);

                            // Matched
                            self.matched = true;
                            return Ok(sz_output);
                        }
                    }
                }
            }
        }
        // No match
        Ok(sz_output)
    }
}
///
/// For debug
impl<'a> fmt::Debug for BufReadSplitter__new<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "buf_extend={:?} option=[{:?}] matched={:?}",
            self.buf_extend, self.options, self.matched
        )
    }
}

///
/// Internal use
enum MatchResult {
    Mismatch,
    NeedNext,
    Match,
}
///
/// Options for BufReadSplitter
///
#[derive(Clone)]
pub struct Options {
    initiale_sz_to_match: usize,
    chunk_sz: usize,
    to_match: Vec<u8>,
}
///
/// Options implementations
impl Options {
    ///
    /// Options by defaults
    pub fn default() -> Self {
        let approximate_pattern_sz = 10;
        Self {
            initiale_sz_to_match: approximate_pattern_sz,
            chunk_sz: 5,
            to_match: Vec::with_capacity(approximate_pattern_sz),
        }
    }
    ///
    /// The pattern to found
    pub fn set_array_to_match(&mut self, to_match: &[u8]) {
        if self.to_match.capacity() < to_match.len() {
            let diff = to_match.len() - self.to_match.capacity();
            self.to_match.reserve(diff);
        }
        unsafe { self.to_match.set_len(to_match.len()) };
        self.to_match.copy_from_slice(to_match);
    }
    ///
    /// Set the initiale size of the pattern to match
    /// This sets the initiale size of the extending buffer needed to read over the reading buffer
    pub fn set_initiale_sz_to_match(&mut self, sz: usize) -> &mut Self {
        self.initiale_sz_to_match = sz;
        self
    }
    ///
    /// Set the size of each extension of the extending buffer needed to read over the reading buffer
    pub fn set_chunk_sz(&mut self, sz: usize) -> &mut Self {
        self.chunk_sz = sz;
        self
    }
}
///
/// Debug
impl<'a> fmt::Debug for Options {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "approximate_pattern_sz={}, chunk_sz={},",
            self.initiale_sz_to_match, self.chunk_sz
        )
    }
}
