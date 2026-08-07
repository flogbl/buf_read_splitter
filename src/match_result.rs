use std::fmt;

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

impl fmt::Display for MatchResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchResult::Mismatch => write!(f, "Mismatch: No match found."),
            MatchResult::NeedNext => {
                write!(
                    f,
                    "NeedNext: Partial match, need to check the next character."
                )
            }
            MatchResult::Match(right, left) => {
                write!(
                    f,
                    "Match: Take {} to the right, {} to the left.",
                    right, left
                )
            }
        }
    }
}
