use crate::kprint;
use crate::mem::allocate_page_frame;
use core::arch::asm;
use core::mem;
use core::ptr::addr_of;

pub fn fopen() -> u64 {
    extern "C" {
        static mut _binary_doom1_wad_start: u8;
        static mut _binary_doom1_wad_end: u8;
    }

    unsafe {
        kprint!(
            "embedded doom1.wad file\nstart: {:x}\n  end: {:x}\n",
            addr_of!(_binary_doom1_wad_start) as *const u8 as usize,
            addr_of!(_binary_doom1_wad_end) as *const u8 as usize
        );

        let size = addr_of!(_binary_doom1_wad_end) as *const u8 as usize
            - addr_of!(_binary_doom1_wad_start) as *const u8 as usize;

        return 0;
    }
}
