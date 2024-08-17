use crate::kprint;
use core::panic;
use core::ptr::addr_of;

static mut FILE_POSITION: usize = 0;

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
    }

    return 0;
}

pub fn fread(ptr: *mut u8, num_bytes: usize) -> u64 {
    extern "C" {
        static mut _binary_doom1_wad_start: u8;
    }

    unsafe {
        for i in 0..num_bytes {
            let dst = ptr.add(i);
            let src = addr_of!(_binary_doom1_wad_start).byte_add(FILE_POSITION + i);

            core::ptr::write_volatile(dst, *src);
        }

        FILE_POSITION += num_bytes;
    }

    return num_bytes as u64;
}

pub fn fseek(offset: usize, origin: usize) -> u64 {
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

    return 0;
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
