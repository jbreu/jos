#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

mod acpi;
mod file;
mod gdt;
mod heap;
mod interrupt;
mod keyboard;
mod kprint;
mod logging;
mod mem;
mod process;
mod syscall;
mod time;
mod userland;
mod util;
mod vga;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ERROR!("Kernel Panic!");

    ERROR!("{}", info);

    loop {}
}

lazy_static! {
    pub static ref USERLAND: Mutex<userland::Userland> = Mutex::new(userland::Userland::new());
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    heap::init_kernel_heap();
    gdt::init_gdt();
    interrupt::init_idt();
    time::set_initial_time();

    clear_console!();
    DEBUG!("successfull boot!");
    DEBUG!("Hellö Wörld!");

    //vga::vga_enter();
    //vga::vga_clear_screen();

    // Trigger test exception
    //unsafe {
    //    asm!("int3", options(nomem, nostack));
    //}

    USERLAND.lock().switch_to_userland(&USERLAND);

    panic!("This should never happen!?");
}
