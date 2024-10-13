use crate::process;
use core::arch::asm;

// TODO make more elegant
// available memory in qemu by default is 128 MByte (2^27); we are using 2 MByte page frames (2^21) -> 2^(27-21) = 64

const MAX_PAGE_FRAMES: usize = 2048;
static mut AVAILABLE_MEMORY: [bool; MAX_PAGE_FRAMES] = {
    let mut array = [false; MAX_PAGE_FRAMES];

    // some page frames are already allocated in main.asm -> setup_page_tables
    array[0] = true;
    array[1] = true;
    array[2] = true;
    array[3] = true;
    array[4] = true;
    array[5] = true;
    array[6] = true;
    array[7] = true;
    array[8] = true;
    array[9] = true;

    array[10] = true;
    array[11] = true;
    array[12] = true;
    array[13] = true;
    array[14] = true;
    array[15] = true;
    array[16] = true;
    array[17] = true;
    array[18] = true;
    array[19] = true;

    // TODO Stack for interrupts, see HackID1
    array[20] = true;
    array
};

pub fn allocate_page_frame() -> u64 {
    // TODO make safe
    // TODO make faster by not iterating instead storing next free page frame
    unsafe {
        for i in 0..MAX_PAGE_FRAMES - 1 {
            if AVAILABLE_MEMORY[i] == false {
                AVAILABLE_MEMORY[i] = true;
                return i as u64 * 0x200000 as u64;
            }
        }
    }

    panic!("No more page frames available!");
}

pub fn allocate_page_frame_for_given_physical_address(address: usize) -> u64 {
    unsafe {
        let page = address / 0x200000;
        AVAILABLE_MEMORY[page] = true;
        return page as u64 * 0x200000 as u64;
    }
}

pub fn map_page_in_page_tables(page: u64, l4: usize, l3: usize, l2: usize, bitmask: u8) {
    let entry_mask: u64 = 0x0008_ffff_ffff_f800;

    unsafe {
        if process::KERNEL_CR3 == 0 {
            asm!("mov r15, cr3", out("r15") process::KERNEL_CR3);
        }

        let l4table = (process::KERNEL_CR3 & entry_mask) as *const process::PageTable;

        let l3table = ((*l4table).entry[l4] & entry_mask) as *const process::PageTable;

        let l2table = ((*l3table).entry[l3] & entry_mask) as *mut process::PageTable;

        (*l2table).entry[l2] = page | bitmask as u64;
    }
}
