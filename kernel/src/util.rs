use core::arch::asm;
use tracing::instrument;

pub fn out_port_b(port: u32, value: u8) {
    unsafe {
        asm!(
            r#"out %al, %dx"#,
            in("edx") port,
            in("al") value,
            options(att_syntax)
        );
    }
}

pub fn in_port_b(port: u32) -> u8 {
    let mut key;
    unsafe {
        asm!("in al, dx", out("al") key, in("rdx") port);
    }
    return key;
}

#[instrument(fields(fid = 30))]
pub fn compare_str_to_memory(s: &str, addr: usize) -> bool {
    let bytes = s.as_bytes();
    let ptr = addr as *const u8;

    unsafe {
        for i in 0..bytes.len() {
            if *ptr.add(i) != bytes[i] {
                return false; // Mismatch found
            }
        }
    }

    true // All bytes match
}
