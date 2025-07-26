/// Base size of a standard page (4 KiB)
pub const BASE_PAGE_SIZE: usize = 0x1000;

/// Size of a huge page (2 MiB)
pub const HUGE_PAGE_SIZE: usize = 0x200000;

// define PAGE_SIZE as HUGE_PAGE_SIZE for compatibility
pub const PAGE_SIZE: usize = HUGE_PAGE_SIZE;

/// Number of entries in each page table
pub const PAGE_TABLE_ENTRIES: usize = 512;

/// Mask for extracting the physical address from a page table entry
pub const ENTRY_MASK: u64 = 0x0008_ffff_ffff_f800;

/// Standard page table entry flags (Present + Writable + No User)
pub const BASE_PAGE_ENTRY_FLAGS: u64 = 0b11;

/// Huge page table entry flags (Present + Writable + No User + Huge)
pub const HUGE_PAGE_ENTRY_FLAGS: u64 = 0b10000011;

pub const PAGE_ENTRY_FLAGS_KERNELSPACE: u64 = HUGE_PAGE_ENTRY_FLAGS;

/// Standard page table entry flags (Present + Writable + User)
pub const BASE_PAGE_ENTRY_FLAGS_USERSPACE: u64 = 0b111;

/// Huge page table entry flags (Present + Writable + User + Huge)
pub const HUGE_PAGE_ENTRY_FLAGS_USERSPACE: u64 = 0b10000111;

pub const PAGE_ENTRY_FLAGS_USERSPACE: u64 = HUGE_PAGE_ENTRY_FLAGS_USERSPACE;

/// Mask for extracting different levels of page table offsets
pub const L4_TABLE_OFFSET_MASK: u64 = 0x0000_ff80_0000_0000;
pub const L3_TABLE_OFFSET_MASK: u64 = 0x0000_007f_c000_0000;
pub const L2_TABLE_OFFSET_MASK: u64 = 0x0000_0000_3fe0_0000;
pub const PAGE_OFFSET_MASK: u64 = 0x0000_000_001f_f000;

/// Shift amounts for page table offsets
pub const L4_TABLE_SHIFT: u32 = 39;
pub const L3_TABLE_SHIFT: u32 = 30;
pub const L2_TABLE_SHIFT: u32 = 21;

/// Memory region information
pub const KERNEL_SIZE: usize = 0x200000 * 20; // 40 MiB
pub const KERNEL_HEAP_SIZE: usize = 0x200000 * 10; // 20 MiB
pub const MAX_PAGE_FRAMES: usize = 0x100000000 / PAGE_SIZE; // 4 GiB total memory

/// Virtual memory layout constants
pub const KERNEL_HIGHER_HALF_BASE: usize = 0xffff800000000000;
pub const STACK_TOP_ADDRESS: usize = 0xffff_ffff_ffff_ffff;
