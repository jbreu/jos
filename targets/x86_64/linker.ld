ENTRY(start)

KERNEL_START = 1M;
KERNEL_OFFSET = 0xFFFFFF8000000000;

SECTIONS
{
	. = KERNEL_START;

	.boot ALIGN(4K) :
	{
		KEEP(*(.multiboot_header))
	}

	.boottext ALIGN(4K) : 
	{
		*(.boottext .boottext.*)
	}

	.bootbss ALIGN(4K) :
	{
		*(.bootbss .bootbss.*)
	}

	.bootrodata ALIGN(4K) :
	{
		*(.bootrodata .bootrodata.*)
	}

	. += KERNEL_OFFSET;

	.text ALIGN(4K): AT(ADDR(.text) - KERNEL_OFFSET)
	{
		*(.text .text.*)
	}

	.rodata ALIGN(4K): AT(ADDR(.rodata) - KERNEL_OFFSET)
	{
		*(.rodata .rodata.*)
	}
	
	.data ALIGN(4K): AT(ADDR(.data) - KERNEL_OFFSET)
	{
		*(.data .data.*)
	}

	.bss ALIGN(4K): AT(ADDR(.bss) - KERNEL_OFFSET)
	{
		*(COMMON)
		*(.bss .bss.*)
	}
}