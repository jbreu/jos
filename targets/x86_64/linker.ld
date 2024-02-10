ENTRY(start)

KERNEL_START = 1M;
KERNEL_OFFSET = 0xffff800000000000;

SECTIONS
{
	. = KERNEL_START;

	.boot ALIGN(4K) :
	{
		KEEP(*(.multiboot_header))
	}

	.text ALIGN(4K) : 
	{
		*boot*.o:*(.text)
	}

	.bootbss ALIGN(4K) :
	{
		*boot*.o:*(.bootbss .bootbss.*)
	}

	.bootrodata ALIGN(4K) :
	{
		*boot*.o:*(.bootrodata .bootrodata.*)
	}

	. += KERNEL_OFFSET;

	.text ALIGN(4K): AT(ADDR(.text) - KERNEL_OFFSET)
	{
		*.a:*(.text .text.*)
		*switch_to_ring3.o:*(.text .text.*)
		*interrupt.o:*(.text .text.*)
		*gdt.o:*(.text .text.*)
	}

	.rodata ALIGN(4K): AT(ADDR(.rodata) - KERNEL_OFFSET)
	{
		*.a:*(.rodata .rodata.*)
		*switch_to_ring3.o:*(.rodata .rodata.*)
		*interrupt.o:*(.rodata .rodata.*)
		*gdt.o:*(.rodata .rodata.*)
	}
	
	.data ALIGN(4K): AT(ADDR(.data) - KERNEL_OFFSET)
	{
		*.a:*(.data .data.*)
		*switch_to_ring3.o:*(.data .data.*)
		*interrupt.o:*(.data .data.*)
		*gdt.o:*(.data .data.*)
	}

	.bss ALIGN(4K): AT(ADDR(.bss) - KERNEL_OFFSET)
	{
		*(COMMON)
		*.a:*(.bss .bss.*)
		*switch_to_ring3.o:*(.bss .bss.*)
		*interrupt.o:*(.bss .bss.*)
		*gdt.o:*(.bss .bss.*)
	}
}