use tracing::instrument;

// http://kbdlayout.info/KBDGR/scancodes+names
// http://kbdlayout.info/KBDGR/virtualkeys
static SCANCODES: [char; 69] = [
    0xfe as char,
    0xfe as char,
    '1',
    '2',
    '3',
    '4',
    '5',
    '6',
    '7',
    '8',
    '9',
    '0',
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    'q',
    'w',
    'e',
    'r',
    't',
    'z',
    'u',
    'i',
    'o',
    'p',
    0xfe as char,
    0xfe as char,
    '\n',         //VK_RETURN -> map to ascii line feed
    0x1d as char, //VK_LCONTROL
    'a',
    's',
    'd',
    'f',
    'g',
    'h',
    'j',
    'k',
    'l',
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    'y',
    'x',
    'c',
    'v',
    'b',
    'n',
    'm',
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    ' ', //VK_SPACE -> map to whitespace
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
    0xfe as char,
];

pub static mut KEYSTATES: [bool; 10] = [false; 10];

#[instrument]
pub fn get_key_for_scancode(scancode: u8) -> char {
    match scancode as u8 {
        0..=68 => SCANCODES[scancode as usize],
        _ => 0xfe as char,
    }
}
