use core::fmt;
use std::cmp;

use crate::buf_ext_iter::BufExtIter;

pub struct BufExt<'a> {
    reader: &'a mut dyn std::io::Read,
    ext: Vec<u8>,
    sz_read_ext: usize,
}
impl<'a> BufExt<'a> {
    ///
    /// Create a new buffer extender
    pub fn new(
        reader: &'a mut dyn std::io::Read,
        initiale_capacity: usize,
        sz_read_ext: usize,
    ) -> Self {
        Self {
            reader,
            ext: Vec::with_capacity(initiale_capacity),
            sz_read_ext,
        }
    }
    ///
    /// Extend the internal buffer by reading the input buffer
    pub fn extend(&mut self) -> std::io::Result<usize> {
        // Extends if needed
        if self.ext.capacity() < self.ext.len() + self.sz_read_ext {
            self.ext.reserve(self.sz_read_ext);
        }

        let start = self.ext.len();

        //TODO: Read from a buffer into a vector --> Optimizable?
        self.ext.resize(start + self.sz_read_ext, 0);
        let sz_read = self.reader.read(&mut self.ext[start..])?;
        if start + sz_read < self.ext.len() {
            // Not all the buffer has been filling, so resize
            self.ext.resize(start + sz_read, 0);
        }

        // Return the position of the readed part
        Ok(sz_read)
    }
    ///
    /// Unstack the buffer extender
    pub fn pop_buf_into(&mut self, buf: &mut [u8]) -> usize {
        let sz = cmp::min(self.ext.len(), buf.len());
        buf[..sz].copy_from_slice(&self.ext[..sz]);
        self.ext.drain(..sz);
        sz
    }
    ///
    /// Remove a certain number of elements at the begin of the extend buffer
    pub fn pop_buf(&mut self, nbr: usize) {
        self.ext.drain(..nbr);
    }
    ///
    /// Read the input buffer
    pub fn read_direct(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
    ///
    ///
    pub fn push_at_begin(&mut self, buf: &[u8]) {
        self.ext.splice(0..0, buf.iter().copied());
    }
    ///
    /// Actual length of the internal buffer
    pub fn len(&self) -> usize {
        self.ext.len()
    }
    ///
    /// Get a value
    pub fn at(&self, pos: usize) -> u8 {
        self.ext[pos]
    }
    ///
    /// To iterate
    pub fn iter_grow<'b>(&'b mut self) -> BufExtIter<'b, 'a> {
        BufExtIter::new(self)
    }
    pub fn cloned_internal_vec(&self) -> Vec<u8> {
        self.ext.clone()
    }
}

///
/// For debugging
impl<'a> fmt::Debug for BufExt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "buf_extend={:?} sz_read_ext=[{:?}]",
            self.ext, self.sz_read_ext
        )
    }
}
