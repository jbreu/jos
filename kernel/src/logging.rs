// TODO add timestamps
#[macro_export]
macro_rules! DEBUG {
    () => {
        kprint("[DEBUG] ", crate::kprint::Colors::KPrintColorGreen);
        kprintln()
    };
    ($($arg:tt)*) => {{
        crate::kprint::kprint("[DEBUG] ", crate::kprint::Colors::KPrintColorGreen);
        crate::kprintln!($($arg)*);
    }};
}

#[macro_export]
macro_rules! ERROR {
    () => {
        kprint("[ERROR] ", crate::kprint::Colors::KPrintColorRed);
        kprintln()
    };
    ($($arg:tt)*) => {{
        crate::kprint::kprint("[ERROR] ", crate::kprint::Colors::KPrintColorGreen);
        crate::kprintln!($($arg)*);
    }};
}
