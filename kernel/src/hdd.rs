use crate::util::*;

static ATA_PRIMARY_BASE: u32 = 0x1F0;
static ATA_PRIMARY_CTRL: u32 = 0x3F6;

// Status register bits
static ATA_STATUS_BSY: u8 = 0x80;
static ATA_STATUS_DRQ: u8 = 0x08;
static ATA_STATUS_ERR: u8 = 0x01;

// Commands
static ATA_CMD_READ: u8 = 0x20;
static ATA_CMD_WRITE: u8 = 0x30;

pub fn ata_pio_read(base: u32, lba: u32, sector_count: u8, buffer: &mut [u16]) {
    // Select the drive and set LBA
    out_port_b(base + 6, (0xE0 | ((lba >> 24) & 0x0F)) as u8); // Drive/Head (use out_port_b)
    out_port_b(base + 2, sector_count); // Sector count
    out_port_b(base + 3, (lba & 0xFF) as u8); // LBA low
    out_port_b(base + 4, ((lba >> 8) & 0xFF) as u8); // LBA mid
    out_port_b(base + 5, ((lba >> 16) & 0xFF) as u8); // LBA high

    // Send READ command
    out_port_b(base + 7, ATA_CMD_READ);

    let mut offset = 0;

    for _ in 0..sector_count {
        // Wait for DRQ
        while (in_port_b(base + 7) & ATA_STATUS_DRQ) == 0 {}

        // Read data (256 words per sector)
        for j in 0..256 {
            buffer[offset + j] = in_port_w(base);
        }

        offset += 256;
    }
}

pub fn hdd_read(lba: u32, sector_count: u8, buffer: &mut [u8]) {
    let mut offset = 0;

    for i in 0..sector_count {
        // SAFETY: We ensure the buffer is properly aligned and sized for u16 access.
        let sector_buf = unsafe {
            core::slice::from_raw_parts_mut(buffer[offset..].as_mut_ptr() as *mut u16, 256)
        };
        ata_pio_read(ATA_PRIMARY_BASE, lba + i as u32, 1, sector_buf);
        offset += 512;
    }
}

macro_rules! hdd_read_struct {
    ($offset:expr, $struct_type:ty) => {{
        let mut struct_bytes: [u8; core::mem::size_of::<$struct_type>()] =
            [0; core::mem::size_of::<$struct_type>()];
        hdd_read($offset, 1, struct_bytes.as_mut());
        unsafe { core::ptr::read(struct_bytes.as_ptr() as *const $struct_type) }
    }};
}
