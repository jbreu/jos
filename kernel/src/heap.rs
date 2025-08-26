use core::{
    alloc::GlobalAlloc,
    arch::asm,
    panic,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
use linked_list_allocator::LockedHeap;

use crate::{DEBUG, mem::allocate_page_frame, mem_config::*};

/**
 * Wrapper around the locked heap allocator.
 *
 * Main purpose is to be able to react on failed allocations
 */
struct LockedHeapWrapper {
    inner: LockedHeap,
}

static HEAP_PAGE_NUMBER: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for LockedHeapWrapper {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            loop {
                match self.inner.lock().allocate_first_fit(layout) {
                    Ok(address) => {
                        return address.as_ptr();
                    }
                    Err(()) => {
                        DEBUG!(
                            "Allocating kernel heap memory failed - attempting to increase heap size\n"
                        );
                    }
                }

                let mut cr3: u64;
                asm!("mov {}, cr3", out(reg) cr3);

                let l4_pml4_table =
                    ((cr3 as usize & ENTRY_MASK) + KERNEL_HIGHER_HALF_BASE) as *const usize;
                let l3_pdpt = ((*l4_pml4_table.add(256) & ENTRY_MASK) + KERNEL_HIGHER_HALF_BASE)
                    as *const usize;

                if PAGE_SIZE == BASE_PAGE_SIZE {
                    let num_l2_page_dirs = KERNEL_SIZE / PAGE_SIZE / PAGE_TABLE_ENTRIES;

                    let l2_page_dir =
                        ((*l3_pdpt & ENTRY_MASK) + KERNEL_HIGHER_HALF_BASE) as *mut usize;
                    let l1_page_table = ((*l2_page_dir.add(num_l2_page_dirs) & ENTRY_MASK)
                        + KERNEL_HIGHER_HALF_BASE)
                        as *mut usize;

                    // allocate more memory if not sufficient amount is available
                    // TODO this only works until one l1 page directory table is full; // later, we need to allocate more l2 page directory tables
                    *l1_page_table.add(HEAP_PAGE_NUMBER.load(Ordering::Relaxed)) =
                        allocate_page_frame() | PAGE_ENTRY_FLAGS_KERNELSPACE as usize;

                    HEAP_PAGE_NUMBER.fetch_add(1, Ordering::Relaxed);
                } else {
                    todo!("allocating more kernel heap for 2MB pages is not implemented yet");
                }

                self.inner.lock().extend(PAGE_SIZE)
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe {
            self.inner
                .lock()
                .deallocate(NonNull::new(ptr).expect("pointer was null"), layout)
        }
    }
}

impl LockedHeapWrapper {
    const fn empty() -> Self {
        LockedHeapWrapper {
            inner: LockedHeap::empty(),
        }
    }

    fn init(&self, start: *mut u8, size: usize) {
        let _event = core::hint::black_box(crate::instrument!());

        unsafe { self.inner.lock().init(start, size) }
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeapWrapper = LockedHeapWrapper::empty();

fn allocate_kernel_heap_pages_after_already_allocated_memory() -> usize {
    let _event = core::hint::black_box(crate::instrument!());

    let mut kernel_cr3: u64;

    unsafe {
        // Load the CR3 register into kernel_cr3
        asm!("mov {}, cr3", out(reg) kernel_cr3);

        let l4_pml4_table = (kernel_cr3 as usize & ENTRY_MASK) as *const usize;
        let l3_pdpt = (*l4_pml4_table.add(256) & ENTRY_MASK) as *const usize;

        if PAGE_SIZE == BASE_PAGE_SIZE {
            let num_l2_page_dirs = KERNEL_SIZE / PAGE_SIZE / PAGE_TABLE_ENTRIES;

            let num_last_l1_table_entries = KERNEL_SIZE / PAGE_SIZE % PAGE_TABLE_ENTRIES;

            let l2_page_dir = (*l3_pdpt & ENTRY_MASK) as *mut usize;
            let l1_page_table = (*l2_page_dir.add(num_l2_page_dirs) & ENTRY_MASK) as *mut usize;

            *l1_page_table.add(num_last_l1_table_entries) =
                allocate_page_frame() | PAGE_ENTRY_FLAGS_KERNELSPACE as usize;

            HEAP_PAGE_NUMBER.store(num_last_l1_table_entries + 1, Ordering::Relaxed);
        } else {
            todo!("allocating kernel heap for 2MB pages is not implemented yet");
        }
    }

    // TODO this is hard coded, as we are adding to the entries above --> make it dynamic
    return KERNEL_HIGHER_HALF_BASE + KERNEL_SIZE;
}

pub fn init_kernel_heap() {
    // TODO add more / dynamic page frames
    // TODO do not start with new page frame, but start where kernel ends

    let heap_start = allocate_kernel_heap_pages_after_already_allocated_memory();

    ALLOCATOR.init(heap_start as *mut u8, PAGE_SIZE);
}
