#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
use lazy_static::lazy_static;
use spin::Mutex;

mod acpi;
mod filesystem;
mod gdt;
mod hdd;
mod heap;
mod interrupt;
mod keyboard;
mod kprint;
mod logging;
mod mem;
mod mem_config;
mod process;
mod profiling;
mod serial;
mod syscall;
mod time;
mod userland;
mod util;
mod vga;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ERROR!("Kernel Panic!");

    ERROR!("{}", info);

    loop {}
}

lazy_static! {
    pub static ref USERLAND: Mutex<userland::Userland> = Mutex::new(userland::Userland::new());
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() {
    mem::init_available_memory();

    clear_console!();
    DEBUG!("Entering JOS Kernel");

    time::set_initial_time();
    DEBUG!("Initialized High Precision Event Timer");

    serial::init_serial();
    DEBUG!("Initialized Serial Port");

    heap::init_kernel_heap();
    DEBUG!("Initialized Kernel Heap Memory");

    gdt::init_gdt();
    DEBUG!("Initialized Global Descriptor Table");

    interrupt::init_idt();
    DEBUG!("Initialized Interrupt Descriptor Table");

    filesystem::init_filesystem();
    DEBUG!("Initialized Filesystem");

    //vga::vga_enter();
    //vga::vga_clear_screen();

    // Trigger test exception
    //unsafe {
    //    asm!("int3", options(nomem, nostack));
    //}

    kprint!(
        r#"
        ,--.-,     _,.---._        ,-,--.  
        |==' -|   ,-.' , -  `.    ,-.'-  _\ 
        |==|- |  /==/_,  ,  - \  /==/_ ,_.' 
      __|==|, | |==|   .=.     | \==\  \    
   ,--.-'\=|- | |==|_ : ;=:  - |  \==\ -\   
   |==|- |=/ ,| |==| , '='     |  _\==\ ,\  
   |==|. /=| -|  \==\ -    ,_ /  /==/\/ _ | 
   \==\, `-' /    '.='. -   .'   \==\ - , / 
    `--`----'       `--`--''      `--`---'    
    "#
    );

    kprint!("JOS by Jakob Breu");

    if let Some(builddate) = option_env!("VERGEN_BUILD_DATE") {
        kprint!("; build date {}", builddate);
    }

    kprintln!("");

    DEBUG!("JOS Kernel initialized; switching to userland");

    USERLAND.lock().switch_to_userland(&USERLAND);

    panic!("This should never happen!?");
}
