use core::str;

use crate::DEBUG;

use crate::{mem, util::compare_str_to_memory};

// https://wiki.osdev.org/RSDP
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct XsdpT {
    signature: [u8; 8],
    checksum: u8,
    oemid: [u8; 6],
    revision: u8,
    rsdt_address: u32, // deprecated since version 2.0
    length: u32,
    xsdt_address: u64,
    extended_checksum: u8,
    reserved: [u8; 3],
}

// https://wiki.osdev.org/RSDT
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct ACPISDTHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oemid: [u8; 6],
    oemtable_id: [u8; 8],
    oemrevision: u32,
    creator_id: u32,
    creator_revision: u32,
}

// https://wiki.osdev.org/HPET
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct HPET {
    event_timer_block_id: u32,
    address_space_id: u8,
    register_bit_width: u8,
    register_bit_offset: u8,
    reserved: u8,
    base_address: u64,
    hpet_number: u8,
    minimum_tick: u16,
    page_protection: u8,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GeneralCapabilitiesAndIdRegister {
    rev_id: u8,
    capabilities: u8,
    vendor_id: u16,
    counter_clk_period: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct GeneralConfigurationRegister {
    config: u64, // only lowest 2 bits are in use
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct MainCounterValueRegister {
    pub main_counter_val: u64,
}

fn find_xsdp() -> *const XsdpT {
    let ebda_address = 0x040E as *const u16;
    let bios_rom_start = 0xE0000 as u32;
    let bios_rom_end = 0xFFFFF as u32;

    // Search in EBDA
    let ebda_seg = unsafe { *ebda_address };
    let ebda: u32 = (ebda_seg as u32) << 4;
    for address in (ebda..(ebda + 1024)).step_by(16) {
        if compare_str_to_memory("RSD PTR ", address as usize) {
            return address as *const XsdpT;
        }
    }

    // Search in BIOS ROM area
    for address in (bios_rom_start..bios_rom_end).step_by(16) {
        if compare_str_to_memory("RSD PTR ", address as usize) {
            return address as *const XsdpT;
        }
    }

    panic!("RSDP not found");
}

// Xsdt Address
// 64-bit physical address of the XSDT table. If you detect ACPI Version 2.0 you should use this table instead of RSDT even on IA-32, casting the address to uint32_t.
fn find_hpet_table() -> *const HPET {
    let xsdp = find_xsdp();

    unsafe {
        if (*xsdp).revision == 0 {
            mem::map_page_in_page_tables(
                mem::allocate_page_frame_for_given_physical_address((*xsdp).rsdt_address as usize),
                0,
                0,
                509,
                0b10000111,
            );

            let virt_rsdt_address =
                ((*xsdp).rsdt_address % 0x200000) as u64 + 0xffff_8000_3fa0_0000;

            DEBUG!("RSDT Address: {:?}", virt_rsdt_address as *const u64);

            let rsdt = virt_rsdt_address as *const ACPISDTHeader;

            DEBUG!("RSDT: {:?}", str::from_utf8(&(*rsdt).signature));

            let entries = ((*rsdt).length as usize - core::mem::size_of::<ACPISDTHeader>()) / 4;

            DEBUG!("RSDT entries: {}", entries);

            // The individual tables are pointed to 32bit pointers coming after the header
            let table_ptrs =
                (virt_rsdt_address as usize + core::mem::size_of::<ACPISDTHeader>()) as *const u32;

            DEBUG!(
                "RSDT table_ptrs: {:?} (size of ACPISDTHeader: {:?})",
                table_ptrs,
                core::mem::size_of::<ACPISDTHeader>()
            );

            for i in 0..entries {
                let header = core::ptr::read_unaligned(table_ptrs.add(i));
                let virt_header =
                    ((header as u64 % 0x200000) + 0xffff_8000_3fa0_0000) as *const ACPISDTHeader;

                DEBUG!(
                    "ACPI Entry: {:?}",
                    str::from_utf8(&(*virt_header).signature)
                );

                for j in 0..=3 {
                    if "HPET".as_bytes()[j] != (*virt_header).signature[j] {
                        break;
                    }
                    if j == 3 {
                        return (virt_header as usize + core::mem::size_of::<ACPISDTHeader>())
                            as *const HPET;
                    }
                }
            }
        } else {
            // Implement ACPI 2.0+
            panic!("You system seems to use ACPI 2.0 or newer, which is not implemented yet");
        }
    }

    panic!("HPET table not found");
}

pub fn init_acpi() {
    let hpet = find_hpet_table();

    unsafe {
        mem::map_page_in_page_tables(
            mem::allocate_page_frame_for_given_physical_address((*hpet).base_address as usize),
            0,
            0,
            508,
            0b10000111,
        );

        let capabilities = (((*hpet).base_address % 0x200000) + 0xffff_8000_3f80_0000)
            as *const GeneralCapabilitiesAndIdRegister;

        let frequency = 10_u64.pow(15) / (*capabilities).counter_clk_period as u64;
        DEBUG!("frequency: {}", frequency);

        HPET_CLOCK_PERIOD_IN_NS = ((*capabilities).counter_clk_period / 1_000_000) as u64;

        let configuration = ((((*hpet).base_address + 0x10) % 0x200000) + 0xffff_8000_3f80_0000)
            as *mut GeneralConfigurationRegister;

        (*configuration).config = 0x1;

        HPET_COUNTER_VALUE = ((((*hpet).base_address + 0xf0) % 0x200000) + 0xffff_8000_3f80_0000)
            as *const MainCounterValueRegister;
    }
}

// TODO better use lazy static
pub static mut HPET_COUNTER_VALUE: *const MainCounterValueRegister = core::ptr::null();
pub static mut HPET_CLOCK_PERIOD_IN_NS: u64 = 0;
