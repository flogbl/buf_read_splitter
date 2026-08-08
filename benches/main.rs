use divan::AllocProfiler;

mod minimal_count_on_huge_buffer;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
