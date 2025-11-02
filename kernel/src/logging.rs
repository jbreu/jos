#[macro_export]
macro_rules! log_with_level {
    ($color:expr, $level:expr, $($arg:tt)*) => {{
        crate::kprintcolor!($color, $level);
        crate::kprint!("[{}] ", crate::time::get_ms_since_boot());
        crate::kprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! DEBUG {
    ($($arg:tt)*) => {
        crate::log_with_level!(crate::kprint::Colors::KPrintColorGreen, "[DEBUG] ", $($arg)*);
    };
}

#[macro_export]
macro_rules! INFO {
    ($($arg:tt)*) => {
        crate::log_with_level!(crate::kprint::Colors::KPrintColorBlack, "[DEBUG] ", $($arg)*);
    };
}

#[macro_export]
macro_rules! ERROR {
    ($($arg:tt)*) => {
        crate::log_with_level!(crate::kprint::Colors::KPrintColorRed, "[ERROR] ", $($arg)*);
    };
}
