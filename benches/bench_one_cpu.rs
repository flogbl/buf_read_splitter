//
/// This test aims to verify there's no performance problem with
/// huge buffers on small chunks (10 octets)
///

#[path = "common/mod.rs"]
mod common;
mod fn_all_in_memory;
mod fn_count_minimaliste;
mod fn_count_throw_buf_read_splitter;

use crate::common::procinfo::*;
use crate::fn_all_in_memory::*;
use crate::fn_count_minimaliste::*;
use crate::fn_count_throw_buf_read_splitter::*;

const CONTENT_LEN: usize = 10;
const TOTAL_STREAM_SIZE: usize = 1_000_000_000;

pub fn main() {
    set_one_cpu();

    // Report header
    println!("|{}|", "-".repeat(47));
    println!("|{:^47}|", "* CPU Time *");
    println!(
        "|{:<47}|",
        format!("Total stream size: {TOTAL_STREAM_SIZE} o")
    );
    println!("|{:<47}|", format!("Part size: {CONTENT_LEN} o"));
    println!("|{}|", "-".repeat(47));
    println!(
        "|  {:>6}   |  {:>6}   |  {:>6}   |  {:>6}   |",
        "buf sz", "brs", "min", "allmem"
    );
    println!("|{}|", "-".repeat(47));

    // Init
    let mut proc_info = Cpu::new();

    for buf_size in [10, 100, 1_000, 10_000, 100_000, 1_000_000] {
        let cpu1_before = proc_info.cpu_time();
        count_throw_buf_read_splitter(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let cpu1_after = proc_info.cpu_time();

        let cpu2_before = proc_info.cpu_time();
        count_minimaliste(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let cpu2_after = proc_info.cpu_time();

        let cpu3_before = proc_info.cpu_time();
        all_in_memory(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let cpu3_after = proc_info.cpu_time();

        println!(
            "| {:>9.3} | {:>6.3} ms | {:>6.3} ms | {:>6.3} ms |",
            buf_size,
            cpu1_after - cpu1_before,
            cpu2_after - cpu2_before,
            cpu3_after - cpu3_before
        );
    }

    // Report footer
    println!("|{}|", "-".repeat(47));
}

fn set_one_cpu() {
    let core_ids = core_affinity::get_core_ids().unwrap();

    if let Some(core_id) = core_ids.first() {
        let result = core_affinity::set_for_current(*core_id);
        if false == result {
            panic!("Can't attach to one thread only")
        }
    }
}
