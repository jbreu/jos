global start
extern long_mode_start

section .boottext exec
bits 32
start:
 	mov esp, stack_top

	call check_multiboot
	call check_cpuid
	call check_long_mode
	call check_sse

	call setup_page_tables
	call enable_paging

	lgdt [gdt64.pointer]
	jmp gdt64.code_segment:long_mode_start

	hlt

; https://wiki.osdev.org/SSE
check_sse:
	;now enable SSE and the like
	mov eax, cr0
	and ax, 0xFFFB		;clear coprocessor emulation CR0.EM
	or ax, 0x2			;set coprocessor monitoring  CR0.MP
	mov cr0, eax
	mov eax, cr4
	or ax, 3 << 9		;set CR4.OSFXSR and CR4.OSXMMEXCPT at the same time
	mov cr4, eax
	ret

check_multiboot:
	cmp eax, 0x36d76289
	jne .no_multiboot
	ret
.no_multiboot:
	mov al, "M"
	jmp error

check_cpuid:
	pushfd
	pop eax
	mov ecx, eax
	xor eax, 1 << 21
	push eax
	popfd
	pushfd
	pop eax
	push ecx
	popfd
	cmp eax, ecx
	je .no_cpuid
	ret
.no_cpuid:
	mov al, "C"
	jmp error

check_long_mode:
	mov eax, 0x80000000
	cpuid
	cmp eax, 0x80000001
	jb .no_long_mode

	mov eax, 0x80000001
	cpuid
	test edx, 1 << 29
	jz .no_long_mode
	
	ret
.no_long_mode:
	mov al, "L"
	jmp error

setup_page_tables:
	; EQU is used to define constants in assembly
	%define HUGE_PAGE_SIZE    0x200000     ; 2 MiB
	%define SMALL_PAGE_SIZE   0x1000       ; 4 KiB
	%define PAGE_SIZE SMALL_PAGE_SIZE
	%define HUGE_PAGE_ENTRY_FLAGS   0b10000011    ; huge page, no access from user, writable, present
	%define PAGE_TABLE_FLAGS        0b011    	  ; no access from user, writable, present
	%define PAGE_ENTRY_FLAGS        PAGE_TABLE_FLAGS

	; Calculate number of pages: KERNEL_SIZE / PAGE_SIZE
	%define KERNEL_SIZE 0x2000000      ; Example: 32 MiB kernel size SYNCID2
	%define NUM_KERNEL_PAGES (KERNEL_SIZE / PAGE_SIZE)	

	mov eax, page_table_l3
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l4], eax
	; temporarily map the same l3 table also to middle of l4 of virtual memory to enable later switch to higher half kernel
	mov [page_table_l4+256*8], eax
	
	mov eax, page_table_l2
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l3], eax

	; uncomment block for 4 KiB page size
	; 40 MiB kernel size, 20 page tables, 512 entries each
	mov eax, page_table_l1_1
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2], eax
	mov eax, page_table_l1_2
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+8], eax
	mov eax, page_table_l1_3
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+16], eax
	mov eax, page_table_l1_4
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+24], eax
	mov eax, page_table_l1_5
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+32], eax
	mov eax, page_table_l1_6
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+40], eax
	mov eax, page_table_l1_7
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+48], eax
	mov eax, page_table_l1_8
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+56], eax
	mov eax, page_table_l1_9
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+64], eax
	mov eax, page_table_l1_10
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+72], eax
	mov eax, page_table_l1_11
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+80], eax
	mov eax, page_table_l1_12
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+88], eax
	mov eax, page_table_l1_13
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+96], eax
	mov eax, page_table_l1_14
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+104], eax
	mov eax, page_table_l1_15
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+112], eax
	mov eax, page_table_l1_16
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+120], eax
	mov eax, page_table_l1_17
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+128], eax
	mov eax, page_table_l1_18
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+136], eax
	mov eax, page_table_l1_19
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+144], eax
	mov eax, page_table_l1_20
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+152], eax
	mov eax, page_table_l1_21
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+160], eax
	; HACKID3
	; map as 510th l1 table
	mov eax, page_table_l1_special
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+510*8], eax

	mov ecx, 0 ; counter
.loop_1:
	
	mov eax, PAGE_SIZE
	mul ecx
	or eax, PAGE_ENTRY_FLAGS
	;mov [page_table_l2 + ecx * 8], eax ; uncomment for 2 MiB page size
	mov [page_table_l1_1 + ecx * 8], eax ; uncomment for 4 KiB page size

	inc ecx ; increment counter

	;cmp ecx, NUM_KERNEL_PAGES ; checks if the whole table is mapped
	cmp ecx, 512 ; checks if the whole table is mapped
	jne .loop_1 ; if not, continue

	mov ecx, 0 ; counter
.loop_2:
	
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 512*PAGE_SIZE ; offset for second page table
	or eax, PAGE_ENTRY_FLAGS
	;mov [page_table_l2 + ecx * 8], eax ; uncomment for 2 MiB page size
	mov [page_table_l1_2 + ecx * 8], eax ; uncomment for 4 KiB page size

	inc ecx ; increment counter

	;cmp ecx, NUM_KERNEL_PAGES ; checks if the whole table is mapped
	cmp ecx, 512 ; checks if the whole table is mapped
	jne .loop_2 ; if not, continue

	mov ecx, 0 ; counter
.loop_3:
	
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 1024*PAGE_SIZE ; offset for third page table
	or eax, PAGE_ENTRY_FLAGS
	;mov [page_table_l2 + ecx * 8], eax ; uncomment for 2 MiB page size
	mov [page_table_l1_3 + ecx * 8], eax ; uncomment for 4 KiB page size

	inc ecx ; increment counter

	;cmp ecx, NUM_KERNEL_PAGES ; checks if the whole table is mapped
	cmp ecx, 512 ; checks if the whole table is mapped
	jne .loop_3 ; if not, continue

	mov ecx, 0 ; counter
.loop_4:
	
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 1536*PAGE_SIZE ; offset for fourth page table
	or eax, PAGE_ENTRY_FLAGS
	;mov [page_table_l2 + ecx * 8], eax ; uncomment for 2 MiB page size
	mov [page_table_l1_4 + ecx * 8], eax ; uncomment for 4 KiB page size

	inc ecx ; increment counter

	;cmp ecx, NUM_KERNEL_PAGES ; checks if the whole table is mapped
	cmp ecx, 512 ; checks if the whole table is mapped
	jne .loop_4 ; if not, continue

	mov ecx, 0 ; counter
.loop_5:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 2048*PAGE_SIZE ; offset for fifth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_5 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_5

	mov ecx, 0 ; counter
.loop_6:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 2560*PAGE_SIZE ; offset for sixth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_6 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_6

	mov ecx, 0 ; counter
.loop_7:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 3072*PAGE_SIZE ; offset for seventh page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_7 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_7

	mov ecx, 0 ; counter
.loop_8:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 3584*PAGE_SIZE ; offset for eighth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_8 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_8

	mov ecx, 0 ; counter
.loop_9:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 4096*PAGE_SIZE ; offset for ninth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_9 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_9

	mov ecx, 0 ; counter
.loop_10:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 4608*PAGE_SIZE ; offset for tenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_10 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_10

	mov ecx, 0 ; counter
.loop_11:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 5120*PAGE_SIZE ; offset for eleventh page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_11 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_11

	mov ecx, 0 ; counter
.loop_12:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 5632*PAGE_SIZE ; offset for twelfth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_12 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_12

	mov ecx, 0 ; counter
.loop_13:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 6144*PAGE_SIZE ; offset for thirteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_13 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_13

	mov ecx, 0 ; counter
.loop_14:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 6656*PAGE_SIZE ; offset for fourteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_14 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_14

	mov ecx, 0 ; counter
.loop_15:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 7168*PAGE_SIZE ; offset for fifteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_15 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_15

	mov ecx, 0 ; counter
.loop_16:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 7680*PAGE_SIZE ; offset for sixteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_16 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_16

	mov ecx, 0 ; counter
.loop_17:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 8192*PAGE_SIZE ; offset for seventeenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_17 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_17

	mov ecx, 0 ; counter
.loop_18:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 8704*PAGE_SIZE ; offset for eighteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_18 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_18

	mov ecx, 0 ; counter
.loop_19:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 9216*PAGE_SIZE ; offset for nineteenth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_19 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_19

	mov ecx, 0 ; counter
.loop_20:
	mov eax, PAGE_SIZE
	mul ecx
	add eax, 9728*PAGE_SIZE ; offset for twentieth page table
	or eax, PAGE_ENTRY_FLAGS
	mov [page_table_l1_20 + ecx * 8], eax
	inc ecx
	cmp ecx, 512
	jne .loop_20

	ret

enable_paging:
	; pass page table location to cpu
	mov eax, page_table_l4
	mov cr3, eax

	; enable PAE
	mov eax, cr4
	or eax, 1 << 5
	mov cr4, eax

	; enable long mode
	mov ecx, 0xC0000080
	rdmsr
	or eax, 1 << 8
	wrmsr

	; enable paging
	mov eax, cr0
	or eax, 1 << 31
	mov cr0, eax

	ret

error:
	; print "ERR: X" where X is the error code
	mov dword [0xb8000], 0x4f524f45
	mov dword [0xb8004], 0x4f3a4f52
	mov dword [0xb8008], 0x4f204f20
	mov byte  [0xb800a], al
	hlt

section .bootbss
align 4096
page_table_l4:
	resb 4096
page_table_l3:
	resb 4096
page_table_l2:
	resb 4096
; uncomment all page_table_l1_* for 4 KiB page size
page_table_l1_1:
	resb 4096
page_table_l1_2:
	resb 4096
page_table_l1_3:
	resb 4096
page_table_l1_4:
	resb 4096
page_table_l1_5:
	resb 4096
page_table_l1_6:
	resb 4096
page_table_l1_7:
	resb 4096
page_table_l1_8:
	resb 4096
page_table_l1_9:
	resb 4096
page_table_l1_10:
	resb 4096
page_table_l1_11:
	resb 4096
page_table_l1_12:
	resb 4096
page_table_l1_13:
	resb 4096
page_table_l1_14:
	resb 4096
page_table_l1_15:
	resb 4096
page_table_l1_16:
	resb 4096
page_table_l1_17:
	resb 4096
page_table_l1_18:
	resb 4096
page_table_l1_19:
	resb 4096
page_table_l1_20:
	resb 4096
; from 21 on will be used for kernel heap
page_table_l1_21:
	resb 4096
; special l1 page table which will be used to map non-sequential page frames like ACPI/HPET
page_table_l1_special:
	resb 4096
; TODO Why is stack in bss? Does this make sense? Also Stack is pretty small
; TODO stack should be moved to end of memory when paging is starting
stack_bottom:
	resb 4096 * 4000
stack_top:

section .bootrodata
gdt64:
	dq 0 ; zero entry
.code_segment: equ $ - gdt64
	dq (1 << 43) | (1 << 44) | (1 << 47) | (1 << 53) ; code segment
.pointer:
	dw $ - gdt64 - 1 ; length
	dq gdt64 ; address
