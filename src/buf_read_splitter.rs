use core::fmt;
use std::{cmp, io::Read};

use crate::buf_ext::BufExt;
use crate::match_result::MatchResult;
use crate::matcher::Matcher;
use crate::options::Options;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

///
/// BufReadSplitter : See unit test or lib documentations for an example
pub struct BufReadSplitter<'a, T: Matcher> {
    //reader: &'a mut dyn std::io::Read, // Buffer reader
    matcher: T,                     // The Matcher
    buf_extend: BufExt<'a>, // Extend buffer, need to detecte the matched part overflowing the output buffer
    options: Options,       // Options stores here
    matched: bool,          // Indicate that the pattern is matched
    curr_limit_read: Option<usize>, // Counter of the limit to read
    #[cfg(feature = "log")]
    log_call_read: usize,
    #[cfg(feature = "log")]
    log_read_extend: usize,
    #[cfg(feature = "log")]
    log_resize_extend: usize,
}
///
/// Implementation
impl<'a, T: Matcher> BufReadSplitter<'a, T> {
    pub fn new(reader: &'a mut dyn std::io::Read, matcher: T, options: Options) -> Self {
        let max_read = options.limit_read;
        Self {
            //reader,
            matcher,
            buf_extend: BufExt::new(reader, options.initiale_sz_to_match, options.chunk_sz),
            options,
            matched: false,
            curr_limit_read: max_read,
            #[cfg(feature = "log")]
            log_call_read: 0,
            #[cfg(feature = "log")]
            log_read_extend: 0,
            #[cfg(feature = "log")]
            log_resize_extend: 0,
        }
    }
    ///
    /// Change the matcher
    pub fn matcher(&mut self, matcher: T) {
        self.matcher = matcher
    }
    ///
    /// Set a limit of bytes to read of a buffer part
    pub fn set_limit_read(&mut self, opt_sz: Option<usize>) {
        self.options.set_limit_read(opt_sz);
        self.curr_limit_read = opt_sz;
    }
    ///
    /// next buffer part
    pub fn next_part(&mut self) -> Result<Option<()>> {
        // We choose to return a Result<Option<()>> to be  representative of this logic :
        //   - call a function --> You have to manage a possible error
        //   - ok there's no error --> So is there something next
        if self.matched == false {
            self.skip_part()?;
        }

        if self.matched == false {
            Ok(None) // At the end of the input buffer
        } else {
            self.matched = false; // We are now on the next buffer, nothing even read, nothing even matched
            self.curr_limit_read = self.options.limit_read;
            Ok(Some(())) // It had just been stopping because it reached the separator
        }
    }

    // ====== PRIVATE FUNCTIONS ====== //

    ///
    /// Skip until the end of the part
    fn skip_part(&mut self) -> Result<()> {
        #[cfg(feature = "log")]
        log::debug!("====next_part skip this :");

        // Have to read until end of buffer or separator
        let mut buf = [0u8; 100];
        while {
            let sz_read = match self.internal_read(&mut buf) {
                Ok(o) => o,
                Err(err) => return Err(err.into()).into(),
            };
            // while condition :
            // At the end if :
            //   - matched and there's nothing more to take in the extend buffer
            //   - or end of file
            self.matched == false && sz_read != 0
        } {}
        #[cfg(feature = "log")]
        log::debug!("====next_part skip end====");
        Ok(())
    }
    ///
    /// Common read buffer function
    fn internal_read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        #[cfg(feature = "log")]
        {
            self.log_call_read += 1;
        }
        // Initialize the size to return
        let mut sz_read = 0;

        // First, feed the output buffer consuming datas of the previous read
        if self.buf_extend.len() > 0 {
            sz_read = self.buf_extend.pop_buf_into(buf);
        }
        // Feed the remaining part by consumming the input buffer
        if sz_read < buf.len() {
            sz_read += self.buf_extend.read_direct(&mut buf[sz_read..])?;
        }

        match self.search_match(buf, sz_read)? {
            Some((sz_matched, pos)) => {
                // Save the part next to the matched part if there's one
                if pos + 1 < buf.len() {
                    self.buf_extend.push_at_begin(&buf[pos + 1..sz_read]);
                }

                // If a part of the buf_extend must not be returned, we remove it
                if pos >= buf.len() {
                    self.buf_extend.pop_buf(pos - buf.len() + 1);
                }

                // Debug
                #[cfg(feature = "log")]
                Self::log_read(
                    "Match ",
                    &buf[0..sz_read],
                    &buf[sz_read..cmp::min(sz_read + sz_matched, buf.len())],
                    &self.buf_extend.cloned_internal_vec(),
                    "",
                );

                // The buffer ending exactly before the matched part (this position can only start inside the output buffer)
                self.matched = true;
                return Ok(pos + 1 - sz_matched);
            }
            None => {
                #[cfg(feature = "log")]
                Self::log_read(
                    "no match-",
                    &buf[0..sz_read],
                    &buf[sz_read..sz_read],
                    &self.buf_extend.cloned_internal_vec(),
                    "",
                );
                Ok(sz_read)
            }
        }
    }
    ///
    /// Searching for a match in buf and buf_ext
    fn search_match(
        &mut self,
        buf: &mut [u8],
        sz_read: usize,
    ) -> std::io::Result<Option<(usize, usize)>> {
        let mut sz_matched = 0;
        let mut pos = 0usize;

        for el in buf[..sz_read].into_iter() {
            match self.matcher.sequel(*el, sz_matched) {
                MatchResult::NeedNext => sz_matched += 1,
                MatchResult::Mismatch => sz_matched = 0,
                MatchResult::Match => {
                    sz_matched += 1;
                    return Ok(Some((sz_matched, pos)));
                }
            }
            pos += 1;
        }
        if sz_matched > 0 {
            let it = self.buf_extend.iter_grow();
            for res in it {
                let el = res?;
                match self.matcher.sequel(el, sz_matched) {
                    MatchResult::NeedNext => sz_matched += 1,
                    MatchResult::Match => {
                        sz_matched += 1;
                        return Ok(Some((sz_matched, pos)));
                    }
                    MatchResult::Mismatch => break,
                }
                pos += 1;
            }
        }
        Ok(None)
    }
    ///
    /// Log read
    #[cfg(feature = "log")]
    fn log_read(comment: &str, out_buf: &[u8], matched: &[u8], ext_buf: &[u8], comment_end: &str) {
        use format_hex::format_hex::FormatHex;
        use log::debug;
        let (l1, l2, l3) = FormatHex::new()
            .push_comment(comment)
            .push_comment("[")
            .push_hex(out_buf)
            .push_comment("]")
            .push_hex(matched)
            .push_comment("[")
            .push_hex(ext_buf)
            .push_comment("] ")
            .push_comment(comment_end)
            .output();
        debug!("{l1}");
        debug!("{l2}");
        debug!("{l3}");
    }
}
///
/// Read Implementation
impl<'a, T: Matcher> Read for BufReadSplitter<'a, T> {
    ///
    /// Read until the begin of a match or end of the buffer
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.matched == true {
            return Ok(0); // Must call next first !
        }
        if let Some(sz) = self.curr_limit_read {
            let max = cmp::min(sz, buf.len());
            if max == 0 {
                Ok(0)
            } else {
                let buf_slice = &mut buf[..max];
                let sz_read = self.internal_read(buf_slice)?;
                self.curr_limit_read = Some(sz - sz_read);
                Ok(sz_read)
            }
        } else {
            self.internal_read(buf)
        }
    }
}
///
/// For debugging
impl<'a, T: Matcher> fmt::Debug for BufReadSplitter<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res;
        #[cfg(feature = "log")]
        {
            res = write!(
                f,
                "buf_extend={:?} option=[{:?}] matched={:?} ({}/{}/{})",
                self.buf_extend,
                self.options,
                self.matched,
                self.log_call_read,
                self.log_read_extend,
                self.log_resize_extend,
            );
        }
        #[cfg(not(feature = "log"))]
        {
            res = write!(
                f,
                "buf_extend={:?} option=[{:?}] matched={:?}",
                self.buf_extend, self.options, self.matched
            );
        }
        res
    }
}
