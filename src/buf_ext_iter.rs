use crate::buf_ext::BufExt;

pub struct BufExtIter<'a, 'b> {
    buf_ext: &'a mut BufExt<'b>,
    next_pos: usize,
}
impl<'a, 'b> BufExtIter<'a, 'b> {
    pub fn new(buf_ext: &'a mut BufExt<'b>) -> Self {
        Self {
            buf_ext,
            next_pos: 0,
        }
    }
}
impl<'a, 'b> Iterator for BufExtIter<'a, 'b> {
    type Item = std::io::Result<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next_pos == self.buf_ext.len() {
            match self.buf_ext.extend() {
                Ok(sz) => {
                    if sz == 0 {
                        None //End of buffer
                    } else {
                        debug_assert!(self.next_pos < self.buf_ext.len(), "Abnormal position");

                        let old_val = self.next_pos;
                        self.next_pos += 1;
                        Some(Ok(self.buf_ext.at(old_val)))
                    }
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            let old_val = self.next_pos;
            self.next_pos += 1;
            Some(Ok(self.buf_ext.at(old_val)))
        }
    }
}
