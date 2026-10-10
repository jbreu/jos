use crate::{mem, mem_config::*, process};
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

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
    unsafe {
        for i in NEXT_FREE_PAGE.load(Ordering::Relaxed)..MAX_PAGE_FRAMES {
            if AVAILABLE_MEMORY[i].load(Ordering::Relaxed) == false {
                AVAILABLE_MEMORY[i].store(true, Ordering::Relaxed);
                NEXT_FREE_PAGE.store(i, Ordering::Relaxed);
                return i * PAGE_SIZE;
            }
        }
    }

    panic!("No more page frames available!");
}

pub fn allocate_page_frame_for_given_physical_address(address: usize) -> usize {
    let _event = core::hint::black_box(crate::instrument!());
    unsafe {
        let page = address / PAGE_SIZE;
        AVAILABLE_MEMORY[page].store(true, Ordering::Relaxed);
        return page as usize * PAGE_SIZE as usize;
    }
}

pub fn map_page_in_page_tables(
    page: usize,
    l4: usize,
    l3: usize,
    l2: usize,
    l1: usize,
    bitmask: u8,
) {
    let _event = core::hint::black_box(crate::instrument!());

    unsafe {
        if process::KERNEL_CR3.load(Ordering::Relaxed) == 0 {
            let mut cr3: u64;
            asm!("mov r15, cr3", out("r15") cr3);
            process::KERNEL_CR3.store(cr3 as usize, Ordering::Relaxed);
        }

        let l4table =
            (process::KERNEL_CR3.load(Ordering::Relaxed) & ENTRY_MASK) as *const process::PageTable;

        let l3table = ((*l4table).entry[l4] & ENTRY_MASK) as *mut process::PageTable;

        let l2table = ((*l3table).entry[l3] & ENTRY_MASK) as *mut process::PageTable;

        let l1table = ((*l2table).entry[l2] & ENTRY_MASK) as *mut process::PageTable;

        if PAGE_SIZE == BASE_PAGE_SIZE {
            (*l1table).entry[l1] = page | bitmask as usize;
        } else {
            (*l2table).entry[l2] = page | bitmask as usize;
        }
    }
}
