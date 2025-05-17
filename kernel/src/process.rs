use crate::filesystem::FileHandle;
use crate::kprint;
use crate::mem::allocate_page_frame;
extern crate alloc;
use crate::ERROR;
use alloc::vec::Vec;
use core::ptr::addr_of;
use core::{arch::asm, fmt::Debug};
use tracing::{debug, info, instrument};

pub static mut KERNEL_CR3: u64 = 0;

// stores a process' registers when it gets interrupted
#[repr(C)]
#[derive(Default)]
struct RegistersStruct {
    // Has to be always in sync with asm macro "pop_all_registers"
    xmm7: u128,
    xmm6: u128,
    xmm5: u128,
    xmm4: u128,
    xmm3: u128,
    xmm2: u128,
    xmm1: u128,
    xmm0: u128,
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
pub struct PageTable {
    pub entry: [u64; 512],
}

impl PageTable {
    fn default() -> Self {
        Self { entry: [0; 512] }
    }
}

fn _print_page_table_tree_for_cr3() {
    let mut cr3: u64;

    unsafe {
        asm!("mov r12, cr3", out("r12") cr3);
    }

    print_page_table_tree(cr3);
}

fn check_half(entry: *const u64) -> *const u64 {
    if entry < 0xffff800000000000 as *const u64 {
        return (entry as u64 + 0xffff800000000000 as u64) as *const u64;
    }
    entry
}

fn print_page_table_tree(start_addr: u64) {
    let entry_mask = 0x0008_ffff_ffff_f800;

    unsafe {
        kprint!("start_addr: {:#x}\n", start_addr);

        for l4_entry in 0..512 {
            let l4bits = *check_half((start_addr + l4_entry * 8) as *const u64);
            if l4bits != 0 {
                kprint!("   L4: {} - {:#x}\n", l4_entry, l4bits & entry_mask);

                for l3_entry in 0..512 {
                    let l3bits = *check_half(((l4bits & entry_mask) + l3_entry * 8) as *const u64);
                    if l3bits != 0 {
                        kprint!("      L3: {} - {:#x}\n", l3_entry, l3bits & entry_mask);

                        for l2_entry in 0..512 {
                            let l2bits =
                                *check_half(((l3bits & entry_mask) + l2_entry * 8) as *const u64);

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

#[derive(Debug)]
enum ProcessState {
    New,
    Prepared,
    Active,
    Passive,
}

pub struct Process {
    registers: RegistersStruct,

    l2_page_directory_table: PageTable,
    l3_page_directory_pointer_table: PageTable,
    l4_page_map_l4_table: PageTable,

    l2_page_directory_table_beginning: PageTable,
    l3_page_directory_pointer_table_beginning: PageTable,

    rip: u64,
    rsp: u64,
    cr3: u64,
    ss: u64,
    cs: u64,
    rflags: u64,

    state: ProcessState,

    heap_allocator: linked_list_allocator::LockedHeap,

    working_directory: &'static str,

    file_handles: Vec<FileHandle>,
}

impl Debug for Process {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Process {{ state: {:?} }}", self.state)
    }
}

impl Process {
    #[instrument]
    pub fn new() -> Self {
        Self {
            registers: RegistersStruct::default(),
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

            working_directory: "/",
            file_handles: Vec::new(),
        }
    }

    #[instrument]
    pub fn initialize(&mut self) {
        // TODO remove hard coding
        // TODO Task stack
        // Upper end of page which begins at 0x2000000 = 50 MByte in phys RAM
        // TODO only one page (2MB) yet!
        self.l2_page_directory_table.entry[511] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user

        // TODO HackID1: Fixed kernel stack for interrupts (starts at 40 MByte)
        self.l2_page_directory_table.entry[510] = 20 * 0x200000 | 0b10000011; // bitmask: present, writable, huge page

        self.l3_page_directory_pointer_table.entry[511] =
            Process::get_physical_address_for_virtual_address(
                &self.l2_page_directory_table as *const _ as u64,
            ) | 0b111;
        self.l4_page_map_l4_table.entry[511] = Process::get_physical_address_for_virtual_address(
            &self.l3_page_directory_pointer_table as *const _ as u64,
        ) | 0b111;

        // allocate two pages page at beginning of virtual memory for elf loading
        // TODO allocate more if needed

        self.l2_page_directory_table_beginning.entry[0] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[1] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[2] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[3] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[4] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[5] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[6] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[7] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l2_page_directory_table_beginning.entry[8] = allocate_page_frame() | 0b10000111; // bitmask: present, writable, huge page, access from user
        self.l3_page_directory_pointer_table_beginning.entry[0] =
            Process::get_physical_address_for_virtual_address(
                &self.l2_page_directory_table_beginning as *const _ as u64,
            ) | 0b111;
        self.l4_page_map_l4_table.entry[0] = Process::get_physical_address_for_virtual_address(
            &self.l3_page_directory_pointer_table_beginning as *const _ as u64,
        ) | 0b111;

        // TODO Hack? map the kernel pages from main.asm to process
        // TODO Later, the kernel pages should be restructed to superuser access; in order to do so, the process code and data must be fully in userspace pages
        unsafe {
            if KERNEL_CR3 == 0 {
                asm!("mov r15, cr3", out("r15") KERNEL_CR3);
            }

            kprint!("Kernel CR3: {:x}\n", KERNEL_CR3);

            self.l4_page_map_l4_table.entry[256] = *((KERNEL_CR3 + 256 * 8) as *const _);
        }

        // TODO Here we load the new pagetable into cr3 for the first process. This needs to happen because otherwise we cant load the programm into the first pages. This is a hack I think
        self.cr3 = Process::get_physical_address_for_virtual_address(
            &self.l4_page_map_l4_table as *const _ as u64,
        );

        kprint!("Process CR3: {:x}\n", self.cr3);

        unsafe {
            print_page_table_tree(KERNEL_CR3 as u64);
        }

        unsafe {
            asm!(
                "mov cr3, r15",
                in("r15") self.cr3,
                options(nostack, preserves_flags)
            );
        }

        print_page_table_tree(&self.l4_page_map_l4_table as *const _ as u64);

        self.rsp = 0xffff_ffff_ffff_ffff;

        let (entry, v_addr, p_memsz) = Process::load_elf_from_bin();
        self.rip = entry;

        self.init_process_heap(v_addr, p_memsz);
        //kprint!("test alloc 5 bytes at {:x}\n", self.malloc(5));

        //todo!();
        //file::fopen();

        unsafe {
            asm!(
                "mov cr3, r15",
                in("r15") KERNEL_CR3,
                options(nostack, preserves_flags)
            );
        }

        self.ss = 0x1b;
        self.cs = 0x23;
        self.rflags = 0x202;
        self.state = ProcessState::Prepared;
    }

    #[instrument]
    fn init_process_heap(&mut self, v_addr: u64, p_memsz: u64) {
        let heap_bottom = v_addr + p_memsz + 1;
        let heap_size = 0x12000000 - 0x1 - heap_bottom; // TODO: 0x12000000 is the upper limit of the allocated memory

        // TODO add more / dynamic page frames
        unsafe {
            self.heap_allocator.lock().init(
                heap_bottom as *mut u8,
                heap_size as usize, // TODO Heap only uses the rest of the current page frame
            );
        }
    }

    #[instrument]
    pub fn malloc(&mut self, size: usize) -> u64 {
        unsafe {
            let layout = core::alloc::Layout::from_size_align_unchecked(size, 0x8);
            match self.heap_allocator.lock().allocate_first_fit(layout) {
                Ok(address) => return address.as_ptr() as u64,
                Err(error) => {
                    kprint!("Allocating memory failed!\n");
                    kprint!("   Size: 0x{:x}\n", size);
                    panic!("Problem allocating memory: {:?}", error) //TODO allocate more memory if not sufficient amount is available
                }
            }
        }
    }

    #[instrument]
    pub fn launch(&mut self) {
        info!("Launching process");
        self.state = ProcessState::Passive;
    }

    #[instrument]
    pub fn activate(&mut self, initial_start: bool) {
        debug!("Activating process");
        extern "C" {
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

                core::ptr::write_volatile(stack_frame.add(0), self.rip);
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

    #[instrument]
    pub fn passivate(&mut self) {
        debug!("Passivating process");
        extern "C" {
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

            self.rip = *(stack_frame.add(0));
            self.cs = *(stack_frame.add(1));
            self.rflags = *(stack_frame.add(2));
            self.rsp = *(stack_frame.add(3));
            self.ss = *(stack_frame.add(4));
        }

        self.state = ProcessState::Passive;
    }

    pub fn activatable(&self) -> bool {
        match self.state {
            ProcessState::Passive => true,
            _ => false,
        }
    }

    fn _get_tss_rsp0(&self) -> u64 {
        0xffff_ffff_ffcf_ffff
    }

    // According to AMD Volume 2, page 146
    fn get_physical_address_for_virtual_address(vaddr: u64) -> u64 {
        // Simple variant, only works for kernel memory
        // adding 1 page frame as heap has different mapping
        //vaddr - 0xffff800000000000 + 0x200000

        // TODO get this running
        let page_map_l4_table_offset = (vaddr & 0x0000_ff80_0000_0000) >> 39;
        let page_directory_pointer_offset = (vaddr & 0x0000_007f_c000_0000) >> 30;
        let page_directory_offset = (vaddr & 0x0000_0000_3fe0_0000) >> 21;
        let page_offset = vaddr & 0x0000_000_001f_f000;

        unsafe {
            let mut cr3: u64;

            asm!("mov {}, cr3", out(reg) cr3);

            let page_map_l4_base_address = cr3;

            let entry_mask: u64 = 0x0008_ffff_ffff_f800;

            let page_directory_pointer_table_address =
                *((page_map_l4_base_address + page_map_l4_table_offset * 8) as *const u64)
                    & entry_mask;

            let page_directory_table_address = *((page_directory_pointer_table_address
                + page_directory_pointer_offset * 8)
                as *const u64)
                & entry_mask;

            let physical_page_address = *((page_directory_table_address + page_directory_offset * 8)
                as *const u64)
                & entry_mask;

            return physical_page_address + page_offset;
        }
    }

    pub fn get_c3_page_map_l4_base_address(&self) -> u64 {
        Process::get_physical_address_for_virtual_address(
            &(self.l4_page_map_l4_table) as *const _ as u64,
        )
    }

    pub fn get_stack_top_address(&self) -> u64 {
        // Virtual Address, see AMD64 Volume 2 p. 146
        0xffff_ffff_ffff_ffff //3fff --> set 3*9 bits to 1 to identify each topmost entry in each table; fffff --> topmost address in the page; rest also 1 because sign extend
    }

    pub fn get_entry_ip(&self) -> u64 {
        self.rip
    }

    #[instrument]
    pub fn load_elf_from_bin() -> (u64, u64, u64) {
        extern "C" {
            static mut _binary_build_userspace_x86_64_unknown_none_debug_helloworld_start: u8;
            static mut _binary_build_userspace_x86_64_unknown_none_debug_helloworld_end: u8;
        }

        unsafe {
            kprint!(
                "embedded elf file\nstart: {:x}\n  end: {:x}\n",
                addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_start)
                    as *const u8 as usize,
                addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_end)
                    as *const u8 as usize
            );

            let size = addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_end)
                as *const u8 as usize
                - addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_start)
                    as *const u8 as usize;

            let slice = core::slice::from_raw_parts(
                addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_start),
                size,
            );

            use elf::abi::PT_LOAD;
            use elf::endian::AnyEndian;

            let file = elf::ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

            let elf_header = file.ehdr;

            kprint!("Entry point is at: {:x}\n", elf_header.e_entry);

            let program_headers = file
                .segments()
                .unwrap()
                .iter()
                .filter(|phdr| phdr.p_type == PT_LOAD);

            let mut last_v_addr: u64 = 0;
            let mut last_p_memsz: u64 = 0;

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
                    in(reg) phdr.p_memsz,
                    in(reg) addr_of!(_binary_build_userspace_x86_64_unknown_none_debug_helloworld_start) as *const u8 as usize + phdr.p_offset as usize,
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

                if last_v_addr < phdr.p_vaddr {
                    last_v_addr = phdr.p_vaddr;
                    last_p_memsz = phdr.p_memsz;
                }
            }

            return (elf_header.e_entry, last_v_addr, last_p_memsz);
        }
    }

    pub fn set_working_directory(&mut self, path: &'static str) -> u64 {
        self.working_directory = path;
        return 0;
    }

    pub fn get_working_directory(&self) -> &'static str {
        self.working_directory
    }

    pub fn fopen(&mut self, path: &str, mode: &str) -> u64 {
        let mode_num = match mode {
            "r" => 0,
            "w" => 1,
            "rw" => 2,
            _ => 0, // default to read-only
        };
        match FileHandle::new(path, mode_num) {
            Some(file_handle) => {
                kprint!("File opened: {}\n", path);
                self.file_handles.push(file_handle);
                let file_handle_index = self.file_handles.len();
                kprint!("File handle index: {}\n", file_handle_index);
                return file_handle_index as u64;
            }
            None => {
                kprint!("Error opening file: {}\n", path);
                return 0;
            }
        }
    }

    pub fn fread(&mut self, file_handle_index: u64, buffer: *mut u8, size: usize) -> u64 {
        // file_handle_index is 1-based
        if file_handle_index as usize > self.file_handles.len() {
            ERROR!("Invalid file handle index: {}\n", file_handle_index);
            return 0;
        }

        let file_handle = &mut self.file_handles[file_handle_index as usize - 1];
        let bytes_read = file_handle.read(buffer, size);
        file_handle.offset += bytes_read as usize;
        return bytes_read;
    }

    pub fn fseek(&mut self, file_handle_index: u64, offset: usize, whence: u32) -> u64 {
        // file_handle_index is 1-based
        if file_handle_index as usize > self.file_handles.len() {
            ERROR!("Invalid file handle index: {}\n", file_handle_index);
            return 0;
        }
        let file_handle = &mut self.file_handles[file_handle_index as usize - 1];

        file_handle.fseek(offset, whence);

        return 0;
    }
}
