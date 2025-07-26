use core::arch::asm;
use linked_list_allocator::LockedHeap;

use crate::{mem::allocate_page_frame, mem_config::*};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn allocate_kernel_heap_pages_after_already_allocated_memory() -> usize {
    let mut kernel_cr3: u64;

    unsafe {
        // Load the CR3 register into kernel_cr3
        asm!("mov {}, cr3", out(reg) kernel_cr3);

        let l4_pml4_table = (kernel_cr3 & ENTRY_MASK) as *const u64;
        let l3_pdpt = (*l4_pml4_table.add(256) & ENTRY_MASK) as *const u64;
        let l2_page_dir = (*l3_pdpt & ENTRY_MASK) as *mut u64;

        // Allocate page frames and update L2 page directory entries
        for i in KERNEL_SIZE / PAGE_SIZE..(KERNEL_HEAP_SIZE / PAGE_SIZE + KERNEL_SIZE / PAGE_SIZE) {
            *l2_page_dir.add(i) = allocate_page_frame() | PAGE_ENTRY_FLAGS_KERNELSPACE;
        }
    }

    // TODO this is hard coded, as we are adding to the 10th entry above --> make it dynamic
    return KERNEL_HIGHER_HALF_BASE + KERNEL_SIZE;
}

pub fn init_kernel_heap() {
    // TODO add more / dynamic page frames
    // TODO do not start with new page frame, but start where kernel ends

    let heap_start_phys = allocate_kernel_heap_pages_after_already_allocated_memory();

    let heap_start_virtual = heap_start_phys;

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start_virtual as *mut u8, KERNEL_HEAP_SIZE);
    }
}
