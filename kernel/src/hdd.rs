use crate::{DEBUG, ERROR, util::*};

static ATA_PRIMARY_BASE: u32 = 0x1F0;
static ATA_PRIMARY_CTRL: u32 = 0x3F6;

// Status register bits
static ATA_STATUS_BSY: u8 = 0x80;
static ATA_STATUS_DRQ: u8 = 0x08;
static ATA_STATUS_ERR: u8 = 0x01;

// Commands
static ATA_CMD_READ: u8 = 0x20;
static ATA_CMD_WRITE: u8 = 0x30;

pub static LBA_SECTOR_SIZE: usize = 512;

pub fn ata_pio_read_sector(base: u32, lba: u32, buffer: &mut [u8], skip: usize) {
    let _event = core::hint::black_box(crate::instrument!());

    // ensure we read all data from the sector
    while (in_port_b(base + 7) & ATA_STATUS_BSY) != 0 {}

    // Select the drive and set LBA
    out_port_b(base + 6, (0xE0 | ((lba >> 24) & 0x0F)) as u8); // Drive/Head
    out_port_b(base + 2, 1); // Sector count
    out_port_b(base + 3, (lba & 0xFF) as u8); // LBA low
    out_port_b(base + 4, ((lba >> 8) & 0xFF) as u8); // LBA mid
    out_port_b(base + 5, ((lba >> 16) & 0xFF) as u8); // LBA high

    // Send READ command
    out_port_b(base + 7, ATA_CMD_READ);

    let mut pos = 0;

    loop {
        let status = in_port_b(base + 7);

        if status & ATA_STATUS_BSY != 0 {
            continue; // still busy
        }
        if status & ATA_STATUS_ERR != 0 {
            let err = in_port_b(base + 1);
            if err & 0x04 != 0 {
                ERROR!("ABRT: command aborted (bad LBA?)");
            }
            if err & 0x10 != 0 {
                ERROR!("IDNF: sector not found (bad LBA?)");
            }
            if err & 0x40 != 0 {
                ERROR!("UNC: uncorrectable data error");
            }
            panic!("HDD read error: status={:02x}, error={:02x}", status, err);
        }
        if status & ATA_STATUS_DRQ != 0 {
            break; // ready to transfer data
        }
    }

    // Read data (max 256 words per sector)
    // always go through the entire sector even if not all data is needed
    // TODO check if this is all correct as we are reading 256 16-bit words
    // FIXME what if skip is odd?

    //let mut cnt = 0;

    for i in 0..LBA_SECTOR_SIZE / 2 {
        //cnt += 1;
        let word = in_port_w(base);

        if i < skip / 2 {
            continue;
        }

        if pos < buffer.len() {
            buffer[pos] = word as u8;
            buffer[pos + 1] = (word >> 8) as u8;
            pos += 2;
        }
    }

    //DEBUG!("Counted {} words", cnt)
}

pub fn hdd_read(lba: u32, sector_count: u8, buffer: &mut [u8], skip: usize) {
    let _event = core::hint::black_box(crate::instrument!());

    let mut offset = 0;

    for i in 0..sector_count {
        let len;

        if i == sector_count - 1 {
            if (buffer.len() % LBA_SECTOR_SIZE) != 0 {
                len = (buffer.len() % LBA_SECTOR_SIZE);
            } else {
                len = LBA_SECTOR_SIZE;
            }
        } else {
            len = LBA_SECTOR_SIZE;
        }

        // SAFETY: We ensure the buffer is properly aligned and sized for u16 access.
        let sector_buf = unsafe {
            core::slice::from_raw_parts_mut(buffer[offset..].as_mut_ptr() as *mut u8, len)
        };
        ata_pio_read_sector(ATA_PRIMARY_BASE, lba + i as u32, sector_buf, skip);
        offset += LBA_SECTOR_SIZE;
    }
}

#[macro_export]
macro_rules! hdd_read_struct {
    ($offset:expr, $struct_type:ty) => {{
        let lba = $offset / crate::hdd::LBA_SECTOR_SIZE;
        let skip = $offset % crate::hdd::LBA_SECTOR_SIZE;

        let mut struct_bytes: [u8; core::mem::size_of::<$struct_type>()] =
            [0; core::mem::size_of::<$struct_type>()];

        let num_of_sectors =
            (struct_bytes.len() + crate::hdd::LBA_SECTOR_SIZE - 1) / crate::hdd::LBA_SECTOR_SIZE;

        crate::hdd::hdd_read(
            lba as u32,
            num_of_sectors as u8,
            struct_bytes.as_mut(),
            skip,
        );
        unsafe { core::ptr::read(struct_bytes.as_ptr() as *const $struct_type) }
    }};
}
