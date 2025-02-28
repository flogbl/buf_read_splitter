use crate::match_result::MatchResult;

pub trait Matcher {
    ///
    /// Called for each byte, return the state of the match
    fn sequel(&mut self, el_buf: u8, pos: usize) -> MatchResult;
    ///
    /// Called at the end of the stream, only if the previous byte (and so last byte) is in a NeedNext state.
    /// This function is useful for specific case.
    /// Note: `pos` is the position of the last readed byte (so the same as the previous one)
    fn sequel_eos(&mut self, _pos: usize) -> MatchResult {
        MatchResult::Mismatch
    }
}
