#[path = "common/mod.rs"]
mod common;

mod minimal_count_on_huge_buffer;
fn main() {
    minimal_count_on_huge_buffer::bench();
}
