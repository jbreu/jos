// https://github.com/scprogramming/Jazz2.0/blob/main/src/interrupts/idt.c
// https://wiki.osdev.org/Interrupts_Tutorial

use crate::DEBUG;
use crate::ERROR;
use crate::USERLAND;
use crate::keyboard;
use crate::kprint;
use crate::profiling;
use crate::userland;
use crate::util::out_port_b;
use core::arch::asm;
use core::arch::global_asm;
use tracing::instrument;

global_asm!(include_str!("interrupt.S"));

#[unsafe(no_mangle)]
pub static mut SCHEDULING_BLOCKED: u8 = 0;

#[repr(C, packed(2))]
#[derive(Clone, Copy)]
struct IdtEntryStruct {
    base_low: u16,
    sel: u16,
    always0: u8,
    flags: u8,
    base_mid: u16,
    base_high: u32,
    reserved: u32,
}

#[repr(C)]
#[repr(packed(2))]
pub struct IdtPtrStruct {
    pub limit: u16,
    pub base: u64,
}

static mut IDT_ENTRIES: [IdtEntryStruct; 256] = [IdtEntryStruct {
    always0: 0,
    base_high: 0,
    base_mid: 0,
    base_low: 0,
    flags: 0,
    sel: 0,
    reserved: 0,
}; 256];

#[unsafe(no_mangle)]
#[instrument(skip(error_code, int_no))]
pub extern "C" fn isr_handler(error_code: u64, int_no: u64) {
    match int_no as u64 {
        0..=31 => {
            ERROR!("ISR {} error_code {:x?}", int_no, error_code);
            ERROR!("{}", CPU_EXCEPTIONS[int_no as usize]);
        }
        _ => DEBUG!("ISR {}", int_no),
    };

    out_port_b(0x20, 0x20);
}

#[unsafe(no_mangle)]
// #[instrument(skip(int_no))]
pub extern "C" fn irq_handler(int_no: u64) {
    // TODO make this a verbose log
    /*extern "C" {
        static mut stack_frame: *const u64;
    }

    unsafe {
        kprint!("Stack frame: {:x}\n", stack_frame as u64);
        kprint!(" RIP: {:x}\n", *(stack_frame.add(0)) as u64);
        kprint!(" RSP: {:x}\n", *(stack_frame.add(3)) as u64);

        let stack = *(stack_frame.add(3)) as *const u64;
        for i in 0..8 {
            kprint!("   {:x}", stack.add(i).read_unaligned() as u64);
        }
        kprint!("\n");
    }*/

    match (int_no - 32) as u64 {
        // Clock
        0 => unsafe {
            if SCHEDULING_BLOCKED == 0 {
                userland::schedule();

                //time::update_clock();
                kprint::kprint_integer_at_pos(
                    USERLAND.lock().get_current_process_id() as i64,
                    1,
                    70,
                    kprint::Colors::KPrintColorDarkGray,
                );
            }
        },
        // Keyboard action
        1 => {
            let mut scancode: i8;

            unsafe {
                asm!("in al, dx", out("al") scancode, in("rdx") 0x60);
            }

            let key = keyboard::get_key_for_scancode(scancode as u8);

            kprint!("{}", key);

            let lcontrol: char = 0x1d as char;

            unsafe {
                match key {
                    'w' => keyboard::KEYSTATES[0] = true,
                    'a' => keyboard::KEYSTATES[1] = true,
                    's' => keyboard::KEYSTATES[2] = true,
                    'd' => keyboard::KEYSTATES[3] = true,
                    ' ' => keyboard::KEYSTATES[5] = true,
                    'l' => profiling::log_tracepoints(),
                    _ if key == lcontrol => keyboard::KEYSTATES[4] = true,
                    _ if key == '\n' => keyboard::KEYSTATES[6] = true,
                    _ => {}
                }
            }

            userland::schedule();
        }
        _ => {}
    }

    // TODO make this a verbose log
    /*unsafe {
        kprint!("Stack frame: {:x}\n", stack_frame as u64);
        kprint!(" RIP: {:x}\n", *(stack_frame.add(0)) as u64);
        kprint!(" RSP: {:x}\n", *(stack_frame.add(3)) as u64);

        let stack = *(stack_frame.add(3)) as *const u64;
        for i in 0..8 {
            kprint!("   {:x}", stack.add(i).read_unaligned() as u64);
        }
        kprint!("\n");
    }*/

    if int_no >= 40 {
        out_port_b(0xA0, 0x20);
    }
    out_port_b(0x20, 0x20);
}

#[instrument(fields(fid = 80))]
fn set_idt_gate(num: usize, base: u64, sel: u16, flags: u8) {
    unsafe {
        IDT_ENTRIES[num].base_low = (base & 0xFFFF) as u16;
        IDT_ENTRIES[num].base_mid = ((base >> 16) & 0xFFFF) as u16;
        IDT_ENTRIES[num].base_high = ((base >> 32) & 0xFFFFFFFF) as u32;
        IDT_ENTRIES[num].sel = sel;
        IDT_ENTRIES[num].always0 = 0;
        IDT_ENTRIES[num].flags = flags; // | 0x60;
        IDT_ENTRIES[num].reserved = 0;
    }
}

#[instrument(fields(fid = 81))]
pub fn init_idt() {
    // https://www.eeeguide.com/8259-programmable-interrupt-controller/
    // https://stackoverflow.com/a/283033
    //0x20 commands and 0x21 data
    //0xA0 commands and 0xA1 data

    out_port_b(0x20, 0x11);
    out_port_b(0xA0, 0x11);

    out_port_b(0x21, 0x20);
    out_port_b(0xA1, 0x28);

    out_port_b(0x21, 0x04);
    out_port_b(0xA1, 0x02);

    out_port_b(0x21, 0x01);
    out_port_b(0xA1, 0x01);

    out_port_b(0x21, 0x0);
    out_port_b(0xA1, 0x0);

    // Set PIC mask to only let keyboard irqs through
    // https://wiki.osdev.org/I_Can%27t_Get_Interrupts_Working#IRQ_problems
    //out_port_b(0x21, 0xfd);
    //out_port_b(0xA1, 0xff);

    // flags are set according to https://wiki.osdev.org/Interrupt_Descriptor_Table#Gate_Descriptor_2
    // TODO: Check Gate Type setting: https://wiki.osdev.org/Interrupt_Descriptor_Table#Gate_Types
    macro_rules! set_isr {
        ($id:expr, $isr:ident) => {
            unsafe extern "C" {
                fn $isr();
            }
            let $isr: unsafe extern "C" fn() = $isr;
            let addr = $isr as u64;
            set_idt_gate($id, addr, 0x08, 0x8e);
        };
    }

    // Manually list each ISR and IRQ function with its corresponding ID
    set_isr!(0, isr0);
    set_isr!(1, isr1);
    set_isr!(2, isr2);
    set_isr!(3, isr3);
    set_isr!(4, isr4);
    set_isr!(5, isr5);
    set_isr!(6, isr6);
    set_isr!(7, isr7);
    set_isr!(8, isr8);
    set_isr!(9, isr9);
    set_isr!(10, isr10);
    set_isr!(11, isr11);
    set_isr!(12, isr12);
    set_isr!(13, isr13);
    set_isr!(14, isr14);
    set_isr!(15, isr15);
    set_isr!(16, isr16);
    set_isr!(17, isr17);
    set_isr!(18, isr18);
    set_isr!(19, isr19);
    set_isr!(20, isr20);
    set_isr!(21, isr21);
    set_isr!(22, isr22);
    set_isr!(23, isr23);
    set_isr!(24, isr24);
    set_isr!(25, isr25);
    set_isr!(26, isr26);
    set_isr!(27, isr27);
    set_isr!(28, isr28);
    set_isr!(29, isr29);
    set_isr!(30, isr30);
    set_isr!(31, isr31);

    set_isr!(32, irq0);
    set_isr!(33, irq1);
    set_isr!(34, irq2);
    set_isr!(35, irq3);
    set_isr!(36, irq4);
    set_isr!(37, irq5);
    set_isr!(38, irq6);
    set_isr!(39, irq7);
    set_isr!(40, irq8);
    set_isr!(41, irq9);
    set_isr!(42, irq10);
    set_isr!(43, irq11);
    set_isr!(44, irq12);
    set_isr!(45, irq13);
    set_isr!(46, irq14);
    set_isr!(47, irq15);

    set_isr!(128, isr128);
    set_isr!(177, isr177);

    unsafe {
        let idt_ptr: IdtPtrStruct = IdtPtrStruct {
            limit: 128 * 256 - 1, //(core::mem::size_of::<IdtEntryStruct>() * 256 - 1) as u16,
            //https://stackoverflow.com/a/64311274
            // https://github.com/rust-osdev/x86_64/blob/master/src/addr.rs#L100C9-L100C9
            // Complexity from last link probably not required
            base: core::ptr::addr_of!(IDT_ENTRIES) as u64, //(((IDT_ENTRIES.as_ptr() as u64) << 16) as i64 >> 16) as u64,
        };
        SCHEDULING_BLOCKED = 1;
        asm!(
            "lidt [{}]
            sti",
            in(reg) &idt_ptr, options(readonly, nostack, preserves_flags)
        );
    }
}

static CPU_EXCEPTIONS: [&str; 32] = [
    "Division By Zero",
    "Debug",
    "Non Maskable Interrupt",
    "Breakpoint",
    "Into Detected Overflow",
    "Out of Bounds",
    "Invalid Opcode",
    "No Coprocessor",
    "Double fault",
    "Coprocessor Segment Overrun",
    "Invalid TSS",
    "Segment not present",
    "Stack fault",
    "General protection fault",
    "Page fault",
    "Unknown Interrupt",
    "Coprocessor Fault",
    "Alignment Fault",
    "Machine Check",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
    "Reserved",
];
