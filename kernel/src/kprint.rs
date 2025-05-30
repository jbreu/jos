use crate::serial;
// add better formatting options, see https://os.phil-opp.com/vga-text-mode/#a-kprintln-macro

#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Colors {
    KPrintColorBlack = 0,
    KPrintColorBlue = 1,
    KPrintColorGreen = 2,
    KPrintColorCyan = 3,
    KPrintColorRed = 4,
    KPrintColorMagenta = 5,
    KPrintColorBrown = 6,
    KPrintColorLightGray = 7,
    KPrintColorDarkGray = 8,
    KPrintColorLightBlue = 9,
    KPrintColorLightGreen = 10,
    KPrintColorLightCyan = 11,
    KPrintColorLightRed = 12,
    KPrintColorPink = 13,
    KPrintColorYellow = 14,
    KPrintColorWhite = 15,
}

fn get_video_byte_string(character: char, foreground: Colors, background: Colors) -> u16 {
    let backgroundu8 = background as u16;
    let foregroundu8 = foreground as u16;
    return (backgroundu8 << 4 | foregroundu8) << 8 | character as u16;
}

// TODO handle those module global variables in rusty way
static mut CURRENT_ROW: u64 = 0;
static mut CURRENT_COL: u64 = 0;

pub struct KPrinter {
    pub color: Colors,
}

impl KPrinter {
    pub fn new(color: Colors) -> Self {
        Self { color: color }
    }
}

impl core::fmt::Write for KPrinter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        crate::kprint::kprint(s, self.color);
        Ok(())
    }
}
#[macro_export]
macro_rules! kprint_internal {
    ($color:expr, $with_newline:expr, $($arg:tt)*) => {{
        let mut kprinter = crate::kprint::KPrinter { color: $color };
        core::fmt::write(&mut kprinter, core::format_args!($($arg)*)).unwrap();
        if $with_newline {
            crate::kprint::kprint_char('\n', $color);
        }
    }};
}

#[macro_export]
macro_rules! kprintlncolor {
    ($color:expr, $($arg:tt)*) => {
        kprint_internal!($color, true, $($arg)*)
    };
    () => {
        crate::kprint::kprint_char('\n', $color)
    };
}

#[macro_export]
macro_rules! kprintcolor {
    ($color:expr, $($arg:tt)*) => {
        crate::kprint_internal!($color, false, $($arg)*)
    };
    () => {
        crate::kprint::kprint_char('\n')
    };
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {
        crate::kprint_internal!(crate::kprint::Colors::KPrintColorBlack, true, $($arg)*)
    };
    () => {
        crate::kprint::kprint_char('\n')
    };
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        crate::kprint_internal!(crate::kprint::Colors::KPrintColorBlack, false, $($arg)*)
    };
    () => {
        crate::kprint::kprint_char('\n')
    };
}

#[macro_export]
macro_rules! clear_console {
    () => {
        crate::kprint::clear()
    };
}

pub fn clear() {
    for column in 0..80 {
        for row in 0..25 {
            unsafe {
                // https://en.wikipedia.org/wiki/VGA_text_mode
                core::ptr::write_volatile(
                    (0xffff80003fc00000 + 0xb8000 + (row * 80 + column) * 2 as u64) as *mut u16,
                    get_video_byte_string(' ', Colors::KPrintColorBlack, Colors::KPrintColorWhite),
                );
            }
        }
    }
}

fn scroll_line() {
    for column in 0..80 {
        for row in 0..24 {
            // Exception for clock and current process
            if row <= 1 && column >= 70 {
                continue;
            }
            unsafe {
                core::ptr::write_volatile(
                    (0xffff80003fc00000 + 0xb8000 + (row * 80 + column) * 2 as u64) as *mut u16,
                    core::ptr::read_volatile(
                        (0xffff80003fc00000 + 0xb8000 + ((row + 1) * 80 + column) * 2) as *mut u16,
                    ),
                );
            }
        }
    }

    // clear bottom line
    for column in 0..80 {
        unsafe {
            core::ptr::write_volatile(
                (0xffff80003fc00000 + 0xb8000 + (24 * 80 + column) * 2 as u64) as *mut u16,
                get_video_byte_string(' ', Colors::KPrintColorBlack, Colors::KPrintColorWhite),
            );
        }
    }

    unsafe {
        CURRENT_COL = 0;
        CURRENT_ROW = 0;
    }
}

pub fn kprint(text: &str, color: Colors) {
    for character in text.chars() {
        kprint_char(character, color);
    }
}

pub fn _kprint_text_at_pos(text: &str, row: u64, column: u64, color: Colors) {
    let mut i = 0;
    for character in text.chars() {
        kprint_char_at_pos(character, row, column + i, color);
        i += 1;
    }
}

pub fn kprint_char_at_pos(character_in: char, row: u64, column: u64, color: Colors) {
    // TODO remove the unsafe
    unsafe {
        let old_row = CURRENT_ROW;
        let old_column = CURRENT_COL;

        CURRENT_ROW = row;
        CURRENT_COL = column;

        kprint_char(character_in, color);

        CURRENT_COL = old_column;
        CURRENT_ROW = old_row;
    }
}

pub fn kprint_char(character_in: char, color: Colors) {
    let mut character = character_in;

    match character as u8 {
        0x20..=0x7e | b'\n' => (),
        _ => character = 0xfe as char,
    }

    unsafe {
        if character == '\n' {
            serial::write_serial('\r');
            serial::write_serial('\n');

            CURRENT_ROW += 1;

            if CURRENT_ROW > 24 {
                scroll_line();
                CURRENT_ROW = 24;
            }

            CURRENT_COL = 0;
            return;
        }

        // Write to serial port first
        serial::write_serial(character);

        // https://en.wikipedia.org/wiki/VGA_text_mode
        core::ptr::write_volatile(
            (0xffff80003fc00000 + 0xb8000 + (CURRENT_COL + CURRENT_ROW * 80) * 2) as *mut u16,
            get_video_byte_string(character, color, Colors::KPrintColorWhite),
        );

        if CURRENT_COL == 80 {
            CURRENT_COL = 0;
            CURRENT_ROW += 1;

            if CURRENT_ROW > 24 {
                scroll_line();
                CURRENT_ROW = 24;
            }
        }
        CURRENT_COL += 1;
    }
}

pub fn kprint_integer(number: i64, color: Colors) {
    if number > 10 {
        kprint_integer(number / 10, color);
    }
    kprint_char((number % 10 + 0x30) as u8 as char, color);
}

pub fn kprint_integer_at_pos(number: i64, row: u64, column: u64, color: Colors) {
    // TODO remove the unsafe
    unsafe {
        let old_row = CURRENT_ROW;
        let old_column = CURRENT_COL;

        CURRENT_ROW = row;
        CURRENT_COL = column;

        if number > 10 {
            kprint_integer_at_pos(number / 10, row, column, color);
            CURRENT_COL = column + 1;
        }
        kprint_char((number % 10 + 0x30) as u8 as char, color);

        CURRENT_COL = old_column;
        CURRENT_ROW = old_row;
    }
}
