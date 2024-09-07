use crate::{
    kprint::{kprint_char, kprint_char_at_pos, kprint_integer, kprint_integer_at_pos},
    util::out_port_b,
};
use core::arch::asm;

#[allow(dead_code)]
#[derive(PartialEq, Clone)]
enum CmosRegister {
    Seconds = 0x00,
    Minutes = 0x02,
    Hours = 0x04,
    Weekday = 0x06,
    DayOfMonth = 0x07,
    Month = 0x08,
    Year = 0x09,
    StatusA = 0x0a,
    StatusB = 0x0b,
}

// https://wiki.osdev.org/CMOS#Accessing_CMOS_Registers
fn read_cmos_i16(register: CmosRegister, bcd_enabled: bool) -> i16 {
    unsafe {
        asm!(
            r#"out %al, %dx"#,
            in("dx") 0x70 as i16,
            in("al") register.clone() as u8,
            options(att_syntax)
        );

        let mut ret: i8;

        asm!(
            r#"in %dx, %al"#,
            in("dx") 0x71 as i16,
            out("al") ret,
            options(att_syntax)
        );

        if bcd_enabled && register != CmosRegister::StatusA {
            return ((ret as i16 & 0xF0) >> 1) + ((ret as i16 & 0xF0) >> 3) + (ret as i16 & 0xf);
        } else {
            return ret as i16;
        }
    }
}

// https://github.com/sphaerophoria/stream-os/blob/master/src/io/io_allocator.rs#L67
// https://stackoverflow.com/a/64818139
pub fn kprint_time() {
    let bcd_enabled: bool = read_cmos_i16(CmosRegister::StatusA, false) != 0;

    let hours: i16 = read_cmos_i16(CmosRegister::Hours, bcd_enabled);
    let minutes: i16 = read_cmos_i16(CmosRegister::Minutes, bcd_enabled);
    let seconds: i16 = read_cmos_i16(CmosRegister::Seconds, bcd_enabled);

    kprint_integer(hours.into());
    kprint_char(':');
    kprint_integer(minutes.into());
    kprint_char(':');
    kprint_integer(seconds.into());
}

static mut MICROSECONDS_SINCE_BOOT: u64 = 0;
static mut INITIAL_HOURS: i16 = 0;
static mut INITIAL_MINUTES: i16 = 0;
static mut INITIAL_SECONDS: i16 = 0;

pub fn update_microsecond_counter() {
    unsafe {
        MICROSECONDS_SINCE_BOOT += 10000;
        //VERBESSERuNG FÜR HÖHERE FREQUENZ: WENN BEIM UPDATE FESTGESTSLLT WIRD DASS DIE UHRZEIT (SEKUNDEN) UND DIE MIKROSEKUNDEN NICHT ZUSAMMENPASSEN --> KORRIGIERE MICROSECONDS_SINCE_BOOT
    }
}

pub fn get_microsecond_counter() -> u64 {
    unsafe { MICROSECONDS_SINCE_BOOT }
}

pub fn set_initial_time() {
    let bcd_enabled: bool = read_cmos_i16(CmosRegister::StatusA, false) != 0;

    unsafe {
        INITIAL_HOURS = read_cmos_i16(CmosRegister::Hours, bcd_enabled);
        INITIAL_MINUTES = read_cmos_i16(CmosRegister::Minutes, bcd_enabled);
        INITIAL_SECONDS = read_cmos_i16(CmosRegister::Seconds, bcd_enabled);
    }
}

pub fn get_time() -> (u32, u32) {
    let usec_since_boot = unsafe { MICROSECONDS_SINCE_BOOT };

    (
        ((usec_since_boot / 1000000 + unsafe { INITIAL_SECONDS } as u64) % 60) as u32,
        (usec_since_boot % 1000000) as u32,
    )
}

pub fn update_clock() {
    let bcd_enabled: bool = read_cmos_i16(CmosRegister::StatusA, false) != 0;

    let hours: i16 = read_cmos_i16(CmosRegister::Hours, bcd_enabled);
    let minutes: i16 = read_cmos_i16(CmosRegister::Minutes, bcd_enabled);
    let seconds: i16 = read_cmos_i16(CmosRegister::Seconds, bcd_enabled);

    kprint_integer_at_pos(hours.into(), 0, 70);
    kprint_char_at_pos(':', 0, 72);
    kprint_integer_at_pos(minutes.into(), 0, 73);
    kprint_char_at_pos(':', 0, 75);
    kprint_integer_at_pos(seconds.into(), 0, 76);
}

static PIC1_COMMAND: u32 = 0x20;
static PIC1_DATA: u32 = 0x21;
static PIC2_COMMAND: u32 = 0xA0;
static PIC2_DATA: u32 = 0xA1;
static PIT_COMMAND: u32 = 0x43;
static PIT_CHANNEL0: u32 = 0x40;

fn init_pic() {
    // Initialize PIC1
    out_port_b(PIC1_COMMAND, 0x11); // Start initialization sequence
    out_port_b(PIC1_DATA, 0x20); // ICW2: Master PIC vector offset
    out_port_b(PIC1_DATA, 0x04); // ICW3: Tell Master PIC there is a slave PIC at IRQ2
    out_port_b(PIC1_DATA, 0x01); // ICW4: 8086/88 mode

    // Initialize PIC2
    out_port_b(PIC2_COMMAND, 0x11); // Start initialization sequence
    out_port_b(PIC2_DATA, 0x28); // ICW2: Slave PIC vector offset
    out_port_b(PIC2_DATA, 0x02); // ICW3: Tell Slave PIC its cascade identity
    out_port_b(PIC2_DATA, 0x01); // ICW4: 8086/88 mode

    // Unmask all interrupts
    out_port_b(PIC1_DATA, 0x0);
    out_port_b(PIC2_DATA, 0x0);
}

pub fn init_timer(frequency: usize) {
    init_pic();

    // Calculate the divisor for the desired frequency
    let divisor = 1193180 / frequency;

    // Send the command byte
    out_port_b(PIT_COMMAND, 0x36);

    // Send the frequency divisor
    out_port_b(PIT_CHANNEL0, (divisor & 0xFF) as u8); // Low byte
    out_port_b(PIT_CHANNEL0, ((divisor >> 8) & 0xFF) as u8); // High byte
}
