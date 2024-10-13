use crate::acpi;
use crate::kprint::{kprint_char, kprint_char_at_pos, kprint_integer, kprint_integer_at_pos};
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

static mut INITIAL_HOURS: i16 = 0;
static mut INITIAL_MINUTES: i16 = 0;
static mut INITIAL_SECONDS: i16 = 0;

pub fn set_initial_time() {
    acpi::init_acpi();

    let bcd_enabled: bool = read_cmos_i16(CmosRegister::StatusA, false) != 0;

    unsafe {
        INITIAL_HOURS = read_cmos_i16(CmosRegister::Hours, bcd_enabled);
        INITIAL_MINUTES = read_cmos_i16(CmosRegister::Minutes, bcd_enabled);
        INITIAL_SECONDS = read_cmos_i16(CmosRegister::Seconds, bcd_enabled);
    }
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

pub fn get_ns_since_boot() -> u64 {
    unsafe { (*acpi::HPET_COUNTER_VALUE).main_counter_val * acpi::HPET_CLOCK_PERIOD_IN_NS }
}

pub fn get_us_since_boot() -> u64 {
    get_ns_since_boot() / 1000
}

pub fn get_ms_since_boot() -> u64 {
    get_ns_since_boot() / 1_000_000
}

pub fn get_time() -> (u32, u32) {
    (
        ((get_ms_since_boot() / 1000 + unsafe { INITIAL_SECONDS } as u64) % 60) as u32,
        (get_us_since_boot() % 1000000) as u32,
    )
}
