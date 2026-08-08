/// Terminology: [----content----][-skipped-][---next_content---]
pub struct PosSizeHelper {
    next_content_pos: usize,
    skipped_pos: usize,
}

impl PosSizeHelper {
    ///
    ///
    pub fn from_match(
        take_left: usize,
        take_right: usize,
        sz_matched: usize,
        end_of_sep_pos: usize,
    ) -> Self {
        let next_content_pos = end_of_sep_pos + 1 - take_right;
        Self {
            next_content_pos,
            skipped_pos: next_content_pos + take_left + take_right - sz_matched,
        }
    }
    ///
    ///
    pub fn from_relative(pos_sz: &PosSizeHelper, relative_pos: usize) -> Self {
        Self {
            next_content_pos: {
                if pos_sz.next_content_pos > relative_pos {
                    pos_sz.next_content_pos - relative_pos
                } else {
                    0
                }
            },
            skipped_pos: {
                if pos_sz.skipped_pos > relative_pos {
                    pos_sz.skipped_pos - relative_pos
                } else {
                    0
                }
            },
        }
    }
    ///
    ///
    pub fn next_content_pos(&self) -> usize {
        self.next_content_pos
    }
    ///
    ///
    pub fn skipped_pos(&self) -> usize {
        self.skipped_pos
    }
}
