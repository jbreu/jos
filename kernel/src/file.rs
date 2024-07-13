use crate::kprint;
use crate::mem::allocate_page_frame;
use core::arch::asm;
use core::ptr::addr_of;
use core::{mem, num};
use core::{panic, ptr};

static mut FILE_POSITION: usize = 0;

pub fn fopen() {
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
    }
}

pub fn fread(ptr: *mut u8, num_bytes: usize) {
    extern "C" {
        static mut _binary_doom1_wad_start: *mut u8;
    }

    unsafe {
        for i in 0..num_bytes {
            core::ptr::write_volatile(
                ptr.add(i),
                _binary_doom1_wad_start.add(FILE_POSITION + i) as u8,
            );
        }

        FILE_POSITION += num_bytes;
    }
}

pub fn fseek(offset: usize, origin: usize) {
    extern "C" {
        static mut _binary_doom1_wad_start: u8;
        static mut _binary_doom1_wad_end: u8;
    }

    unsafe {
        let size = addr_of!(_binary_doom1_wad_end) as *const u8 as usize
            - addr_of!(_binary_doom1_wad_start) as *const u8 as usize;

        match origin {
            0 => FILE_POSITION = offset,
            1 => FILE_POSITION = FILE_POSITION + offset,
            2 => FILE_POSITION = size - offset,
            _ => panic!("undefined fseek"),
        }
    }
}

pub fn ftell() -> usize {
    return unsafe { FILE_POSITION };
}

pub fn feof() -> u64 {
    extern "C" {
        static mut _binary_doom1_wad_start: u8;
        static mut _binary_doom1_wad_end: u8;
    }

    unsafe {
        let size = addr_of!(_binary_doom1_wad_end) as *const u8 as usize
            - addr_of!(_binary_doom1_wad_start) as *const u8 as usize;

        if FILE_POSITION > size {
            return 1;
        } else {
            return 0;
        }
    }
}
