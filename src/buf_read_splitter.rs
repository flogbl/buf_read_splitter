use core::fmt;
use std::fmt::Error;
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
    remain: usize,
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
            remain: 0,
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
            #[cfg(feature = "log")]
            log::debug!("Set matched to FALSE");

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
            (self.matched == false || self.remain > 0) && sz_read != 0
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

        if self.matched {
            // Here to manage the remain part to return in the actual buffer
            if self.remain == 0 {
                #[cfg(feature = "log")]
                log::debug!("Matched but no remain");
                Ok(0)
            } else {
                let sz_max = cmp::min(self.remain, buf.len());
                let sz = self.buf_extend.pop_buf_into(&mut buf[0..sz_max]);
                self.remain -= sz;

                // Debug
                #[cfg(feature = "log")]
                Self::log_read(
                    "Remain ",
                    &buf[0..sz],
                    &buf[0..0],
                    &self.buf_extend.cloned_internal_vec(),
                    "",
                );

                Ok(sz)
            }
        } else {
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
                    // Calculate absolute position (in buf+buf_ext) and relative positions (in buf_ext)
                    let abs_end = pos + 1;
                    let abs_start = abs_end - sz_matched;
                    let rel_end = {
                        if abs_end > buf.len() {
                            abs_end - buf.len()
                        } else {
                            0
                        }
                    };
                    let rel_start = {
                        if rel_end > sz_matched {
                            rel_end - sz_matched
                        } else {
                            0
                        }
                    };

                    // Save the part next to the matched part if there's one
                    if abs_end < buf.len() {
                        self.buf_extend.push_at_begin(&buf[abs_end..sz_read]);
                    }

                    // If a part of the buf_extend must not be returned, we remove it
                    if rel_end > 0 {
                        self.buf_extend.drain(rel_start..rel_end); //.pop_buf(pos - buf.len() + 1);
                    }

                    // If there's something next to return in the extend buf
                    if abs_start > buf.len() {
                        self.remain = rel_start;
                    }

                    let sz_to_return = cmp::min(buf.len(), abs_start);

                    // Debug
                    #[cfg(feature = "log")]
                    Self::log_read(
                        "Match ",
                        &buf[0..sz_to_return],
                        &buf[sz_to_return..buf.len()],
                        &self.buf_extend.cloned_internal_vec(),
                        &format!("sz_to_return={sz_to_return} bs_start={abs_start} abs_end={abs_end} rel_start={rel_start} rel_end={rel_end} self.remain={remain}",remain=self.remain),
                    );

                    self.matched = true;
                    Ok(sz_to_return)
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
    }
    ///
    /// Searching for a match in buf and buf_ext
    fn search_match(
        &mut self,
        buf: &mut [u8],
        sz_read: usize,
    ) -> std::io::Result<Option<(usize, usize)>> {
        let mut sz_matched = 0; //Size matched
        let mut pos = 0usize; //Absolute position of the last position that matched

        // For factorisation of the common part in the two loops
        let fn_calc_returned = |take_left: usize,
                                take_right: usize,
                                sz_matched: usize,
                                pos: usize| {
            if take_left + take_right > sz_matched {
                panic!("Size matched overflow ! take_left={take_left} + take_right={take_left} > sz_matched={sz_matched}")
            }
            let sz_returned = sz_matched - take_left - take_right;
            let pos_returned = pos - take_right;
            (sz_returned, pos_returned)
        };

        // Search in buf
        for el in buf[..sz_read].into_iter() {
            match self.matcher.sequel(*el, sz_matched) {
                MatchResult::NeedNext => sz_matched += 1,
                MatchResult::Match(take_left, take_right) => {
                    sz_matched += 1;
                    let res = fn_calc_returned(take_left, take_right, sz_matched, pos);
                    return Ok(Some(res));
                }
                MatchResult::Mismatch => sz_matched = 0,
            }
            pos += 1;
        }

        if sz_matched == 0 {
            Ok(None)
        } else
        // Continue to search in buf_ext if needed
        {
            let it = self.buf_extend.iter_growing();
            for res in it {
                let el = res?;
                match self.matcher.sequel(el, sz_matched) {
                    MatchResult::NeedNext => {
                        sz_matched += 1;
                    }
                    MatchResult::Match(take_left, take_right) => {
                        sz_matched += 1;
                        let res = fn_calc_returned(take_left, take_right, sz_matched, pos);
                        return Ok(Some(res));
                    }
                    MatchResult::Mismatch => return Ok(None),
                }
                pos += 1;
            }
            // We arrived here because in NeedNext state, so we have to manage the EOS call
            if false == self.buf_extend.eos_reached() {
                Ok(None)
            } else {
                match self.matcher.sequel_eos(sz_matched - 1) {
                    MatchResult::Match(take_left, take_right) => {
                        let res = fn_calc_returned(take_left, take_right, sz_matched, pos - 1);
                        Ok(Some(res))
                    }
                    _ => Ok(None),
                }
            }
        }
    }
    ///
    /// Log read
    #[cfg(feature = "log")]
    fn log_read(comment: &str, out_buf: &[u8], matched: &[u8], ext_buf: &[u8], comment_end: &str) {
        use format_hex::format_hex::FormatHex;
        use log::debug;
        let (l1, l2, l3) = FormatHex::new()
            .push_comment(comment)
            .push_comment("in[")
            .push_hex(out_buf)
            .push_comment("] ign[")
            .push_hex(matched)
            .push_comment("] ext[")
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
        if self.matched == true && self.remain == 0 {
            #[cfg(feature = "log")]
            log::debug!("Must call next first !");

            return Ok(0); // Must call next first !
        }
        if let Some(sz) = self.curr_limit_read {
            let max = cmp::min(sz, buf.len());
            if max == 0 {
                #[cfg(feature = "log")]
                log::debug!("curr_limit_read reached !");

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
