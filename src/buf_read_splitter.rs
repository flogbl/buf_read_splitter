use core::fmt;
use std::{cmp, io::Read};

///
/// BufReadSplitter : See unit test or lib documentations for an example
pub struct BufReadSplitter<'a> {
    reader: &'a mut dyn std::io::Read,
    buf_extend: Vec<u8>,
    options: Options,
    matched: bool,
}
///
/// Implementation
impl<'a> BufReadSplitter<'a> {
    pub fn new(reader: &'a mut dyn std::io::Read, options: Options) -> Self {
        Self {
            reader,
            buf_extend: Vec::with_capacity(options.initiale_sz_to_match),
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
    fn read_in_buf_extend_at_end(&mut self) -> std::io::Result<(usize,usize)> {
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
        Ok((start, sz_read))
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
impl<'a> Read for BufReadSplitter<'a> {
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
                        let (start, sz) = self.read_in_buf_extend_at_end()?;
                        if sz == 0 {
                            break 'loop_extend_buffer; // End of buffer
                        }
                        start
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
impl<'a> fmt::Debug for BufReadSplitter<'a> {
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
///
/// unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_none_to_match() {
        let input = "one to three four five six seven height nine ten".to_string();
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(&mut input_reader, Options::default());
        let mut buf = vec![0u8; 10];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if reader.matched() || sz == 0 {
                if reader.matched() == false {
                    // We enter here because of `sz=0` condition, so it's the end of the buffer
                    break;
                }
            }
        }
        assert_eq!(text, input, "Case 1");
    }

    #[test]
    fn test_common() {
        for i in 1..1000 {
            sub_test_common(i);
        }
    }
    fn sub_test_common(buf_ext: usize) {
        let input = "First<SEP><SEP>X<SEP>Second<SEP2>Y<SEP2>Small<>0<>Bigger<SEPARATOR_03>Till the end...<end>The last!".to_string();
        //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
        //                    10        20        30        40        50        60        70        80        90

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(
            &mut input_reader,
            Options::default()
                .set_initiale_sz_to_match(2)
                .set_chunk_sz(1)
                .clone(),
        );
        reader.set_array_to_match("<SEP>".as_bytes());
        let mut i = 0;
        let mut buf = vec![0u8; buf_ext];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if reader.matched() || sz == 0 {
                i += 1;

                match i {
                    1 => assert_eq!(text, "First", "Case 1"),
                    2 => assert_eq!(text, "", "Case 2"),
                    3 => assert_eq!(text, "X", "Case 3"),
                    4 => assert_eq!(text, "Second", "Case 4"),
                    5 => assert_eq!(text, "Y", "Case 5"),
                    6 => assert_eq!(text, "Small", "Case 6"),
                    7 => assert_eq!(text, "0", "Case 7"),
                    8 => assert_eq!(text, "Bigger", "Case 8"),
                    9 => assert_eq!(text, "Till the end...", "Case 9"),
                    10 => assert_eq!(text, "The last!", "Case 10"),
                    _ => assert_eq!(false, true, "Overflow"),
                }
                text.clear();

                if reader.matched() == false {
                    // We enter here because of `sz=0` condition, so it's the end of the buffer
                    break;
                }
            }

            match i {
                3 => reader.set_array_to_match("<SEP2>".as_bytes()),
                5 => reader.set_array_to_match("<>".as_bytes()),
                7 => reader.set_array_to_match("<SEPARATOR_03>".as_bytes()),
                8 => reader.set_array_to_match("<end>".as_bytes()),
                _ => {}
            }
        }
        assert_eq!(i, 10, "Missing iterations for {buf_ext}")
    }
    #[test]
    fn test_sep_first_pos() {
        for i in 1..1000 {
            sub_test_sep_first_pos(i);
        }
    }
    fn sub_test_sep_first_pos(buf_sz: usize) {
        let input = "<SEP>First<SEP>".to_string();

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(&mut input_reader, Options::default().clone());
        reader.set_array_to_match("<SEP>".as_bytes());
        let mut i = 0;

        let mut buf = vec![0u8; buf_sz];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if reader.matched() || sz == 0 {
                i += 1;

                match i {
                    1 => {
                        assert_eq!(text, "", "Case 1");
                    }
                    2 => {
                        assert_eq!(text, "First", "Case 2");
                    }
                    3 => {
                        assert_eq!(text, "", "Case 3");
                    }
                    _ => {
                        assert_eq!(false, true, "Overflow")
                    }
                }
                text.clear();

                if reader.matched() == false {
                    // We enter here because of `sz=0` condition, so it's the end of the buffer
                    break;
                }
            }
        }
        assert_eq!(i, 3, "Missing iterations for {buf_sz}")
    }
    
    #[test]
    fn test_sep_partial() {
        for i in 1..1000 {
            sub_test_sep_partial(i);
        }
    }
    fn sub_test_sep_partial(buf_sz: usize) {
        let input = "<SEP>First<S".to_string();

        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(&mut input_reader, Options::default().clone());
        reader.set_array_to_match("<SEP>".as_bytes());
        let mut i = 0;

        let mut buf = vec![0u8; buf_sz];
        let mut text = String::new();
        loop {
            let sz = reader.read(&mut buf).unwrap();
            let str = String::from_utf8_lossy(&buf[..sz]);

            text.push_str(&str);

            if reader.matched() || sz == 0 {
                i += 1;

                match i {
                    1 => assert_eq!(text, "", "Case 1"),
                    2 => assert_eq!(text, "First<S", "Case 2"),
                    _ => assert_eq!(false, true, "Overflow"),
                }
                text.clear();

                if reader.matched() == false {
                    // We enter here because of `sz=0` condition, so it's the end of the buffer
                    break;
                }
            }
        }
        assert_eq!(i, 2, "Missing iterations for {buf_sz}")
    }
}
