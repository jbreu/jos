use crate::{kprint, util::compare_str_to_memory};

// https://wiki.osdev.org/RSDP
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
struct XSDP_t {
    Signature: [char; 8],
    Checksum: u8,
    OEMID: [char; 6],
    Revision: u8,
    RsdtAddress: u32, // deprecated since version 2.0
    Length: u32,
    XsdtAddress: u64,
    ExtendedChecksum: u8,
    reserved: [u8; 3],
}

fn find_xsdp() -> *const XSDP_t {
    let ebda_address = 0x040E as *const u16;
    let bios_rom_start = 0xE0000 as u32;
    let bios_rom_end = 0xFFFFF as u32;

    // Search in EBDA
    let ebda_seg = unsafe { *ebda_address };
    let ebda: u32 = (ebda_seg as u32) << 4;
    for address in (ebda..(ebda + 1024)).step_by(16) {
        if compare_str_to_memory("RSD PTR ", address as usize) {
            return address as *const XSDP_t;
        }
    }

    // Search in BIOS ROM area
    for address in (bios_rom_start..bios_rom_end).step_by(16) {
        if compare_str_to_memory("RSD PTR ", address as usize) {
            return address as *const XSDP_t;
        }
    }

    panic!("RSDP not found");
}

pub fn init_acpi() {
    let xsdp = find_xsdp();

    unsafe {
        kprint!("Revision: {:x}", { (*xsdp).RsdtAddress });
    }
}
