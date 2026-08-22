use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static CURRENT_ALLOC: AtomicUsize = AtomicUsize::new(0);

pub static PEAK_ALLOC: AtomicUsize = AtomicUsize::new(0);

pub struct MyAllocator;

pub fn get_peak_memory() -> usize {
    PEAK_ALLOC.load(Ordering::Relaxed)
}

pub fn reset_tracking() {
    CURRENT_ALLOC.store(0, Ordering::Relaxed);
    PEAK_ALLOC.store(0, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for MyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _prev = CURRENT_ALLOC.fetch_add(layout.size(), Ordering::Relaxed);

        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let prev = CURRENT_ALLOC.fetch_sub(layout.size(), Ordering::Relaxed);
        PEAK_ALLOC.fetch_max(prev, Ordering::Relaxed);

        unsafe { System.dealloc(ptr, layout) }
    }
}
