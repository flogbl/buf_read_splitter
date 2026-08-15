use crate::MatchResult;
use crate::Matcher;

pub struct AllEndOfLineMatcher {
    prev_char: u8,
}
impl AllEndOfLineMatcher {
    pub fn new() -> Self {
        Self { prev_char: 0 }
    }
}
impl Matcher for AllEndOfLineMatcher {
    // This function is called at each byte read
    //   `el_buf` contains the value of the byte
    //   `pos` contains the position matched
    fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult {
        if pos == 0 {
            if el_buf == b'\r' || el_buf == b'\n' {
                self.prev_char = el_buf;
                MatchResult::NeedNext
            } else {
                MatchResult::Mismatch
            }
        } else if pos == 1 {
            if el_buf == b'\n' && self.prev_char == b'\r' {
                MatchResult::Match(0, 0) //We are on \r\n
            } else {
                MatchResult::Match(0, 1) //We have to ignore the last byte since it's not a part of the end of line pattern
            }
        } else {
            panic!("We can't reach this code since we just manage 2 positions")
        }
    }

    // This function is called at the end of the buffer, useful to manage partial cases
    fn sequel_eos(&mut self, pos: usize) -> MatchResult {
        if pos == 0 {
            MatchResult::Match(0, 0) //Here the last char is \r or \n, at position 0
        } else {
            panic!("We can't reach this code since we just manage 2 positions")
        }
    }
}
