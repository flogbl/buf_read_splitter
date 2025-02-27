///
/// Used by a matcher, returning the result of the search of one character
pub enum MatchResult {
    ///
    /// Not matched
    Mismatch,
    ///
    /// This char match, but need a next one
    NeedNext,
    ///
    /// Matched
    Match(usize, usize),
}
