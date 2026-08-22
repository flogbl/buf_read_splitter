///
/// This test aims to verify there's no performance problem with
/// huge buffers on small chunks (10 octets)
///

#[path = "common/mod.rs"]
mod common;
mod fn_all_in_memory;
mod fn_count_minimaliste;
mod fn_count_throw_buf_read_splitter;

use crate::common::allocinfo::*;
use crate::common::format_octet::format_octet;
use crate::fn_all_in_memory::*;
use crate::fn_count_minimaliste::*;
use crate::fn_count_throw_buf_read_splitter::*;

const CONTENT_LEN: usize = 10_000;
const TOTAL_STREAM_SIZE: usize = 100_000_000;

#[global_allocator]
static GLOBAL: MyAllocator = MyAllocator;

pub fn main() {
    // Report header
    println!("|{}|", "-".repeat(47));
    println!("|{:^47}|", "* Memory peak *");
    println!(
        "|{:<47}|",
        format!("Total stream size: {}", format_octet(TOTAL_STREAM_SIZE))
    );
    println!("|{:<47}|", format!("Part size: {CONTENT_LEN} o"));
    println!("|{}|", "-".repeat(47));
    println!(
        "|  {:>6}   |  {:>6}   |  {:>6}   |  {:>6}   |",
        "buf sz", "brs", "min", "allmem"
    );
    println!("|{}|", "-".repeat(47));

    for buf_size in [10, 100, 1_000, 10_000, 100_000, 1_000_000] {
        reset_tracking();
        count_throw_buf_read_splitter(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let peak_1 = get_peak_memory();

        reset_tracking();
        count_minimaliste(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let peak_2 = get_peak_memory();

        reset_tracking();
        all_in_memory(TOTAL_STREAM_SIZE, buf_size, CONTENT_LEN);
        let peak_3 = get_peak_memory();

        println!(
            "| {:>9} | {:>9} | {:>9} | {:>9} |",
            buf_size,
            format_octet(peak_1),
            format_octet(peak_2),
            format_octet(peak_3)
        );
    }

    // Report footer
    println!("|{}|", "-".repeat(47));
}
