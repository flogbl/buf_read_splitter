use crate::match_result::MatchResult;

pub trait Matcher {
    fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult;
}
