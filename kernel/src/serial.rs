use crate::util::{in_port_b, out_port_b};

// Standard PC COM1 port
const SERIAL_PORT: u32 = 0x3F8;

pub fn init_serial() {
    // Disable interrupts
    out_port_b(SERIAL_PORT + 1, 0x00);

    // Set baud rate to 38400
    out_port_b(SERIAL_PORT + 3, 0x80); // Enable DLAB
    out_port_b(SERIAL_PORT + 0, 0x03); // Low byte
    out_port_b(SERIAL_PORT + 1, 0x00); // High byte

    // 8 bits, no parity, one stop bit
    out_port_b(SERIAL_PORT + 3, 0x03);

    // Enable FIFO, clear it, with 14-byte threshold
    out_port_b(SERIAL_PORT + 2, 0xC7);

    // Enable interrupts, RTS/DSR set
    out_port_b(SERIAL_PORT + 4, 0x0B);
}

fn is_transmit_empty() -> bool {
    (in_port_b(SERIAL_PORT + 5) & 0x20) != 0
}

pub fn write_serial(c: char) {
    while !is_transmit_empty() {
        // Wait for the serial port to be ready
    }
    out_port_b(SERIAL_PORT, c as u8);
}
