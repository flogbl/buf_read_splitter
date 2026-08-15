use crate::MatchResult;
use crate::Matcher;

pub struct SimpleMatcher {
    to_match: Vec<u8>,
}
impl SimpleMatcher {
    pub fn new(to_match: &[u8]) -> Self {
        Self {
            to_match: Vec::from(to_match),
        }
    }
}
impl Matcher for SimpleMatcher {
    fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult {
        if cfg!(debug_assertions) {
            if pos > self.to_match.len() {
                panic!(
                    "Line {} : Unexpected overflow : {} > {}",
                    line!(),
                    pos,
                    self.to_match.len()
                );
            }
        }
        if pos == self.to_match.len() || el_buf != *self.to_match.get(pos).unwrap() {
            MatchResult::Mismatch
        } else if self.to_match.len() == pos + 1 {
            MatchResult::Match(0, 0)
        } else {
            MatchResult::NeedNext
        }
    }
}
