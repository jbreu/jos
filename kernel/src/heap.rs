use core::arch::asm;

use linked_list_allocator::LockedHeap;

use crate::mem::allocate_page_frame;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

const ENTRY_MASK: u64 = 0x0008_ffff_ffff_f800;
const PAGE_ENTRY_FLAGS: u64 = 0x83; // RW + Present + Other Flags

fn add_kernel_lower_l2_page_directory_table() -> u64 {
    let mut kernel_cr3: u64;

    unsafe {
        // Load the CR3 register into kernel_cr3
        asm!("mov {}, cr3", out(reg) kernel_cr3);

        let l4_pml4_table = (kernel_cr3 & ENTRY_MASK) as *const u64;
        let l3_pdpt = (*l4_pml4_table.add(256) & ENTRY_MASK) as *const u64;
        let l2_page_dir = (*l3_pdpt & ENTRY_MASK) as *mut u64;

        // Allocate page frames and update L2 page directory entries
        for i in 20..30 {
            *l2_page_dir.add(i) = allocate_page_frame() | PAGE_ENTRY_FLAGS;
        }
    }

    // TODO this is hard coded, as we are adding to the 10th entry above --> make it dynamic
    return 0xffff800002800000;
}

pub fn init_kernel_heap() {
    // TODO add more / dynamic page frames
    // TODO do not start with new page frame, but start where kernel ends

    // Hardcoded heap size: 10 * 2MB pages
    const HEAP_SIZE: usize = 0x200000 * 10;

    let heap_start_phys = add_kernel_lower_l2_page_directory_table();

    let heap_start_virtual = heap_start_phys;

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start_virtual as *mut u8, HEAP_SIZE);
    }
}
