use crate::util::in_port_b;
use crate::util::out_port_b;

const VGA_ADDRESS: u64 = 0xB8000;
const REGION0: u64 = 0xA0000;
const REGION1: u32 = 0xA0000;
const REGION2: u32 = 0xB0000;
const REGION3: u32 = 0xB8000;

const VGA_MEM_ADDR: u64 = REGION0;

const VGA_AC_INDEX: u32 = 0x3C0;
const VGA_AC_WRITE: u32 = 0x3C0;
const VGA_AC_READ: u32 = 0x3C1;
const VGA_INSTAT_READ: u32 = 0x3DA;
const VGA_MISC_WRITE: u32 = 0x3C2;
const VGA_MISC_READ: u32 = 0x3CC;

/*COLOR emulation	 MONO emulation */
const VGA_CRTC_INDEX: u32 = 0x3D4; /* 0x3B4 */
const VGA_CRTC_DATA: u32 = 0x3D5; /* 0x3B5 */
const VGA_GC_INDEX: u32 = 0x3CE;
const VGA_GC_DATA: u32 = 0x3CF;
const VGA_SEQ_INDEX: u32 = 0x3C4;
const VGA_SEQ_DATA: u32 = 0x3C5;

const VGA_PALETTE_MASK: u32 = 0x3C6;
const VGA_PALETTE_READ: u32 = 0x3C7;
const VGA_PALETTE_WRITE: u32 = 0x3C8;
const VGA_PALETTE_DATA: u32 = 0x3C9;

const VGA_NUM_AC_REGS: u32 = 21;
const VGA_NUM_CRTC_REGS: u32 = 25;
const VGA_NUM_GC_REGS: u32 = 9;
const VGA_NUM_SEQ_REGS: u32 = 5;

const VGA_SCREEN_WIDTH: u32 = 320;
const VGA_SCREEN_HEIGHT: u32 = 200;
const VGA_SCREEN_SIZE: u32 = 320 * 200;

// migrated from https://github.com/pagekey/pkos/blob/vid/os015/src/vga/vga.c#L93-L146
pub fn vga_write_regs() {
    let mut pos = 0;

    let mut G_320x200x256: [u8; 61] = [
        /* MISC */
        0x63, /* SEQ */
        0x03, 0x01, 0x0F, 0x00, 0x0E, /* CRTC */
        0x5F, 0x4F, 0x50, 0x82, 0x54, 0x80, 0xBF, 0x1F, 0x00, 0x41, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x9C, 0x0E, 0x8F, 0x28, 0x40, 0x96, 0xB9, 0xA3, 0xFF, /* GC */
        0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x05, 0x0F, 0xFF, /* AC */
        0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
        0x0F, 0x41, 0x00, 0x0F, 0x00, 0x00,
    ];

    /* write MISCELLANEOUS reg */
    out_port_b(VGA_MISC_WRITE, G_320x200x256[pos]);
    pos += 1;

    /* write SEQUENCER regs */
    for i in 0..VGA_NUM_SEQ_REGS as u8 {
        out_port_b(VGA_SEQ_INDEX, i);
        out_port_b(VGA_SEQ_DATA, G_320x200x256[pos]);
        pos += 1;
    }

    /* unlock CRTC registers */
    out_port_b(VGA_CRTC_INDEX, 0x03);
    out_port_b(VGA_CRTC_DATA, in_port_b(VGA_CRTC_DATA) | 0x80);
    out_port_b(VGA_CRTC_INDEX, 0x11);
    out_port_b(VGA_CRTC_DATA, in_port_b(VGA_CRTC_DATA) & !0x80);

    /* make sure they remain unlocked */
    G_320x200x256[0x03] |= 0x80;
    G_320x200x256[0x11] &= !0x80;

    /* write CRTC regs */
    for i in 0..VGA_NUM_CRTC_REGS as u8 {
        out_port_b(VGA_CRTC_INDEX, i);
        out_port_b(VGA_CRTC_DATA, G_320x200x256[pos]);
        pos += 1;
    }

    /* write GRAPHICS CONTROLLER regs */
    for i in 0..VGA_NUM_GC_REGS as u8 {
        out_port_b(VGA_GC_INDEX, i);
        out_port_b(VGA_GC_DATA, G_320x200x256[pos]);
        pos += 1;
    }

    /* write ATTRIBUTE CONTROLLER regs */
    for i in 0..VGA_NUM_AC_REGS as u8 {
        in_port_b(VGA_INSTAT_READ);
        out_port_b(VGA_AC_INDEX, i);
        out_port_b(VGA_AC_WRITE, G_320x200x256[pos]);
        pos += 1;
    }

    /* lock 16-color palette and unblank display */
    in_port_b(VGA_INSTAT_READ);
    out_port_b(VGA_AC_INDEX, 0x20);
}

pub fn vga_clear_screen() {
    for i in 0..320 {
        for j in 0..200 {
            vga_plot_pixel(i, j, 0x2);
        }
    }
}

pub fn vga_plot_pixel(x: u64, y: u64, color: u8) {
    let offset = x + 320 * y;

    unsafe {
        core::ptr::write_volatile(
            (0xffff80003fc00000 + VGA_MEM_ADDR + offset) as *mut u16,
            color as u16,
        );
    }
}
