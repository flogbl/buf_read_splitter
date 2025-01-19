use core::fmt;
use std::{cmp, io::Read};

use crate::fifo::Fifo;

/// To manage a generic Result
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct BufReadSplitter<'a> {
    reader: &'a mut dyn std::io::Read,
    buffer_sz: usize,
    fifo: Fifo,
    sep: Vec<u8>, //TODO: Is Vec the most adapted ?
    found_sz: usize,
    found_nextpos: usize,
}

impl<'a> BufReadSplitter<'a> {
    ///
    /// Create a new BufReadSpitter
    /// The `buffer_sz` parameter is the reserved size used to communicate between input buffer and output buffer
    /// (but the output buffer will of course be fill as much as possible)
    pub fn new(buffer_sz: usize, reader: &'a mut dyn std::io::Read, sep: &[u8]) -> Self {
        Self {
            reader,
            buffer_sz,
            fifo: Fifo::new(Self::fifo_size_needed(buffer_sz, sep.len())).unwrap(), //TODO: return an error instead of unwrap
            sep: sep.to_vec(),
            found_sz: 0,
            found_nextpos: 0,
        }
    }
    ///
    /// Calculate the needed size of the buffer
    /// The size of our internal buffer have to be the size in
    /// parameter + the size of the searched data. This is because, to
    /// know the part of the buffer we can send (so without the
    /// hypothétique founded part), we need this extra-size.
    ///
    /// Example where :
    ///  - input buffer size = 3
    ///  - output buffer size = 3
    ///  - "SSS" is the searched data
    ///  - so internal buffer size 3 + size of the searched data = 6
    ///
    /// - first read :
    ///      a b c d e f <-- none found
    ///
    /// - second read :
    ///      d e f g h S <-- 1 position found at pos 5, does the next part match all ?
    ///                ^     In all case this one have to NOT be send in the output buffer
    /// - third read :
    ///      g h S S S i <-- yes, it matches ! so only the first 2 characters have to be send to the output buffer
    ///          ^ ^ ^
    fn fifo_size_needed(buffer_sz: usize, sep_sz: usize) -> usize {
        buffer_sz + sep_sz
    }
    ///
    /// Read the buffer and update match
    fn read_and_upd_match(&mut self) -> Result<()> {
        let (slice_1, slice_2) = self.fifo.get_available_mut();

        let (sz_read_1, sz_read_2) = Self::read_2slices(self.reader, slice_1, slice_2)?;

        if self.sep.len() != self.found_sz {
            if let Some(pos_last) =
                Self::sequel_search(&slice_1[..sz_read_1], &self.sep, &mut self.found_sz)
            {
                // All match in slice1, set the position
                self.found_nextpos = self.fifo.len() + pos_last + 1;
            } else {
                if sz_read_2 != 0 {
                    if let Some(pos_last) =
                        Self::sequel_search(&slice_2[..sz_read_2], &self.sep, &mut self.found_sz)
                    {
                        // All match in slice2
                        self.found_nextpos = self.fifo.len() + slice_1.len() + pos_last + 1;
                    }
                }
            }
        }

        self.fifo.commit(sz_read_1 + sz_read_2);

        Ok(())
    }
    ///
    /// Read the buffer
    fn read_2slices(
        reader: &mut dyn std::io::Read,
        slice_1: &mut [u8],
        slice_2: &mut [u8],
    ) -> Result<(usize, usize)> {
        let sz_read_1 = reader.read(slice_1)?;
        let sz_read_2 = {
            if sz_read_1 == slice_1.len() {
                reader.read(slice_2)?
            } else {
                0usize
            }
        };
        Ok((sz_read_1, sz_read_2))
    }
    ///
    /// Complete the current match, returning the last position of the match if found, or None otherwise
    fn sequel_search(slice: &[u8], searched: &Vec<u8>, sz_found: &mut usize) -> Option<usize> {
        for (i, el) in slice.into_iter().enumerate() {
            if *el == searched[*sz_found] {
                *sz_found += 1;
                if *sz_found == searched.len() {
                    return Some(i);
                }
            } else {
                *sz_found = 0;
            }
        }
        None
    }
    ///
    /// Pop the buffer in the output buffer part in parameter, returning size poped
    fn pop_buffer(&mut self, data: &mut [u8]) -> usize {
        let (slice_1, slice_2) = self.fifo.pop(data.len());
        data[..slice_1.len()].copy_from_slice(slice_1);
        let mut sz = slice_1.len();
        if slice_2.len() > 0 {
            data[slice_1.len()..slice_1.len() + slice_2.len()].copy_from_slice(slice_2);
            sz += slice_2.len();
        }
        return sz;
    }
    ///
    /// Pass to the next splitted buffer, unchanging the separator
    pub fn next(&mut self) -> Option<Result<()>> {
        self.next_split_on(None)
    }
    ///
    /// Pass to next splitted buffer, changing the separator or None if unchanged
    pub fn next_split_on(&mut self, opt_new_sep: Option<&[u8]>) -> Option<Result<()>> {
        // We can call "next" in severals case :
        //  1- It's the end of the buffer
        //  2- The "wanted" pattern has been reached
        //  3- Next is call but the caller didn't wait the end
        // Case 3- Next is call but the caller didn't wait the end => Go to the end of the buffer
        while self.found_sz != self.sep.len() && self.fifo.len() != 0 {
            if let Err(err) = self.read_and_upd_match() {
                return Some(Err(err));
            }
        }

        // Indicate that the buffer needs to refresh his search datas
        // It's the case if :
        //  - Wanted has been changed
        //  - We are in the case of an "after a match", so the search stopped at this first match
        let mut buffer_need_search_update = false;

        // Case 2- The "wanted" pattern has been reached
        if self.found_sz == self.sep.len() {
            // Remove the wanted part if there's one
            self.fifo.pop(self.found_nextpos);
            buffer_need_search_update = true;
        }
        // Case 1- It's the end of the buffer
        else if self.fifo.len() == 0 {
            // The end of the buffer
            return None;
        }

        // Change the "wanted" pattern if asking for
        if let Some(new_sep) = opt_new_sep {
            self.sep.clear();
            self.sep.extend_from_slice(new_sep);
            let needed_capacity = Self::fifo_size_needed(self.buffer_sz, self.sep.len());
            // Review capacity only if no data will be removed !
            if self.fifo.len() < needed_capacity {
                if let Err(err) = self.fifo.set_capacity(needed_capacity) {
                    return Some(Err(err));
                }
            }

            buffer_need_search_update = true;
        }

        // Analysing all the current buffer if needed
        if buffer_need_search_update {
            self.found_sz = 0;
            let (slice_1, slice_2) = self.fifo.get_feeded_mut();
            if let Some(pos_last) = Self::sequel_search(&slice_1, &self.sep, &mut self.found_sz) {
                // All match in slice1, set the position
                self.found_nextpos = pos_last + 1;
            } else {
                if let Some(pos_last) = Self::sequel_search(&slice_2, &self.sep, &mut self.found_sz)
                {
                    // All match in slice2
                    self.found_nextpos = slice_1.len() + pos_last + 1;
                }
            }
        }

        if self.fifo.len() > 0 {
            println!("({}) Some {}", line!(), self.fifo.len());
            return Some(Ok(()));
        } else {
            println!("({}) None", line!());
            return None;
        }
    }
}
///
/// Facilities function to convert an dyn Error to an io one
fn err_to_io(err: Box<dyn std::error::Error>) -> std::io::Error {
    if let Ok(err) = err.downcast::<std::io::Error>() {
        if let Ok(err) = err.downcast::<std::io::Error>() {
            return err;
        }
    }
    std::io::Error::new(std::io::ErrorKind::Other, "Unmanaged error")
}
///
/// Buffer reader implementation
impl<'a> Read for BufReadSplitter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Initialization
        let mut sz_send = 0;

        // We have to loop because the output buffer can be larger than the input one
        while buf.len() != sz_send {
            // Feed available space of the buffer
            // - The output buffer can be smaller than our, so we have to set this
            //   condition on the feed of our buffer
            // - The internal buffer has in fact a length of buffer+wanted to avoid sending the "wanted" part when found
            if let Err(err) = self.read_and_upd_match() {
                return Err(err_to_io(err));
            };

            // Determine the size to send according to the case
            let len = {
                if self.sep.len() != self.found_sz || self.sep.len() == 0 {
                    //Case : "wanted" not at all or not totally found
                    //  So : simply use the size of the output buffer or of our buffer
                    cmp::min(buf.len() - sz_send, self.buffer_sz)
                } else {
                    //Case : "wanted" found and it is not empty (if empty we are searching nothing)
                    let len = cmp::min(buf.len() - sz_send, self.found_nextpos - self.found_sz);
                    // We have to maintain the position found because it will not stop the buffer in this iteration
                    self.found_nextpos -= len;
                    len
                }
            };

            // Feed the output buffer
            let sz_poped = self.pop_buffer(&mut buf[sz_send..sz_send + len]);
            sz_send += sz_poped;
            if sz_poped == 0 {
                // End of this stream iteration
                return Ok(sz_send);
            }
        }
        return Ok(sz_send);
    }
}

impl<'a> fmt::Debug for BufReadSplitter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct BufReadSplitterDEBUG {
            buffer_sz: usize,
            fifo: String,
            sep: Vec<u8>,
            found_sz: usize,
            found_nextpos: usize,
        }

        // per Chayim Friedman’s suggestion
        fmt::Debug::fmt(
            &BufReadSplitterDEBUG {
                buffer_sz: self.buffer_sz,
                fifo: format!("{:?}", self.fifo),
                sep: self.sep.clone(),
                found_sz: self.found_sz,
                found_nextpos: self.found_nextpos,
            },
            f,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common() {
        for i in 1..100 {
            for j in 1..100 {
                sub_test_common(i, j);
            }
        }
    }
    fn sub_test_common(buf_split: usize, buf_ext: usize) {
        let input = "First<SEP><SEP>X<SEP>Second<SEP2>Y<SEP2>Small<>0<>Bigger<SEPARATOR_03>Till the end...<end>The last!".to_string();
        //           123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789 123456789
        //                    10        20        30        40        50        60        70        80        90
        let mut input_reader = input.as_bytes();
        let mut reader = BufReadSplitter::new(buf_split, &mut input_reader, "<SEP>".as_bytes());
        let mut i = 0;
        loop {
            i += 1;

            let mut buf = vec![0u8; buf_ext];
            let mut text = String::new();

            loop {
                let sz = reader.read(&mut buf).unwrap();
                if sz > 0 {
                    let str = String::from_utf8_lossy(&buf[..sz]);
                    text.push_str(&str);
                    println!("{sz} '{text}'");
                } else {
                    // End of buffer
                    match i {
                        1 => {
                            assert_eq!(text, "First", "Case 1");
                        }
                        2 => {
                            assert_eq!(text, "", "Case 2");
                        }
                        3 => {
                            assert_eq!(text, "X", "Case 3");
                        }
                        4 => {
                            assert_eq!(text, "Second", "Case 4");
                        }
                        5 => {
                            assert_eq!(text, "Y", "Case 5");
                        }
                        6 => {
                            assert_eq!(text, "Small", "Case 6");
                        }
                        7 => {
                            assert_eq!(text, "0", "Case 7");
                        }
                        8 => {
                            assert_eq!(text, "Bigger", "Case 8");
                        }
                        9 => {
                            assert_eq!(text, "Till the end...", "Case 9");
                        }
                        10 => {
                            assert_eq!(text, "The last!", "Case 10");
                        }
                        _ => {
                            assert_eq!(false, true, "Overflow")
                        }
                    }
                    break;
                }
            }

            match reader.next_split_on({
                match i {
                    3 => Some("<SEP2>".as_bytes()),
                    5 => Some("<>".as_bytes()),
                    7 => Some("<SEPARATOR_03>".as_bytes()),
                    8 => Some("<end>".as_bytes()),
                    _ => None,
                }
            }) {
                Some(Ok(_)) => {}
                Some(Err(err)) => panic!("Error : {err}"),
                None => break,
            }
        }
        assert_eq!(i, 10, "Missing iterations for {buf_split}/{buf_ext}")
    }
}
