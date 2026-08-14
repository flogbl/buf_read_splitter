///
/// This test aims to verify there's no performance problem with
/// huge buffers on small chunks (10 octets)
///

#[path = "common/mod.rs"]
mod common;

use buf_read_splitter::{BufReadSplitter, Options, SimpleMatcher};
use std::io::Read;

use crate::common::procinfo::*;
use crate::common::stream_generator::*;

const BUF_SIZE: usize = 100_000;
const CONTENT_LEN: usize = 10;

pub fn bench() {
    let mut proc_info = ProcInfo::new();

    let cpu1_before = proc_info.cpu_time();
    buf_read_splitter();
    let cpu1_after = proc_info.cpu_time();
    let cpu2_before = proc_info.cpu_time();
    count_minimaliste();
    let cpu2_after = proc_info.cpu_time();

    // Report
    println!("|{}|", "-".repeat(23));
    println!(
        "|  {:>6.3}   |  {:>6.3}   |",
        "buf_read_splitter", "minimal_count"
    );
    println!("|{}|", "-".repeat(23));
    println!(
        "| {:>6.3} ms | {:>6.3} ms |",
        cpu1_after - cpu1_before,
        cpu2_after - cpu2_before
    );
    println!("|{}|", "-".repeat(23));
}

fn buf_read_splitter() {
    let nbr_of_iterations = 10_000_000 / CONTENT_LEN;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(CONTENT_LEN, separator, nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(separator.as_bytes()),
        Options::default(),
    );

    let mut buf = vec![0u8; BUF_SIZE];

    let mut nb_part_found = 0usize;

    while {
        let sz = reader.read(&mut buf).unwrap();
        if sz > 0 {
            true
        } else {
            nb_part_found += 1;
            match reader.next_part().unwrap() {
                //Pass to the next part of the buffer
                Some(_) => true, //There's a next part
                None => false,   //End of the stream
            }
        }
    } {}
    assert!(
        nb_part_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ( {nb_part_found} != {nbr_of_iterations} )"
    )
}

fn count_minimaliste() {
    let nbr_of_iterations = 10_000_000 / CONTENT_LEN;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(CONTENT_LEN, separator, nbr_of_iterations);

    let mut buf = vec![0u8; BUF_SIZE];
    let mut nb_sep_found = 0usize;

    let mut pos_found = 0usize;
    let separator_bytes = separator.as_bytes();

    while {
        let sz = stream.read(&mut buf).unwrap();
        if sz > 0 {
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
            true
        } else {
            false
        }
    } {}

    assert!(
        nb_sep_found + 1 == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ({nb_sep_found}!={nbr_of_iterations}) "
    )
}
