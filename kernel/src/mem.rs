use crate::{mem, mem_config::*, process};
use core::{
    arch::asm,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

static AVAILABLE_MEMORY: [AtomicBool; MAX_PAGE_FRAMES] =
    [const { AtomicBool::new(false) }; MAX_PAGE_FRAMES];
static NEXT_FREE_PAGE: AtomicUsize = AtomicUsize::new(0);

static mut PHYSICAL_MEMORY_PAGE_TABLES: [process::PageTable;
    MAX_MEMORY / HUGE_PAGE_SIZE / PAGE_TABLE_ENTRIES] = [const {
    process::PageTable {
        entry: [0; PAGE_TABLE_ENTRIES],
    }
};
    MAX_MEMORY / HUGE_PAGE_SIZE / PAGE_TABLE_ENTRIES];
static mut PHYSICAL_MEMORY_PAGE_DIRECTORY_POINTER_TABLE: process::PageTable = process::PageTable {
    entry: [0; PAGE_TABLE_ENTRIES],
};

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

// Maps all RAM to the same virtual address as physical memory, so that we can access it directly without needing to set up page tables first
pub fn physical_memory_mapping() {
    let mut physical_address = 0;
    let mut virtual_address = PHYSICAL_MEMORY_BASE;

    unsafe {
        if process::KERNEL_CR3.load(Ordering::Relaxed) == 0 {
            let mut cr3: u64;
            asm!("mov r15, cr3", out("r15") cr3);
            process::KERNEL_CR3.store(cr3 as usize, Ordering::Relaxed);
        }

        let kernel_cr3 = process::KERNEL_CR3.load(Ordering::Relaxed) as *mut process::PageTable;

        (*kernel_cr3).entry[257] = &raw const PHYSICAL_MEMORY_PAGE_DIRECTORY_POINTER_TABLE
            as *const process::PageTable as usize
            | BASE_PAGE_ENTRY_FLAGS_USERSPACE as usize;

        // use huge page size to identity map physical memory, so that we don't have to set up page tables for the identity mapping and can just use the page directory pointer table to map it all at once
        for i in 0..(MAX_MEMORY / HUGE_PAGE_SIZE / PAGE_TABLE_ENTRIES) {
            PHYSICAL_MEMORY_PAGE_DIRECTORY_POINTER_TABLE.entry[i] =
                &raw const PHYSICAL_MEMORY_PAGE_TABLES[i] as *const process::PageTable as usize
                    | BASE_PAGE_ENTRY_FLAGS_USERSPACE as usize;
        }

        while physical_address < MAX_MEMORY {
            PHYSICAL_MEMORY_PAGE_TABLES[physical_address / HUGE_PAGE_SIZE / PAGE_TABLE_ENTRIES]
                .entry[physical_address / HUGE_PAGE_SIZE % PAGE_TABLE_ENTRIES] =
                physical_address | HUGE_PAGE_ENTRY_FLAGS_USERSPACE as usize;
            physical_address += 0x200000;
            virtual_address += 0x200000;
        }
    }
}
