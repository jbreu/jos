use core::str;
use core::sync::atomic::{AtomicPtr, AtomicU64, Ordering};

use crate::DEBUG;
use crate::mem_config::PHYSICAL_MEMORY_BASE;

use crate::util::compare_str_to_memory;

// TODO better use lazy static
pub static HPET_COUNTER_VALUE_ADDRESS: AtomicPtr<u64> = AtomicPtr::new(0 as *mut u64);
pub static HPET_CLOCK_PERIOD_IN_NS: AtomicU64 = AtomicU64::new(0);

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
    hardware_rev_id: u8,
    cluster_8_bits: u8, // attention, mind the colons in the wiki, this is a bitfield
    pci_vendor_id: u16,
    address_space_id: u8,
    register_bit_width: u8,
    register_bit_offset: u8,
    reserved: u8,
    address: u64,
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

fn find_xsdp() -> *const XsdpT {
    let ebda_address = (PHYSICAL_MEMORY_BASE + 0x040E) as *const u64;
    let bios_rom_start = (PHYSICAL_MEMORY_BASE + 0xE0000) as u64;
    let bios_rom_end = (PHYSICAL_MEMORY_BASE + 0xFFFFF) as u64;

    // Search in EBDA
    let ebda_seg = unsafe { ebda_address.read_unaligned() as u16 };
    let ebda = PHYSICAL_MEMORY_BASE + ((ebda_seg as usize) << 4);
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
    let offset = PHYSICAL_MEMORY_BASE;

    unsafe {
        if (*xsdp).revision == 0 {
            let virt_rsdt_address;

            virt_rsdt_address = ((*xsdp).rsdt_address as usize) + offset;

            DEBUG!("RSDT Address: {:?}", virt_rsdt_address as *const u64);

            let rsdt = virt_rsdt_address as *const ACPISDTHeader;

            DEBUG!("RSDT: {:?}", str::from_utf8(&(*rsdt).signature));

            let entries = ((*rsdt).length as usize - core::mem::size_of::<ACPISDTHeader>()) / 4;

            DEBUG!("RSDT entries: {}", entries);

            // The individual tables are pointed to 32bit pointers coming after the header
            let table_ptrs =
                (virt_rsdt_address as usize + core::mem::size_of::<ACPISDTHeader>()) as usize;

            DEBUG!(
                "RSDT table_ptrs: {:?} (size of ACPISDTHeader: {:?})",
                table_ptrs,
                core::mem::size_of::<ACPISDTHeader>()
            );

            for i in 0..entries {
                let header = core::ptr::read_unaligned((table_ptrs + i * 4) as *const usize) >> 32;
                let virt_header = ((header as usize) + offset) as *const ACPISDTHeader;

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
    let offset = PHYSICAL_MEMORY_BASE;

    unsafe {
        let capabilities =
            (((*hpet).address as usize) + offset) as *const GeneralCapabilitiesAndIdRegister;

        let frequency = 10_u64.pow(15) / (*capabilities).counter_clk_period as u64;
        DEBUG!("frequency: {}", frequency);

        HPET_CLOCK_PERIOD_IN_NS.store(
            ((*capabilities).counter_clk_period / 1_000_000) as u64,
            Ordering::Relaxed,
        );

        let configuration =
            (((*hpet).address as usize + 0x10) + offset) as *mut GeneralConfigurationRegister;

        // Enable HPET
        (*configuration).config = 0x1;

        let hpet_counter_value_address = (((*hpet).address + 0xf0) as usize) + offset;

        HPET_COUNTER_VALUE_ADDRESS.store(hpet_counter_value_address as *mut u64, Ordering::Relaxed);
    }
}
