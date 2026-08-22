#[path = "common/mod.rs"]
mod common;
use std::io::Read;

use crate::common::stream_generator::*;

pub fn count_minimaliste(total_length: usize, buf_size: usize, content_len: usize) {
    let nbr_of_iterations = total_length / content_len;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(content_len, separator, nbr_of_iterations);

    let mut buf = vec![0u8; buf_size];
    let mut nb_sep_found = 0usize;
    let mut nb_datas = 0usize;

    let mut pos_found = 0usize;
    let separator_bytes = separator.as_bytes();

    let mut sz;
    while {
        sz = stream.read(&mut buf).unwrap();
        sz > 0
    } {
        for &b in &buf[..sz] {
            if b == separator_bytes[pos_found] {
                pos_found += 1;
                if pos_found == separator_bytes.len() {
                    nb_sep_found += 1;
                    pos_found = 0;
                }
            } else if b == separator_bytes[0] {
                pos_found = 1;
                if pos_found == separator_bytes.len() {
                    nb_sep_found += 1;
                    pos_found = 0;
                }
            } else {
                if pos_found > 0 {
                    pos_found = 0;
                }
            }
        }
        nb_datas += sz;
    }

    assert!(
        nb_sep_found + 1 == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({nb_sep_found}!={nbr_of_iterations}) "
    );

    assert!(nb_datas > 0)
}
