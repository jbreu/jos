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

        let slice = core::slice::from_raw_parts(addr_of!(_binary_doom1_wad_start), size);

        let first_page = allocate_page_frame();
        let second_page = allocate_page_frame();
        let third_page = allocate_page_frame();

        asm!(
            "mov rcx, {}
            mov rsi, {}
            mov rdi, {}
            rep movsb",
            in(reg) size,
            in(reg) _binary_doom1_wad_start as usize,
            in(reg) first_page,
            out("rcx") _,
            out("rsi") _,
            out("rdi") _
        );

        return first_page;
    }
}
