use core::fmt;
///
/// Options for BufReadSplitter
#[derive(Clone)]
pub struct Options {
    pub(crate) initiale_sz_to_match: usize,
    pub(crate) chunk_sz: usize,
    pub(crate) limit_read: Option<usize>,
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
            limit_read: None,
        }
    }
    ///
    /// Set the initiale size of the pattern to match
    /// This sets the initiale size of the extending buffer needed to read over the reading buffer
    pub fn set_reserve_sz_to_match(&mut self, sz: usize) -> &mut Self {
        self.initiale_sz_to_match = sz;
        self
    }
    ///
    /// Set the size of each extension of the extending buffer needed to read over the reading buffer
    pub fn set_extend_buffer_additionnal_sz(&mut self, sz: usize) -> &mut Self {
        self.chunk_sz = sz;
        self
    }
    ///
    /// Set a limit of bytes to read of a buffer part
    pub fn set_limit_read(&mut self, opt_sz: Option<usize>) -> &mut Self {
        self.limit_read = opt_sz;
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
