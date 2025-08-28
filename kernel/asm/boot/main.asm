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
	%define KERNEL_SIZE 0x2800000      ; 40 MiB kernel size SYNCID2
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

	; Deduplicated L1 page table setup
	mov esi, page_tables_l1          ; ESI = base address of first L1 table
	;mov edi, page_table_l2           ; EDI = base address of L2 table
	mov ebx, 0                       ; EBX = L1 table index
.l1_table_loop:
	mov eax, esi
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2 + ebx*8], eax           ; Set L2 entry to point to L1 table

	; Fill L1 table
	mov ecx, 0
.l1_entry_loop:
		mov eax, PAGE_SIZE
		mul ecx
		mov edx, ebx
		imul edx, 512*PAGE_SIZE
		add eax, edx
		or eax, PAGE_ENTRY_FLAGS
		mov [esi + ecx*8], eax
		inc ecx
		cmp ecx, 512
		jne .l1_entry_loop

	add esi, 4096                    ; Next L1 table (each is 4096 bytes)
	inc ebx
	cmp ebx, (NUM_KERNEL_PAGES + 511) / 512    ; 20 L1 tables (1..20)
	jne .l1_table_loop

	; Special L1 table for non-sequential frames
	mov eax, page_tables_l1_heap
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+((NUM_KERNEL_PAGES + 511) / 512)*8], eax
	
	; Special L1 table for non-sequential frames
	mov eax, page_table_l1_special
	or eax, PAGE_TABLE_FLAGS
	mov [page_table_l2+510*8], eax

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
; uncomment for 4 KiB page size
; from 21 on will be used for kernel heap
page_tables_l1:
	resb (NUM_KERNEL_PAGES + 511) / 512 * 4096
page_tables_l1_heap:
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
