ENTRY(start)

KERNEL_START = 1M;
KERNEL_OFFSET = 0xffff800000000000;

SECTIONS
{
	. = KERNEL_START;

	.boot :
	{
		KEEP(*(.multiboot_header))
	}

	.text : 
	{
		*boot*.o:*(.text)
	}

	.bss :
	{
		*boot*.o:*(.bss .bss.*)
	}

	.rodata :
	{
		*boot*.o:*(.rodata .rodata.*)
	}

	. += KERNEL_OFFSET;

	.kerneltext ALIGN(4K): AT(ADDR(.kerneltext) - KERNEL_OFFSET)
	{
		*.a:*(.text .text.*)
		*switch_to_ring3.o:*(.text .text.*)
		*interrupt.o:*(.text .text.*)
		*gdt.o:*(.text .text.*)
	}

	.kernelrodata ALIGN(4K): AT(ADDR(.kernelrodata) - KERNEL_OFFSET)
	{
		*.a:*(.rodata .rodata.*)
		*switch_to_ring3.o:*(.rodata .rodata.*)
		*interrupt.o:*(.rodata .rodata.*)
		*gdt.o:*(.rodata .rodata.*)
	}
	
	.kerneldata ALIGN(4K): AT(ADDR(.kerneldata) - KERNEL_OFFSET)
	{
		*.a:*(.data .data.*)
		*switch_to_ring3.o:*(.data .data.*)
		*interrupt.o:*(.data .data.*)
		*gdt.o:*(.data .data.*)
	}

	.kernelbss ALIGN(4K): AT(ADDR(.kernelbss) - KERNEL_OFFSET)
	{
		*.a:*(.bss .bss.*)
		*switch_to_ring3.o:*(.bss .bss.*)
		*interrupt.o:*(.bss .bss.*)
		*gdt.o:*(.bss .bss.*)
	}
}