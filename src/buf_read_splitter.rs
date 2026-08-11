use core::fmt;
use std::{cmp, io::Read};

use crate::errors::*;
use crate::BufExt;
use crate::MatchResult;
use crate::Matcher;
use crate::Options;
use crate::PosSizeHelper;

///
/// BufReadSplitter : See unit test or lib documentations for an example
pub struct BufReadSplitter<'a, T: Matcher> {
    //reader: &'a mut dyn std::io::Read, // Buffer reader
    matcher: T,                     // The Matcher
    buf_extend: BufExt<'a>, // Extend buffer, need to detecte the matched part overflowing the output buffer
    options: Options,       // Options stores here
    matched: bool,          // Indicate that the pattern is matched
    curr_limit_read: Option<usize>, // Counter for the size limit to read
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

            self.matched = false; // We are now at the next buffer, nothing even read, nothing even matched
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

            if self.buf_extend.len() > 0 {
                sz_read = self.buf_extend.pop_buf_into(buf);
            }
            // Feed the remaining part by consumming the input buffer
            //todo: is necessary if there's a match inside it ?
            if sz_read < buf.len() {
                sz_read += self.buf_extend.read_direct(&mut buf[sz_read..])?;
            }

            match self.search_match(buf, sz_read)? {
                Some(ps_absolute) => {
                    let ps_bufext = PosSizeHelper::from_relative(&ps_absolute, buf.len());

                    // Save the part next to the matched part if there's one
                    // (we have to push at the begin because the buffer can already contains datas)
                    //todo: is there a simpler way ?
                    if ps_absolute.next_content_pos() < buf.len() {
                        self.buf_extend
                            .push_at_begin(&buf[ps_absolute.next_content_pos()..sz_read]);
                    }

                    // If a part of the buf_extend have not to be returned, we remove it
                    if ps_bufext.next_content_pos() > 0 {
                        self.buf_extend
                            .drain(ps_bufext.skipped_pos()..ps_bufext.next_content_pos());
                    }

                    // If there's something next to return in the buf_extend
                    if ps_absolute.skipped_pos() > buf.len() {
                        self.remain = ps_bufext.skipped_pos();
                    }

                    let sz_to_return = cmp::min(buf.len(), ps_absolute.skipped_pos());

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
    ///
    /*
     *
    fn convert_take_to_pos_sz(
        take_left: usize,
        take_right: usize,
        sz_matched: usize,
        pos: usize,
    ) -> (usize, usize) {
        if take_left + take_right > sz_matched {
            panic!("Size matched overflow ! take_left={take_left} + take_right={take_left} > sz_matched={sz_matched}")
        }
        let sz_returned = sz_matched - take_left - take_right;
        let pos_returned = pos - take_right;
        (sz_returned, pos_returned)
    }
    */
    ///
    /// Searching for a match in buf and buf_ext
    fn search_match(
        &mut self,
        buf: &[u8],
        sz_read: usize,
    ) -> std::io::Result<Option<PosSizeHelper>> {
        // Initialize
        let mut sz_matched = 0usize; //Size matched
        let mut pos = 0usize; //Absolute position of the last position that matched

        // Search in the buffer
        let mut state = self.search_match_in_buffer(buf, sz_read, &mut sz_matched, &mut pos);
        if matches!(state, MatchResult::NeedNext) {
            // Search in the extended buffer
            state = self.search_match_in_buf_extend(&mut sz_matched, &mut pos)?;
        }

        match state {
            MatchResult::Mismatch => Ok(None),
            MatchResult::Match(take_left, take_right) => {
                let pos_sz = PosSizeHelper::from_match(take_left, take_right, sz_matched, pos);
                Ok(Some(pos_sz))
            }
            MatchResult::NeedNext => {
                panic!("Abnormal case: normally there's no stop until end is reached")
            }
        }
    }

    fn search_match_in_buffer(
        &mut self,
        buf: &[u8],
        sz_read: usize,
        sz_matched: &mut usize,
        pos: &mut usize,
    ) -> MatchResult {
        let mut latest_state = MatchResult::Mismatch;
        for el in buf[..sz_read].into_iter() {
            latest_state = self.matcher.sequel(*el, *sz_matched);
            match latest_state {
                MatchResult::NeedNext => *sz_matched += 1,
                MatchResult::Match(_, _) => {
                    *sz_matched += 1;
                    return latest_state;
                }
                MatchResult::Mismatch => *sz_matched = 0,
            }
            *pos += 1;
        }
        latest_state
    }

    fn search_match_in_buf_extend(
        &mut self,
        sz_matched: &mut usize,
        pos: &mut usize,
    ) -> std::io::Result<MatchResult> {
        // We are here because the begin of the potentiel pattern has been found in the buffer part, so we have
        // to determine if it is really matched or not to stop the buffer.
        let it = self.buf_extend.iter_growing();
        for res in it {
            let state = self.matcher.sequel(res?, *sz_matched);
            match state {
                MatchResult::NeedNext => {
                    *sz_matched += 1;
                }
                MatchResult::Match(_, _) => {
                    *sz_matched += 1;
                    return Ok(state);
                }
                MatchResult::Mismatch => return Ok(state),
            }
            *pos += 1;
        }
        // We are at the end of the stream => we manage the EOS call
        if false == self.buf_extend.eos_reached() {
            Ok(MatchResult::Mismatch)
        } else {
            let state = self.matcher.sequel_eos(*sz_matched - 1);
            if matches!(state, MatchResult::Match(_, _)) {
                *pos -= 1;
            }
            Ok(state)
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
