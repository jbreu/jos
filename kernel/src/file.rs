use crate::kprint;
use core::panic;
use core::ptr::addr_of;

static mut FILE_POSITION: usize = 0;

unsafe fn file_start() -> *const u8 {
    extern "C" {
        static mut _binary_doom1_wad_start: u8;
    }
    addr_of!(_binary_doom1_wad_start) as *const u8
}

unsafe fn file_end() -> *const u8 {
    extern "C" {
        static mut _binary_doom1_wad_end: u8;
    }
    addr_of!(_binary_doom1_wad_end) as *const u8
}

unsafe fn file_size() -> usize {
    file_end() as usize - file_start() as usize
}

pub fn fopen() -> u64 {
    unsafe {
        kprint!(
            "embedded doom1.wad file\nstart: {:x}\n  end: {:x}\n",
            file_start() as usize,
            file_end() as usize
        );
    }
    1
}

pub fn fread(ptr: *mut u8, num_bytes: usize) -> u64 {
    unsafe {
        for i in 0..num_bytes {
            let dst = ptr.add(i);
            let src = file_start().add(FILE_POSITION + i);

            core::ptr::write_volatile(dst, *src);
        }

        FILE_POSITION += num_bytes;
    }
    num_bytes as u64
}

pub fn fseek(offset: usize, origin: usize) -> u64 {
    unsafe {
        let size = file_size();

        match origin {
            0 => FILE_POSITION = offset,
            1 => FILE_POSITION += offset,
            2 => FILE_POSITION = size - offset,
            _ => panic!("undefined fseek"),
        }
    }
    0
}

pub fn ftell() -> usize {
    unsafe { FILE_POSITION }
}

pub fn feof() -> u64 {
    unsafe {
        if FILE_POSITION >= file_size() {
            1
        } else {
            0
        }
    }
}
