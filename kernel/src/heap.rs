use core::{arch::asm, ops::Add};

use linked_list_allocator::LockedHeap;

use crate::{kprint, mem::allocate_page_frame};

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

fn add_kernel_lower_l2_page_directory_table() -> u64 {
    let entry_mask = 0x0008_ffff_ffff_f800;
    let mut kernel_cr3: u64;

    unsafe {
        asm!("mov r15, cr3", out("r15") kernel_cr3);

        let l4_page_map_l4_table_start = (kernel_cr3 & entry_mask) as *const u64;
        let l3_page_directory_pointer_table_start =
            (*l4_page_map_l4_table_start.add(256) & entry_mask) as *const u64;
        let l2_page_directory_table_start =
            (*l3_page_directory_pointer_table_start & entry_mask) as *mut u64;

        *l2_page_directory_table_start.add(10) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(11) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(12) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(13) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(14) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(15) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(16) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(17) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(18) = allocate_page_frame() | 0x83;
        *l2_page_directory_table_start.add(19) = allocate_page_frame() | 0x83;
    }

    // TODO this is hard coded, as we are adding to the 10th entry above --> make it dynmic
    return 0xffff800001400000;
}

pub fn init_kernel_heap() {
    // TODO add more / dynamic page frames
    // TODO do not start with new page frame, but start where kernel ends

    let heap_start_phys = add_kernel_lower_l2_page_directory_table();

    let heap_start_virtual = heap_start_phys;

    unsafe {
        ALLOCATOR
            .lock()
            .init(heap_start_virtual as *mut u8, 0x200000 * 10);
    }
}
