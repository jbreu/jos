use crate::{mem_config::*, process};
use core::{arch::asm, sync::atomic::Ordering};

static mut AVAILABLE_MEMORY: [bool; MAX_PAGE_FRAMES] = [false; MAX_PAGE_FRAMES];

pub fn init_available_memory() {
    for i in 0..(KERNEL_SIZE / PAGE_SIZE) {
        unsafe {
            AVAILABLE_MEMORY[i] = true;
        }
    }
}

pub fn allocate_page_frame() -> u64 {
    let _event = core::hint::black_box(crate::instrument!());
    // TODO make safe
    // TODO make faster by not iterating instead storing next free page frame
    unsafe {
        for i in 0..MAX_PAGE_FRAMES - 1 {
            if AVAILABLE_MEMORY[i] == false {
                AVAILABLE_MEMORY[i] = true;
                return i as u64 * PAGE_SIZE as u64;
            }
        }
    }

    panic!("No more page frames available!");
}

pub fn allocate_page_frame_for_given_physical_address(address: usize) -> usize {
    let _event = core::hint::black_box(crate::instrument!());
    unsafe {
        let page = address / PAGE_SIZE;
        AVAILABLE_MEMORY[page] = true;
        return page as usize * PAGE_SIZE as usize;
    }
}

pub fn map_page_in_page_tables(page: usize, l4: usize, l3: usize, l2: usize, bitmask: u8) {
    let _event = core::hint::black_box(crate::instrument!());

    unsafe {
        if process::KERNEL_CR3.load(Ordering::Relaxed) == 0 {
            let mut cr3: u64;
            asm!("mov r15, cr3", out("r15") cr3);
            process::KERNEL_CR3.store(cr3, Ordering::Relaxed);
        }

        let l4table =
            (process::KERNEL_CR3.load(Ordering::Relaxed) & ENTRY_MASK) as *const process::PageTable;

        let l3table = ((*l4table).entry[l4] & ENTRY_MASK) as *const process::PageTable;

        let l2table = ((*l3table).entry[l3] & ENTRY_MASK) as *mut process::PageTable;

        (*l2table).entry[l2] = page as u64 | bitmask as u64;
    }
}
