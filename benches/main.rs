use divan::AllocProfiler;

mod buf_read_splitter;
//mod claude_recommandation;
mod minimal_count;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();
