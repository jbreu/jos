use crate::{
    DEBUG, ERROR, INFO, filesystem::FileHandle, kprint, mem::allocate_page_frame, mem_config::*,
};
extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::arch::asm;
use core::fmt::Debug;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;
use elf::abi::PT_LOAD;
use elf::endian::AnyEndian;

pub static KERNEL_CR3: AtomicUsize = AtomicUsize::new(0);
pub static NEXT_PROCESS_ID: AtomicUsize = AtomicUsize::new(1);

// stores a process' registers when it gets interrupted
#[repr(C)]
#[derive(Default, Clone)]
struct RegistersStruct {
    // Has to be always in sync with asm macro "pop_all_registers"
    xmm7: [u64; 2],
    xmm6: [u64; 2],
    xmm5: [u64; 2],
    xmm4: [u64; 2],
    xmm3: [u64; 2],
    xmm2: [u64; 2],
    xmm1: [u64; 2],
    xmm0: [u64; 2],
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug, Clone, Copy)]
pub struct PageTable {
    pub entry: [usize; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    fn default() -> Self {
        Self {
            entry: [0; PAGE_TABLE_ENTRIES],
        }
    }
}

fn _print_page_table_tree_for_cr3() {
    let mut cr3: u64;

    unsafe {
        asm!("mov r12, cr3", out("r12") cr3);
    }

    print_page_table_tree(cr3);
}

fn print_page_table_tree(start_addr: u64) {
    let _event = core::hint::black_box(crate::instrument!());
    let entry_mask = 0x0008_ffff_ffff_f800;

    unsafe {
        kprint!("start_addr: {:#x}\n", start_addr);

        for l4_entry in 0..512 {
            let l4bits =
                *(((start_addr + l4_entry * 8) | KERNEL_HIGHER_HALF_BASE as u64) as *const u64);
            if l4bits != 0 {
                kprint!("   L4: {} - {:#x}\n", l4_entry, l4bits & entry_mask);

                for l3_entry in 0..512 {
                    let l3bits = *((((l4bits & entry_mask) + l3_entry * 8)
                        | KERNEL_HIGHER_HALF_BASE as u64)
                        as *const u64);
                    if l3bits != 0 {
                        kprint!("      L3: {} - {:#x}\n", l3_entry, l3bits & entry_mask);

                        for l2_entry in 0..512 {
                            let l2bits = *((((l3bits & entry_mask) + l2_entry * 8)
                                | KERNEL_HIGHER_HALF_BASE as u64)
                                as *const u64);

                            if l2bits != 0 {
                                kprint!("         L2: {} - {:#x}\n", l2_entry, l2bits & entry_mask);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ProcessState {
    New,
    Prepared,
    Active,
    Passive,
    Sleeping,
}

pub struct Process {
    process_id: u64,

    registers: RegistersStruct,

    l1_page_table: PageTable,
    l2_page_directory_table: PageTable,
    l3_page_directory_pointer_table: PageTable,
    l4_page_map_l4_table: PageTable,

    l1_page_table_beginning: [PageTable; 16],
    l2_page_directory_table_beginning: PageTable,
    l3_page_directory_pointer_table_beginning: PageTable,

    rip: usize,
    rsp: u64,
    cr3: usize,
    ss: u64,
    cs: u64,
    rflags: u64,

    state: ProcessState,

    heap_allocator: linked_list_allocator::LockedHeap,
    heap_l1_table_number: usize,
    heap_l2_table_number: usize,

    stack_page_counter: usize,

    working_directory: &'static str,

    file_handles: BTreeMap<u64, FileHandle>,
    next_handle_id: u64,

    parent_id: u64,
}

impl Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Process {{ state: {:?} }}", self.state)
    }
}

impl Process {
    pub fn new() -> Self {
        let _event = core::hint::black_box(crate::instrument!());

        Self {
            process_id: NEXT_PROCESS_ID.fetch_add(1, Ordering::Relaxed) as u64,

            registers: RegistersStruct::default(),
            l1_page_table: PageTable::default(),
            l1_page_table_beginning: [PageTable::default(); 16],
            l2_page_directory_table: PageTable::default(),
            l2_page_directory_table_beginning: PageTable::default(),
            l3_page_directory_pointer_table: PageTable::default(),
            l3_page_directory_pointer_table_beginning: PageTable::default(),
            l4_page_map_l4_table: PageTable::default(),

            rip: 0,
            cr3: 0,
            ss: 0x1b,
            cs: 0x23,
            rflags: 0x202,
            rsp: 0,
            state: ProcessState::New,

            heap_allocator: linked_list_allocator::LockedHeap::empty(),
            heap_l1_table_number: 0,
            heap_l2_table_number: 0,

            stack_page_counter: 0,

            working_directory: "/",
            file_handles: BTreeMap::new(),
            next_handle_id: 1,

            parent_id: 0,
        }
    }

    pub fn initialize(&mut self, file_path: &str, execve: bool) {
        let _event = core::hint::black_box(crate::instrument!());

        // reset everything (relevant if process was forked from another process)
        self.registers = RegistersStruct::default();
        self.l1_page_table = PageTable::default();
        self.l1_page_table_beginning = [PageTable::default(); 16];
        self.l2_page_directory_table = PageTable::default();
        self.l2_page_directory_table_beginning = PageTable::default();
        self.l3_page_directory_pointer_table = PageTable::default();
        self.l3_page_directory_pointer_table_beginning = PageTable::default();
        self.l4_page_map_l4_table = PageTable::default();
        self.heap_allocator = linked_list_allocator::LockedHeap::empty();
        self.file_handles = BTreeMap::new();
        self.heap_l1_table_number = 0;
        self.heap_l2_table_number = 0;
        self.stack_page_counter = 0;

        // TODO Hack? map the kernel pages from main.asm to process
        // TODO Later, the kernel pages should be restricted to superuser access; in order to do so, the process code and data must be fully in userspace pages
        unsafe {
            let mut kernel_cr3 = KERNEL_CR3.load(Ordering::Relaxed) as u64;

            if kernel_cr3 == 0 {
                let mut cr3: u64;
                asm!("mov r15, cr3", out("r15") cr3);

                KERNEL_CR3.store(cr3 as usize, Ordering::Relaxed);
                kernel_cr3 = cr3;
            } else {
                // load user stack into kernel page table
                /*let mut user_cr3: u64;
                asm!("mov r15, cr3", out("r15") user_cr3);

                *((kernel_cr3 | KERNEL_HIGHER_HALF_BASE as u64 + 255 * 8) as *mut u64) =
                    *((user_cr3 | KERNEL_HIGHER_HALF_BASE as u64 + 255 * 8) as *const u64);

                // in case we are not in the kernel page table, we need to switch to it temporarily to read the kernel page table entries
                asm!(
                    "mov cr3, r15",
                    in("r15") kernel_cr3,
                    options(nostack, preserves_flags)
                );*/
            }

            kprint!("Kernel CR3: {:x}\n", kernel_cr3);

            print_page_table_tree(KERNEL_CR3.load(Ordering::Relaxed) as u64);

            self.l4_page_map_l4_table.entry[256] =
                *((KERNEL_CR3.load(Ordering::Relaxed) + 256 * 8) as *const _);
        }

        if PAGE_SIZE == HUGE_PAGE_SIZE {
            // allocate one user stack page
            self.l2_page_directory_table.entry[511] =
                allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;

            // TODO HackID1: Fixed kernel stack for interrupts - limited to one PAGE_SIZE!
            self.l2_page_directory_table.entry[0] =
                allocate_page_frame() | PAGE_ENTRY_FLAGS_KERNELSPACE as usize;
        } else {
            // allocate 502 user stack pages (512 - 10 for kernel stack, see next loop)
            for i in 0..(512 - 10) {
                self.l1_page_table.entry[511 - i] =
                    allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;
            }

            // TODO HackID1: Fixed kernel stack for interrupts - limited to ten PAGE_SIZE!
            for i in 0..10 {
                self.l1_page_table.entry[i] =
                    allocate_page_frame() | PAGE_ENTRY_FLAGS_KERNELSPACE as usize;
            }

            self.l2_page_directory_table.entry[511] =
                Process::get_physical_address_for_virtual_address(
                    &self.l1_page_table as *const _ as usize,
                ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;
        }

        self.stack_page_counter = 1;

        self.l3_page_directory_pointer_table.entry[511] =
            Process::get_physical_address_for_virtual_address(
                &self.l2_page_directory_table as *const _ as usize,
            ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;

        self.l4_page_map_l4_table.entry[255] = Process::get_physical_address_for_virtual_address(
            &self.l3_page_directory_pointer_table as *const _ as usize,
        ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;

        let mut file_handle = FileHandle::new(file_path, 0).unwrap();

        // put the whole file into a buffer
        // TODO ensure the *kernel* has enough memory for this
        let size = file_handle.size() as usize;
        let mut buffer: Vec<u8> = Vec::with_capacity(size);
        unsafe {
            buffer.set_len(size); // unsafe, but we will overwrite all bytes
        }
        let program_slice = unsafe { core::slice::from_raw_parts_mut(buffer.as_mut_ptr(), size) };
        let bytes_read = file_handle.read(program_slice.as_mut_ptr(), size);
        if bytes_read as usize != size {
            panic!("Error reading file");
        }

        // allocate enough pages at beginning of virtual memory for elf loading
        // TODO do not map pages before the first used virtual address in the elf file (typically 0x400000)
        let heap_page_number =
            (self.get_size_of_program(program_slice) + PAGE_SIZE - 1) / PAGE_SIZE;

        self.heap_l2_table_number =
            (heap_page_number + PAGE_TABLE_ENTRIES - 1) / PAGE_TABLE_ENTRIES;
        self.heap_l1_table_number = (heap_page_number) % PAGE_TABLE_ENTRIES;

        if self.heap_l2_table_number > 15 {
            panic!("Heap size exceeds maximum limit of 16 L2 page directory tables");
        }

        for i in 0..self.heap_l2_table_number {
            self.l2_page_directory_table_beginning.entry[i] =
                Process::get_physical_address_for_virtual_address(
                    &self.l1_page_table_beginning[i] as *const _ as usize,
                ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;

            if i < self.heap_l2_table_number - 1 {
                for j in 0..PAGE_TABLE_ENTRIES {
                    self.l1_page_table_beginning[i].entry[j] =
                        allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;
                }
            } else {
                for j in 0..self.heap_l1_table_number {
                    self.l1_page_table_beginning[i].entry[j] =
                        allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;
                }
            }
        }

        self.l3_page_directory_pointer_table_beginning.entry[0] =
            Process::get_physical_address_for_virtual_address(
                &self.l2_page_directory_table_beginning as *const _ as usize,
            ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;
        self.l4_page_map_l4_table.entry[0] = Process::get_physical_address_for_virtual_address(
            &self.l3_page_directory_pointer_table_beginning as *const _ as usize,
        ) | PAGE_ENTRY_FLAGS_USERSPACE as usize;

        //print_page_table_tree(&self.l4_page_map_l4_table as *const _ as u64);

        // TODO Here we load the new pagetable into cr3 for the first process. This needs to happen because otherwise we cant load the programm into the first pages. This is a hack I think
        self.cr3 = Process::get_physical_address_for_virtual_address(
            &self.l4_page_map_l4_table as *const _ as usize,
        );

        kprint!("Process CR3: {:x}\n", self.cr3);

        unsafe {
            asm!(
                "mov cr3, r15",
                in("r15") self.cr3,
                options(nostack, preserves_flags)
            );
        }

        //print_page_table_tree(&self.l4_page_map_l4_table as *const _ as u64);

        self.rsp = USERSPACE_STACK_TOP_ADDRESS as u64;

        let (entry, v_addr, p_memsz) = self.load_elf_from_bin(&program_slice);
        self.rip = entry;

        self.init_process_heap(v_addr, p_memsz);

        unsafe {
            asm!(
                "mov cr3, r15",
                in("r15") KERNEL_CR3.load(Ordering::Relaxed) as u64,
                options(nostack, preserves_flags)
            );
        }

        self.ss = 0x1b;
        self.cs = 0x23;
        self.rflags = 0x202;
        self.state = ProcessState::Prepared;
    }

    fn init_process_heap(&mut self, v_addr: usize, p_memsz: usize) {
        let _event = core::hint::black_box(crate::instrument!());

        let heap_bottom = v_addr + p_memsz + 1;

        // calculate heap size as difference between the beginning of the next page frame and heap bottom
        let heap_size = PAGE_SIZE - (heap_bottom % PAGE_SIZE);

        unsafe {
            self.heap_allocator.lock().init(
                heap_bottom as *mut u8,
                heap_size as usize, // TODO Heap only uses the rest of the current page frame
            );
        }
    }

    // TODO unallocate stack memory when stack gets smaller again?
    pub fn extend_stack(&mut self) {
        let _event = core::hint::black_box(crate::instrument!());

        self.stack_page_counter += 1;

        if self.stack_page_counter >= PAGE_TABLE_ENTRIES {
            panic!("Stack size exceeds maximum limit of 512 pages");
        }

        // TODO limited to 512 stack pages
        if PAGE_SIZE == HUGE_PAGE_SIZE {
            self.l2_page_directory_table.entry[PAGE_TABLE_ENTRIES - self.stack_page_counter] =
                allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;
            return;
        } else {
            self.l1_page_table.entry[PAGE_TABLE_ENTRIES - self.stack_page_counter] =
                allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;
        }
    }

    pub fn malloc(&mut self, size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            let layout = core::alloc::Layout::from_size_align_unchecked(size, 0x8);

            let success = false;

            while !success {
                match self.heap_allocator.lock().allocate_first_fit(layout) {
                    Ok(address) => {
                        return address.as_ptr() as u64;
                    }
                    Err(()) => {
                        //DEBUG!("Allocating userspace memory failed - attempting to increase heap size\n");
                    }
                }

                // allocate more memory if not sufficient amount is available
                // TODO this only works until 7 l1 page directory tables are full; // later, we need to allocate more l1 page directory tables

                // Map a new frame into the L1 table
                self.l1_page_table_beginning[self.heap_l2_table_number - 1].entry
                    [self.heap_l1_table_number] =
                    allocate_page_frame() | PAGE_ENTRY_FLAGS_USERSPACE as usize;

                // Ensure the page table write is visible before proceeding
                // TODO really required?
                core::sync::atomic::fence(Ordering::Release);

                self.heap_l1_table_number += 1;

                if self.heap_l1_table_number >= PAGE_TABLE_ENTRIES {
                    self.heap_l1_table_number = 0;
                    self.heap_l2_table_number += 1;

                    if self.heap_l2_table_number >= 15 {
                        ERROR!("Heap size exceeds maximum limit of 16 L1 page directory tables\n");
                        panic!("Out of memory");
                    }

                    self.l2_page_directory_table_beginning.entry[self.heap_l2_table_number - 1] =
                        &self.l1_page_table_beginning[self.heap_l2_table_number - 1] as *const _
                            as usize
                            - KERNEL_HIGHER_HALF_BASE
                            | PAGE_ENTRY_FLAGS_USERSPACE as usize;
                }

                self.heap_allocator.lock().extend(PAGE_SIZE);
            }

            ERROR!("Failed to allocate memory after extending heap\n");
            panic!("Out of memory");
        }
    }

    pub fn realloc(&mut self, ptr: u64, new_size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            if ptr == 0 {
                return self.malloc(new_size);
            }

            let layout = core::alloc::Layout::from_size_align_unchecked(new_size, 0x8);

            if new_size == 0 {
                self.heap_allocator
                    .lock()
                    .deallocate(core::ptr::NonNull::new_unchecked(ptr as *mut u8), layout);
                return 0;
            }

            /*
                // SAFETY: the caller must ensure that the `new_size` does not overflow.
                // `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
                let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
                // SAFETY: the caller must ensure that `new_layout` is greater than zero.
                let new_ptr = unsafe { self.alloc(new_layout) };
                if !new_ptr.is_null() {
                    // SAFETY: the previously allocated block cannot overlap the newly allocated block.
                    // The safety contract for `dealloc` must be upheld by the caller.
                    unsafe {
                        ptr::copy_nonoverlapping(ptr, new_ptr, cmp::min(layout.size(), new_size));
                        self.dealloc(ptr, layout);
                    }
                }
            new_ptr */

            let new_ptr = self.heap_allocator.lock().allocate_first_fit(layout);

            if new_ptr.is_ok() {
                let new_address = new_ptr.unwrap().as_ptr() as u64;

                core::ptr::copy_nonoverlapping(ptr as *const u8, new_address as *mut u8, new_size);

                self.heap_allocator
                    .lock()
                    .deallocate(core::ptr::NonNull::new_unchecked(ptr as *mut u8), layout);

                return new_address;
            }

            ERROR!("Failed to reallocate memory\n");
            panic!("Out of memory");
        }
    }

    pub fn launch(&mut self) {
        let _event = core::hint::black_box(crate::instrument!());

        INFO!("Launching process");
        self.state = ProcessState::Passive;
    }

    pub fn activate(&mut self, initial_start: bool) {
        let _event = core::hint::black_box(crate::instrument!());

        DEBUG!("Activating process");
        unsafe extern "C" {
            static mut pushed_registers: *mut RegistersStruct;
            static mut stack_frame: *mut u64;
        }

        unsafe {
            //kprint!("Stack frame: {:x}\n", stack_frame as u64);
            //kprint!("Pushed registers: {:x}\n", pushed_registers as u64);

            if !initial_start {
                (*pushed_registers).xmm7 = self.registers.xmm7;
                (*pushed_registers).xmm6 = self.registers.xmm6;
                (*pushed_registers).xmm5 = self.registers.xmm5;
                (*pushed_registers).xmm4 = self.registers.xmm4;
                (*pushed_registers).xmm3 = self.registers.xmm3;
                (*pushed_registers).xmm2 = self.registers.xmm2;
                (*pushed_registers).xmm1 = self.registers.xmm1;
                (*pushed_registers).xmm0 = self.registers.xmm0;
                (*pushed_registers).r15 = self.registers.r15;
                (*pushed_registers).r14 = self.registers.r14;
                (*pushed_registers).r13 = self.registers.r13;
                (*pushed_registers).r12 = self.registers.r12;
                (*pushed_registers).r11 = self.registers.r11;
                (*pushed_registers).r10 = self.registers.r10;
                (*pushed_registers).r9 = self.registers.r9;
                (*pushed_registers).r8 = self.registers.r8;
                (*pushed_registers).rbp = self.registers.rbp;
                (*pushed_registers).rsi = self.registers.rsi;
                (*pushed_registers).rdx = self.registers.rdx;
                (*pushed_registers).rcx = self.registers.rcx;
                (*pushed_registers).rbx = self.registers.rbx;
                (*pushed_registers).rax = self.registers.rax;

                core::ptr::write_volatile(stack_frame.add(0), self.rip as u64);
                core::ptr::write_volatile(stack_frame.add(1), self.cs);
                core::ptr::write_volatile(stack_frame.add(2), self.rflags);
                core::ptr::write_volatile(stack_frame.add(3), self.rsp);
                core::ptr::write_volatile(stack_frame.add(4), self.ss);
            }

            asm!(
                "mov cr3, r15",
                in("r15") self.cr3,
                options(nostack, preserves_flags),
                clobber_abi("C")
            );
        }

        self.state = ProcessState::Active;
    }

    pub fn passivate(&mut self) {
        let _event = core::hint::black_box(crate::instrument!());

        DEBUG!("Passivating process");
        unsafe extern "C" {
            static pushed_registers: *const RegistersStruct;
            static stack_frame: *const u64;
        }

        unsafe {
            //kprint!("Stack frame: {:x}\n", stack_frame as u64);
            self.registers.xmm7 = (*pushed_registers).xmm7;
            self.registers.xmm6 = (*pushed_registers).xmm6;
            self.registers.xmm5 = (*pushed_registers).xmm5;
            self.registers.xmm4 = (*pushed_registers).xmm4;
            self.registers.xmm3 = (*pushed_registers).xmm3;
            self.registers.xmm2 = (*pushed_registers).xmm2;
            self.registers.xmm1 = (*pushed_registers).xmm1;
            self.registers.xmm0 = (*pushed_registers).xmm0;
            self.registers.r15 = (*pushed_registers).r15;
            self.registers.r14 = (*pushed_registers).r14;
            self.registers.r13 = (*pushed_registers).r13;
            self.registers.r12 = (*pushed_registers).r12;
            self.registers.r11 = (*pushed_registers).r11;
            self.registers.r10 = (*pushed_registers).r10;
            self.registers.r9 = (*pushed_registers).r9;
            self.registers.r8 = (*pushed_registers).r8;
            self.registers.rbp = (*pushed_registers).rbp;
            self.registers.rsi = (*pushed_registers).rsi;
            self.registers.rdx = (*pushed_registers).rdx;
            self.registers.rcx = (*pushed_registers).rcx;
            self.registers.rbx = (*pushed_registers).rbx;
            self.registers.rax = (*pushed_registers).rax;

            self.rip = *(stack_frame.add(0)) as usize;
            self.cs = *(stack_frame.add(1));
            self.rflags = *(stack_frame.add(2));
            self.rsp = *(stack_frame.add(3));
            self.ss = *(stack_frame.add(4));
        }

        self.state = ProcessState::Passive;
    }

    pub fn activatable(&self) -> bool {
        let _event = core::hint::black_box(crate::instrument!());

        match self.state {
            ProcessState::Passive => true,
            _ => false,
        }
    }

    // According to AMD Volume 2, page 146

    fn get_physical_address_for_virtual_address(vaddr: usize) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        let l4_page_map_table_offset = (vaddr >> L4_TABLE_SHIFT) & 0x1ff;
        let l3_page_directory_pointer_offset = (vaddr >> L3_TABLE_SHIFT) & 0x1ff;
        let l2_page_directory_offset = (vaddr >> L2_TABLE_SHIFT) & 0x1ff;
        let l1_page_table_offset = (vaddr >> L1_TABLE_SHIFT) & 0x1ff;
        let page_offset = vaddr & PAGE_OFFSET_MASK;

        unsafe {
            let mut cr3: u64;

            asm!("mov {}, cr3", out(reg) cr3);

            let l4_page_map_base_address = cr3 as usize | KERNEL_HIGHER_HALF_BASE;

            let l3_page_directory_pointer_table_address =
                *((l4_page_map_base_address + l4_page_map_table_offset * 8) as *const usize)
                    & ENTRY_MASK
                    | KERNEL_HIGHER_HALF_BASE;

            let l2_page_directory_table_address = *((l3_page_directory_pointer_table_address
                + l3_page_directory_pointer_offset * 8)
                as *const usize)
                & ENTRY_MASK
                | KERNEL_HIGHER_HALF_BASE;

            if PAGE_SIZE == HUGE_PAGE_SIZE {
                let physical_page_address = *((l2_page_directory_table_address
                    + l2_page_directory_offset * 8)
                    as *const usize)
                    & ENTRY_MASK;

                return physical_page_address + page_offset;
            } else {
                let l1_page_table_address = *((l2_page_directory_table_address
                    + l2_page_directory_offset * 8)
                    as *const usize)
                    & ENTRY_MASK
                    | KERNEL_HIGHER_HALF_BASE;

                let physical_page_address = *((l1_page_table_address + l1_page_table_offset * 8)
                    as *const usize)
                    & ENTRY_MASK;

                return physical_page_address + page_offset;
            }
        }
    }

    pub fn get_c3_page_map_l4_base_address(&self) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        Process::get_physical_address_for_virtual_address(
            &(self.l4_page_map_l4_table) as *const _ as usize,
        )
    }

    pub fn get_entry_ip(&self) -> usize {
        self.rip
    }

    // TODO reduce code duplication with load_elf_from_bin
    pub fn get_size_of_program(&mut self, program_slice: &[u8]) -> usize {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            let file =
                elf::ElfBytes::<AnyEndian>::minimal_parse(program_slice).expect("Open test1");

            let elf_header = file.ehdr;

            kprint!("Entry point is at: {:x}\n", elf_header.e_entry);

            let program_headers = file
                .segments()
                .unwrap()
                .iter()
                .filter(|phdr| phdr.p_type == PT_LOAD);

            let mut last_v_addr: usize = 0;
            let mut last_p_memsz: usize = 0;

            for phdr in program_headers {
                kprint!(
                    "Load segment is at: {:x}\nMem Size is: {:x}\n",
                    phdr.p_vaddr,
                    phdr.p_memsz
                );

                if last_v_addr < phdr.p_vaddr as usize {
                    last_v_addr = phdr.p_vaddr as usize;
                    last_p_memsz = phdr.p_memsz as usize;
                }
            }

            return last_v_addr + last_p_memsz;
        }
    }

    pub fn load_elf_from_bin(
        &mut self,
        program_slice: &[u8],
        offset: usize,
    ) -> (usize, usize, usize) {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            let file =
                elf::ElfBytes::<AnyEndian>::minimal_parse(program_slice).expect("Open test1");

            let elf_header = file.ehdr;

            kprint!("Entry point is at: {:x}\n", elf_header.e_entry);

            let program_headers = file
                .segments()
                .unwrap()
                .iter()
                .filter(|phdr| phdr.p_type == PT_LOAD);

            let mut last_v_addr: usize = 0;
            let mut last_p_memsz: usize = 0;

            for phdr in program_headers {
                kprint!(
                    "Load segment is at: {:x}\nMem Size is: {:x}\n",
                    phdr.p_vaddr,
                    phdr.p_memsz
                );

                asm!(
                    "mov rcx, {}
                    mov rsi, {}
                    mov rdi, {}
                    rep movsb",
                    in(reg) phdr.p_filesz,
                    in(reg) program_slice.as_ptr() as usize + phdr.p_offset as usize,
                    in(reg) phdr.p_vaddr,
                    out("rcx") _,
                    out("rsi") _,
                    out("rdi") _
                );

                if phdr.p_flags & 0x2 != 0 {
                    // Writable segment --> BSS
                    let bss_start = phdr.p_vaddr + phdr.p_filesz;
                    let bss_size = phdr.p_memsz - phdr.p_filesz;

                    // Zeroes the bss region
                    asm!(
                        "mov rcx, {}
                        xor rsi, rsi
                        mov rdi, {}
                        rep movsb",
                        in(reg) bss_size,
                        in(reg) bss_start,
                        out("rcx") _,
                        out("rsi") _,
                        out("rdi") _
                    );
                }

                if last_v_addr < phdr.p_vaddr as usize {
                    last_v_addr = phdr.p_vaddr as usize;
                    last_p_memsz = phdr.p_memsz as usize;
                }
            }

            return (elf_header.e_entry as usize, last_v_addr, last_p_memsz);
        }
    }

    pub fn set_working_directory(&mut self, path: &'static str) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());
        self.working_directory = path;
        return 0;
    }

    pub fn get_working_directory(&self) -> &'static str {
        let _event = core::hint::black_box(crate::instrument!());
        self.working_directory
    }

    pub fn fopen(&mut self, path: &str, mode: &str) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        let mode_num = match mode {
            "r" => 0,
            "w" => 1,
            "rw" => 2,
            _ => 0, // default to read-only
        };
        match FileHandle::new(path, mode_num) {
            Some(file_handle) => {
                kprint!("File opened: {}\n", path);
                let handle_id = self.next_handle_id;
                self.next_handle_id += 1;
                self.file_handles.insert(handle_id, file_handle);
                kprint!("File handle id: {}\n", handle_id);
                return handle_id;
            }
            None => {
                kprint!("Error opening file: {}\n", path);
                return 0;
            }
        }
    }

    pub fn fread(&mut self, handle_id: u64, buffer: *mut u8, size: usize) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        if let Some(file_handle) = self.file_handles.get_mut(&handle_id) {
            let bytes_read = file_handle.read(buffer, size);
            file_handle.offset += bytes_read as usize;
            return bytes_read;
        } else {
            ERROR!("Invalid file handle id: {}\n", handle_id);
            return 0;
        }
    }

    pub fn fseek(&mut self, handle_id: u64, offset: usize, whence: u32) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());

        if let Some(file_handle) = self.file_handles.get_mut(&handle_id) {
            file_handle.fseek(offset, whence);
            return 0;
        } else {
            ERROR!("Invalid file handle id: {}\n", handle_id);
            return 0;
        }
    }

    pub fn get_parent_id(&self) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());
        self.parent_id
    }

    pub fn get_pid(&self) -> u64 {
        let _event = core::hint::black_box(crate::instrument!());
        self.process_id
    }

    pub fn clone_from_parent(&mut self, parent: &Process) {
        let _event = core::hint::black_box(crate::instrument!());

        self.working_directory = parent.working_directory;
        self.parent_id = parent.process_id;
    }

    pub fn put_to_sleep(&mut self) {
        let _event = core::hint::black_box(crate::instrument!());

        DEBUG!("Putting process to sleep");
        self.state = ProcessState::Sleeping;
    }
}
