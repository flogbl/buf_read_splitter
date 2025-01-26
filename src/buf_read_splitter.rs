use core::fmt;
use std::{cmp, io::Read};

use crate::from_v2;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct BufReadSplitter<'a> {
    bfs: from_v2::BufReadSplitter__new<'a>,
    need_next: bool,
    last_sz_read: usize,
    last_matched: bool,
}

impl<'a> BufReadSplitter<'a> {
    ///
    /// Create a new BufReadSpitter
    /// The `buffer_sz` parameter is the reserved size used to communicate between input buffer and output buffer
    /// (but the output buffer will of course be fill as much as possible)
    pub fn new(buffer_sz: usize, reader: &'a mut dyn std::io::Read, sep: &[u8]) -> Self {
        let mut bfs = from_v2::BufReadSplitter__new::<'a>::new(
            reader,
            from_v2::Options::default()
                .set_initiale_sz_to_match(buffer_sz)
                .clone(),
        );
        bfs.set_array_to_match(sep);
        Self {
            bfs,
            need_next: false,
            last_sz_read: 0,
            last_matched: false,
        }
    }
    ///
    /// Pass to the next splitted buffer, unchanging the separator
    pub fn next(&mut self) -> Option<Result<()>> {
        self.next_split_on(None)
    }
    ///
    /// Pass to next splitted buffer, changing the separator or None if unchanged
    pub fn next_split_on(&mut self, opt_new_sep: Option<&[u8]>) -> Option<Result<()>> {
        if self.need_next == false {
            // Have to read until end of buffer or separator
            let mut buf = [0u8; 100];
            while self.need_next == false {
                match self.read(&mut buf) {
                    Ok(o) => {}
                    Err(err) => return Some(Err(err.into())),
                }
            }
        }

        self.need_next = false;

        if let Some(sep) = opt_new_sep {
            self.bfs.set_array_to_match(sep);
        }

        if self.last_sz_read == 0 && self.last_matched == false {
            None
        } else {
            Some(Ok(()))
        }
    }
}
///
/// Buffer reader implementation
impl<'a> Read for BufReadSplitter<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Need a call to "next()" to continue
        if self.need_next == false {
            self.last_sz_read = self.bfs.read(buf)?;
            self.need_next = self.bfs.matched() || self.last_sz_read == 0;
            self.last_matched = self.bfs.matched();

            Ok(self.last_sz_read)
        } else {
            Ok(0)
        }
    }
}

///
/// For debug
impl<'a> fmt::Debug for BufReadSplitter<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "buf_extend={:?}", self.bfs)
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
                    //println!("{sz} '{text}'");
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
