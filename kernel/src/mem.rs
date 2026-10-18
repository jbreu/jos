use crate::mem_config::*;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static AVAILABLE_MEMORY: [AtomicBool; MAX_PAGE_FRAMES] =
    [const { AtomicBool::new(false) }; MAX_PAGE_FRAMES];
static NEXT_FREE_PAGE: AtomicUsize = AtomicUsize::new(0);

pub fn init_available_memory() {
    let _event = core::hint::black_box(crate::instrument!());
    // Kernel memory
    for i in 0..(KERNEL_SIZE / PAGE_SIZE) {
        AVAILABLE_MEMORY[i].store(true, Ordering::Relaxed);
    }

    NEXT_FREE_PAGE.store(KERNEL_SIZE / PAGE_SIZE, Ordering::Relaxed);
}

pub fn allocate_page_frame() -> usize {
    //let _event = core::hint::black_box(crate::instrument!());
    // TODO make safe
    for i in NEXT_FREE_PAGE.load(Ordering::Relaxed)..MAX_PAGE_FRAMES {
        if AVAILABLE_MEMORY[i].load(Ordering::Relaxed) == false {
            AVAILABLE_MEMORY[i].store(true, Ordering::Relaxed);
            NEXT_FREE_PAGE.store(i, Ordering::Relaxed);
            return i * PAGE_SIZE;
        }
    }

    panic!("No more page frames available!");
}
