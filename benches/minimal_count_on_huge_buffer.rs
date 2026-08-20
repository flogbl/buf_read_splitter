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

const CONTENT_LEN: usize = 10;

pub fn bench() {
    // Report header
    println!("|{}|", "-".repeat(35));
    println!("|  {:>6}   |  {:>6}   |  {:>6}   |", "sz", "buf", "min");
    println!("|{}|", "-".repeat(35));

    // Init
    let mut proc_info = ProcInfo::new();

    for buf_size in [10, 100, 1_000, 10_000, 100_000, 1_000_000] {
        let cpu1_before = proc_info.cpu_time();
        buf_read_splitter(buf_size);
        let cpu1_after = proc_info.cpu_time();
        let cpu2_before = proc_info.cpu_time();
        count_minimaliste(buf_size);
        let cpu2_after = proc_info.cpu_time();

        println!(
            "| {:>9.3} | {:>6.3} ms | {:>6.3} ms |",
            buf_size,
            cpu1_after - cpu1_before,
            cpu2_after - cpu2_before
        );
    }

    // Report footer
    println!("|{}|", "-".repeat(35));
}

fn buf_read_splitter(buf_size: usize) {
    let nbr_of_iterations = 10_000_000 / CONTENT_LEN;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(CONTENT_LEN, separator, nbr_of_iterations);

    let mut reader = BufReadSplitter::new(
        &mut stream,
        SimpleMatcher::new(separator.as_bytes()),
        Options::default(),
    );

    let mut buf = vec![0u8; buf_size];

    let mut nb_part_found = 0usize;
    let mut nb_datas = 0usize;

    while reader.next().unwrap() {
        nb_part_found += 1;
        let mut sz;
        while {
            sz = reader.read(&mut buf).unwrap();
            sz > 0
        } {
            nb_datas += 1;
        }
    }

    assert!(
        nb_part_found == nbr_of_iterations,
        "nb_found different of nbr_of_iterations ( {nb_part_found} != {nbr_of_iterations} )"
    );
    assert!(nb_datas > 0)
}

fn count_minimaliste(buf_size: usize) {
    let nbr_of_iterations = 10_000_000 / CONTENT_LEN;
    let separator = "<SEP>";
    let mut stream = StreamGenerator::new(CONTENT_LEN, separator, nbr_of_iterations);

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
