use core::fmt;
use std::{cmp, ops::Range};

use crate::BufGrowingExtIter;

pub struct BufExt<'a> {
    reader: &'a mut dyn std::io::Read, // The stream to read
    ext: Vec<u8>,                      // Bytes in memory
    sz_read_ext: usize,                // Size of the grow for each read
    eos_reached: bool,                 // Indicate that End of stream was reached
    inner_start: usize, // Virtual start to have a constant speed whatever the size of the buffer is
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
            eos_reached: false,
            inner_start: 0,
        }
    }
    ///
    /// Extend the internal buffer by reading the input buffer
    pub fn extend(&mut self) -> std::io::Result<usize> {
        // We have to reduce the inner buffer to avoid a buffer that can grow infinitly
        self.ext.drain(..self.inner_start);
        self.inner_start = 0;

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

        if sz_read == 0 {
            self.eos_reached = true;
        }

        // Return the position of the readed part
        Ok(sz_read)
    }
    ///
    /// Unstack the buffer extender
    pub fn pop_buf_into(&mut self, buf: &mut [u8]) -> usize {
        let sz = cmp::min(self.len(), buf.len());
        buf[..sz].copy_from_slice(&self.ext[self.inner_start..self.inner_start + sz]);
        self.inner_start += sz;
        sz
    }
    ///
    /// Remove a certain number of elements at the begin of the extend buffer
    pub fn drain(&mut self, range: Range<usize>) {
        if range.start == 0 {
            self.inner_start += range.end;
        } else {
            let new_start = range.start + self.inner_start;
            let new_end = range.end + self.inner_start;
            self.ext.drain(new_start..new_end);
        }
    }
    ///
    /// Read the input buffer
    pub fn read_direct(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.reader.read(buf)
    }
    ///
    ///
    pub fn push_at_begin(&mut self, buf: &[u8]) {
        self.ext.drain(..self.inner_start);
        self.inner_start = 0;

        if buf.len() <= self.inner_start {
            self.inner_start -= buf.len();
            self.ext[self.inner_start..self.inner_start + buf.len()].copy_from_slice(buf);
        } else {
            self.ext[0..self.inner_start].copy_from_slice(&buf[0..self.inner_start]);
            self.ext.splice(
                self.inner_start..self.inner_start,
                buf[self.inner_start..].iter().copied(),
            );
            self.inner_start = 0;
        }
    }
    ///
    /// Actual length of the internal buffer
    pub fn len(&self) -> usize {
        self.ext.len() - self.inner_start
    }
    ///
    /// Get a value
    pub fn at(&self, pos: usize) -> u8 {
        self.ext[pos + self.inner_start]
    }
    ///
    /// Indicate if End Of Stream is reached or not
    pub fn eos_reached(&self) -> bool {
        self.eos_reached
    }
    ///
    /// To iterate
    pub fn iter_growing<'b>(&'b mut self) -> BufGrowingExtIter<'b, 'a> {
        BufGrowingExtIter::new(self)
    }
    #[allow(dead_code)]
    pub fn cloned_internal_vec(&self) -> Vec<u8> {
        self.ext[self.inner_start..].to_vec()
    }
}

///
/// For debugging
impl<'a> fmt::Debug for BufExt<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "buf_extend={:?} sz_read_ext=[{:?}]",
            String::from_utf8_lossy(self.ext.as_slice()),
            self.sz_read_ext
        )
    }
}
