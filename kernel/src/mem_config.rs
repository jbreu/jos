/// Base size of a standard page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 0x1000;

/// Size of a huge page (2 MiB)
pub const HUGE_PAGE_SIZE: usize = 0x200000;

// define PAGE_SIZE as HUGE_PAGE_SIZE for compatibility
pub const PAGE_SIZE: usize = BASE_PAGE_SIZE;

/// Number of entries in each page table
pub const PAGE_TABLE_ENTRIES: usize = 512;

/// Standard page table entry flags (Present + Writable + No User)
pub const BASE_PAGE_ENTRY_FLAGS: u8 = 0b11;

/// Huge page table entry flags (Present + Writable + No User + Huge)
pub const HUGE_PAGE_ENTRY_FLAGS: u8 = 0b10000011;

pub const PAGE_ENTRY_FLAGS_KERNELSPACE: u8 = BASE_PAGE_ENTRY_FLAGS;

/// Standard page table entry flags (Present + Writable + User)
pub const BASE_PAGE_ENTRY_FLAGS_USERSPACE: u8 = 0b111;

/// Huge page table entry flags (Present + Writable + User + Huge)
pub const HUGE_PAGE_ENTRY_FLAGS_USERSPACE: u8 = 0b10000111;

pub const PAGE_ENTRY_FLAGS_USERSPACE: u8 = BASE_PAGE_ENTRY_FLAGS_USERSPACE;

pub const PAGE_OFFSET_MASK: usize = PAGE_SIZE - 1;

/// Mask for extracting the physical address from a page table entry
pub const ENTRY_MASK_2MB: usize = 0x000f_ffff_ffe0_0000; // bits [51:21]
pub const ENTRY_MASK_4KB: usize = 0x000f_ffff_ffff_f000; // bits [51:12]

pub const ENTRY_MASK: usize = ENTRY_MASK_4KB;

/// Shift amounts for page table offsets
pub const L4_TABLE_SHIFT: u32 = 39;
pub const L3_TABLE_SHIFT: u32 = 30;
pub const L2_TABLE_SHIFT: u32 = 21;
pub const L1_TABLE_SHIFT: u32 = 12;

/// Memory region information
pub const KERNEL_SIZE: usize = 0x200000 * 20; // 40 MiB // SYNCID2
pub const MAX_PAGE_FRAMES: usize = 0x100000000 / PAGE_SIZE; // 4 GiB total memory

/// Virtual memory layout constants
pub const KERNEL_HIGHER_HALF_BASE: usize = 0xffff_8000_0000_0000;
pub const KERNEL_STACK_TOP_ADDRESS: usize = 0xffff_ffff_ffff_ffff;
pub const USERSPACE_STACK_TOP_ADDRESS: usize = 0x0000_7fff_ffff_fff0;
