///
/// Used by a matcher, returning the result of the search of one character
#[derive(Clone)]
pub enum MatchResult {
    ///
    /// Not matched
    Mismatch,
    ///
    /// This char match the position, need a next one to define if match or not
    NeedNext,
    ///
    /// Matched. Arguments are : ( size to take to right , size_to_take_to_the_left )
    Match(usize, usize),
}
